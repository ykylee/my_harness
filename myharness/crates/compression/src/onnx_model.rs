//! Kompress-base ML model manager (v2.0, D-67, tract Commit 1).
//!
//! Lazy singleton — first use 시 `HuggingFace` 에서 all-MiniLM-L6-v2 ONNX 모델
//! streaming download + SHA256 verify + `tract` ONNX load + inference setup.
//!
//! # Commit 1 scope
//! - `ModelManager` skeleton + download + verify + tract runnable load
//! - `embed()` 는 stub (Commit 2 에서 actual inference — tokenization + tract run)
//! - Layer 2 opt-in 유지 (기존 `kompress_v1` rule-based fallback 그대로)
//!
//! # Model metadata
//! - 출처: `sentence-transformers/all-MiniLM-L6-v2` (`HuggingFace`, Apache 2.0)
//! - 파일: `onnx/model_O4.onnx` (O4 optimized, ~45 MB, 모든 opset 호환)
//! - embedding dim: 384, max tokens: 256
//! - License 호환: Apache 2.0 ↔ myharness `MIT OR Apache-2.0` dual ✅
//!
//! # Cache path
//! `~/.cache/myharness/models/all-MiniLM-L6-v2.onnx`
//! (`dirs::cache_dir()` cross-platform)
//!
//! # CI 영향
//! - **Pure Rust** (no C++ toolchain, no ONNX Runtime binary download)
//! - binary size: ~22-33 MB 추가 (Pure Rust compiled, D-67 trade-off)
//! - Linux/macOS/Windows matrix 모두 동작 (tract 가 cross-platform)

#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss, clippy::cast_sign_loss, clippy::cast_possible_wrap)]
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use anyhow::{anyhow, Context, Result};
use sha2::{Digest, Sha256};
use tracing::{debug, info, warn};

use tract_onnx::prelude::*;

/// Process-wide lazy singleton. `ModelManager::get()` 으로 접근.
static MODEL_MANAGER: OnceLock<Arc<ModelManager>> = OnceLock::new();

/// Kompress-base 가 사용하는 ONNX 모델 메타데이터.
#[derive(Debug, Clone)]
pub struct ModelInfo {
    /// `HuggingFace` `resolve` endpoint URL (LFS binary)
    pub url: &'static str,
    /// 다운로드 후 검증할 SHA256 hex (lowercase, 64-char)
    pub sha256: &'static str,
    /// `HuggingFace` model id (license/attribution 표기용)
    pub model_id: &'static str,
    /// 출력 embedding vector 차원
    pub embedding_dim: usize,
    /// 모델이 처리 가능한 최대 input token 수
    pub max_tokens: usize,
    /// local 캐시 파일 경로의 마지막 segment
    pub cache_filename: &'static str,
}

impl Default for ModelInfo {
    fn default() -> Self {
        // `model_O4.onnx`: ONNX Runtime optimization level 4, ~45 MB float32
        // (quantized variant 는 tract 가 int8 quantization 미지원 시 정확도 손실)
        Self {
            url: "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/onnx/model_O4.onnx",
            sha256: "1667d7f3ba669048b13a96ee3a44456d5e42c8f44588ae8b603430e16160c485",
            model_id: "sentence-transformers/all-MiniLM-L6-v2",
            embedding_dim: 384,
            max_tokens: 256,
            cache_filename: "all-MiniLM-L6-v2.onnx",
        }
    }
}

/// Thread-safe model manager.
///
/// **Commit 1 scope**: download + SHA256 verify + `into_runnable()` load verify only.
/// 실제 inference (`Runnable::run()`) 는 Commit 2 에서.
pub struct ModelManager {
    info: ModelInfo,
    loaded: OnceLock<()>,
    client: reqwest::Client,
}

