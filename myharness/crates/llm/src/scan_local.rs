//! 로컬 LLM 서버 HTTP probe (Ollama / vLLM / LM Studio / llama.cpp).
//!
//! 각 endpoint 에 대해 짧은 timeout 으로 GET 시도.

use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalHit {
    pub server: String,
    pub url: String,
    pub available: bool,
    #[serde(default)]
    pub models: Vec<String>,
}

const PROBE_TIMEOUT_MS: u64 = 500;

pub async fn scan_local_servers() -> Vec<LocalHit> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(PROBE_TIMEOUT_MS))
        .build()
        .expect("reqwest client build");

    let targets: &[(&str, &str)] = &[
        ("ollama", "http://localhost:11434/api/tags"),
        ("vllm", "http://localhost:8000/v1/models"),
        ("lm-studio", "http://localhost:1234/v1/models"),
        ("llama.cpp", "http://localhost:8080/v1/models"),
    ];

    let mut results = Vec::with_capacity(targets.len());
    for (name, url) in targets {
        let resp = client.get(*url).send().await;
        let hit = match resp {
            Ok(r) if r.status().is_success() => {
                let models = parse_models(name, r).await;
                LocalHit {
                    server: (*name).into(),
                    url: (*url).into(),
                    available: true,
                    models,
                }
            }
            _ => LocalHit {
                server: (*name).into(),
                url: (*url).into(),
                available: false,
                models: vec![],
            },
        };
        results.push(hit);
    }
    results
}

async fn parse_models(server: &str, resp: reqwest::Response) -> Vec<String> {
    let body = match resp.text().await {
        Ok(b) => b,
        Err(_) => return vec![],
    };
    match server {
        "ollama" => {
            #[derive(Deserialize)]
            struct O { models: Vec<OM> }
            #[derive(Deserialize)]
            struct OM { name: String }
            serde_json::from_str::<O>(&body)
                .map(|o| o.models.into_iter().map(|m| m.name).collect())
                .unwrap_or_default()
        }
        _ => {
            // OpenAI 호환: { "data": [ { "id": "model-name" } ] }
            #[derive(Deserialize)]
            struct R { data: Vec<D> }
            #[derive(Deserialize)]
            struct D { id: String }
            serde_json::from_str::<R>(&body)
                .map(|r| r.data.into_iter().map(|d| d.id).collect())
                .unwrap_or_default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn scan_local_handles_connection_refused_gracefully() {
        // 실제 서버가 없는 환경에서도 panic 없이 결과 반환
        let r = scan_local_servers().await;
        assert_eq!(r.len(), 4);
        // 모두 unavailable 이어야 함 (테스트 환경에 Ollama 안 깔려있음)
        for hit in &r {
            assert!(!hit.available, "unexpected available: {:?}", hit.server);
        }
    }

    #[tokio::test]
    async fn scan_local_returns_four_targets() {
        let r = scan_local_servers().await;
        let names: Vec<_> = r.iter().map(|h| h.server.as_str()).collect();
        assert!(names.contains(&"ollama"));
        assert!(names.contains(&"vllm"));
        assert!(names.contains(&"lm-studio"));
        assert!(names.contains(&"llama.cpp"));
    }
}