impl ModelManager {
    ///
    /// # Errors
    ///
    /// This function returns an error if the underlying operation fails.
    /// Process-wide singleton 접근. 첫 호출 시 instance 생성.
    pub fn get() -> Result<Arc<Self>> {
        if let Some(mm) = MODEL_MANAGER.get() {
            return Ok(mm.clone());
        }
        let mm = Arc::new(Self::new()?);
        MODEL_MANAGER
            .set(mm.clone())
            .map_err(|_| anyhow!("ModelManager::get: race lost, singleton already set"))?;
        Ok(mm)
    }

    ///
    /// # Errors
    ///
    /// This function returns an error if the underlying operation fails.
    /// 새 instance 생성 (download/load 안 함, singleton 등록 안 함).
    pub fn new() -> Result<Self> {
        Ok(Self {
            info: ModelInfo::default(),
            loaded: OnceLock::new(),
            client: reqwest::Client::builder()
                .user_agent(concat!("myharness/", env!("CARGO_PKG_VERSION")))
                .build()
                .context("reqwest client build")?,
        })
    }

    /// 모델 메타데이터.
    pub fn info(&self) -> &ModelInfo {
        &self.info
    }

    /// Local cache 경로: `~/.cache/myharness/models/all-MiniLM-L6-v2.onnx`
    #[must_use] 
    pub fn cache_path() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("myharness")
            .join("models")
            .join(ModelInfo::default().cache_filename)
    }

    ///
    /// # Errors
    ///
    /// This function returns an error if the underlying operation fails.
    /// 모델 파일이 다운로드되어 있고 SHA256 이 일치하는지 확인.
    /// 없거나 mismatch 면 download. 성공 시 cache path 반환.
    pub async fn ensure_downloaded(&self) -> Result<PathBuf> {
        let path = Self::cache_path();

        if path.exists() {
            let actual = sha256_file(&path).await?;
            if actual == self.info.sha256 {
                debug!("model already cached and verified: {:?}", path);
                return Ok(path);
            }
            warn!(
                "cached model sha256 mismatch (got {actual}, want {}) — re-downloading",
                self.info.sha256
            );
            tokio::fs::remove_file(&path).await.ok();
        }

        self.download(&path).await?;

        let actual = sha256_file(&path).await?;
        if actual != self.info.sha256 {
            tokio::fs::remove_file(&path).await.ok();
            anyhow::bail!(
                "sha256 mismatch after download: expected {}, got {}",
                self.info.sha256,
                actual
            );
        }
        info!("model downloaded and verified: {:?}", path);
        Ok(path)
    }

    async fn download(&self, dest: &PathBuf) -> Result<()> {
        use tokio::io::AsyncWriteExt;

        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .context("create cache dir")?;
        }

        let response = self
            .client
            .get(self.info.url)
            .send()
            .await
            .context("HF download request")?;
        let total = response.content_length().unwrap_or(0);
        info!(
            "downloading {} ({:.1} MB)",
            self.info.model_id,
            total as f64 / 1_048_576.0
        );

        let mut file = tokio::fs::File::create(dest)
            .await
            .context("create model file")?;
        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();
        while let Some(chunk) = tokio_stream::StreamExt::next(&mut stream).await {
            let chunk = chunk.context("download chunk")?;
            file.write_all(&chunk).await.context("write chunk")?;
            downloaded += chunk.len() as u64;
        }
        file.flush().await.context("flush")?;
        info!("download complete: {:.1} MB", downloaded as f64 / 1_048_576.0);
        Ok(())
    }

    ///
    /// # Errors
    ///
    /// This function returns an error if the underlying operation fails.
    /// tract ONNX model load + runnable verify. `ensure_downloaded()` 이후 호출.
    ///
    /// **Commit 1 scope**: `into_runnable()` 가 성공하는지 검증만. 실제 inference 는
    /// Commit 2 에서 `Runnable::run()` 으로 (Commit 1 은 Runnable 보관 API 미정착).
    pub fn load_runnable(&self, model_path: &PathBuf) -> Result<()> {
        if self.loaded.get().is_some() {
            return Ok(());
        }
        info!("loading tract ONNX runnable from {:?}", model_path);
        let _runnable = tract_onnx::onnx()
            .model_for_path(model_path)
            .map_err(|e| anyhow!("tract model_for_path: {e:?}"))?
            .into_runnable()
            .map_err(|e| anyhow!("tract into_runnable: {e:?}"))?;
        self.loaded
            .set(())
            .map_err(|()| anyhow!("load_runnable: race lost, runnable already set"))?;
        Ok(())
    }

    /// Runnable 이 load 됐는지.
    pub fn is_loaded(&self) -> bool {
        self.loaded.get().is_some()
    }

    ///
    /// # Errors
    ///
    /// This function returns an error if the underlying operation fails.
    /// Embedding inference. **Commit 1 stub**: Commit 2 에서 tokenization +
    /// tract run 구현 예정. 현재는 `Err` 반환하여 Kompress-base 가
    /// rule-based v1 로 graceful fallback 하도록.
    pub fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        if !self.is_loaded() {
            anyhow::bail!("runnable not loaded — call ensure_downloaded() + load_runnable() first");
        }
        anyhow::bail!("embed() is Commit 2 stub — tokenization + tract run pending");
    }
}

#[allow(clippy::format_collect)] // sha2 0.11 Array<u8, U32> 가 LowerHex 미구현 → byte 단위 hex (의도적, D-75 batch)
async fn sha256_file(path: &PathBuf) -> Result<String> {
    let data = tokio::fs::read(path).await.context("read file for sha256")?;
    let hash = Sha256::digest(&data);
    // sha2 0.11: Output (= Array<u8, U32>) 가 LowerHex 미구현 → byte 단위 hex encoding
    Ok(hash.as_slice().iter().map(|b| format!("{b:02x}")).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_path_under_myharness_models() {
        let p = ModelManager::cache_path();
        let s = p.to_string_lossy();
        assert!(s.contains("myharness"), "expected myharness dir, got {s}");
        assert!(s.contains("models"), "expected models dir, got {s}");
        assert!(
            s.ends_with("all-MiniLM-L6-v2.onnx"),
            "expected model filename, got {s}"
        );
    }

    #[test]
    #[allow(clippy::format_collect)] // sha2 0.11 Array<u8, U32> 가 LowerHex 미구현 → byte 단위 hex (의도적, D-75 batch)
    fn sha256_of_known_data_is_correct() {
        let mut hasher = Sha256::new();
        hasher.update(b"hello world");
        // sha2 0.11: Output (= Array<u8, U32>) 가 LowerHex 미구현 → byte 단위 hex encoding
        let result: String = hasher
            .finalize()
            .as_slice()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(
            result,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn model_info_defaults_are_correct() {
        let info = ModelInfo::default();
        assert_eq!(info.embedding_dim, 384);
        assert_eq!(info.max_tokens, 256);
        assert!(info.url.starts_with("https://huggingface.co/"));
        assert!(info.url.contains("sentence-transformers"));
        assert_eq!(info.sha256.len(), 64, "sha256 must be 64 hex chars");
        assert_eq!(info.model_id, "sentence-transformers/all-MiniLM-L6-v2");
        assert_eq!(info.cache_filename, "all-MiniLM-L6-v2.onnx");
    }

    #[test]
    fn model_manager_new_is_not_loaded() {
        let mm = ModelManager::new().unwrap();
        assert!(!mm.is_loaded());
        assert_eq!(mm.info().model_id, "sentence-transformers/all-MiniLM-L6-v2");
    }

    #[test]
    fn embed_stub_returns_err_before_session_loaded() {
        let mm = ModelManager::new().unwrap();
        let err = mm.embed("hello").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("runnable not loaded"),
            "expected 'runnable not loaded' error, got: {msg}"
        );
    }
}
