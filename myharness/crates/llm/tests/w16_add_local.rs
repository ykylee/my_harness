//! W16 add-local 의 L2 Integration TC (D-59, §7.2 / TC_INTEGRATION.md §W16-AddLocal).
//!
//! mock strategy: wiremock + tempfile + MYHARNESS_HOME env override.

use myharness_llm::add_local::{
    probe_local_models, register_local_provider, register_local_provider_non_interactive,
};
use myharness_llm::{ModelInfo, ProviderId, ProviderRegistry};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ── W18 (v1.5 R-4 대응) L2 Integration ────────────────────────────────────────

/// TC-W18-I01 — register 시 backup 자동 생성 → 연속 register 후 backup 검증
#[tokio::test]
#[serial_test::serial(env)]
async fn tc_w18_i01_register_creates_backup_before_overwrite() {
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("MYHARNESS_HOME", tmp.path()); }

    let server = MockServer::start().await;
    let body = serde_json::json!({
        "data": [{"id": "first-model", "owned_by": "test"}]
    });
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;

    let base_url = format!("{}/v1", server.uri());

    // 1) 첫 register — backup ❌
    let r1 = register_local_provider(
        base_url.clone(),
        None,
        ModelInfo { id: "first-model".into(), owned_by: None },
        vec![ModelInfo { id: "first-model".into(), owned_by: None }],
    )
    .await
    .unwrap();
    assert_eq!(r1.model_id, "first-model");

    // 2) mock body 갱신 후 두 번째 register — backup 1개
    let body2 = serde_json::json!({
        "data": [{"id": "second-model", "owned_by": "test"}]
    });
    // 기존 mock unmount + 재mount 는 wiremock API 복잡 → second register 시 mock server 다른 endpoint 사용
    let server2 = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body2))
        .mount(&server2)
        .await;
    let base_url2 = format!("{}/v1", server2.uri());

    // ts 가 동일하면 filename 동일 → sleep 으로 분기
    std::thread::sleep(std::time::Duration::from_millis(1100));

    let _r2 = register_local_provider(
        base_url2,
        None,
        ModelInfo { id: "second-model".into(), owned_by: None },
        vec![ModelInfo { id: "second-model".into(), owned_by: None }],
    )
    .await
    .unwrap();

    // backup 1개 존재 확인 (first-model 내용)
    let backups: Vec<_> = std::fs::read_dir(tmp.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().contains(".backup."))
        .collect();
    assert_eq!(backups.len(), 1, "두 번째 register 후 backup 1개");
    let backup_content = std::fs::read_to_string(backups[0].path()).unwrap();
    assert!(backup_content.contains("first-model"), "backup = first register");

    unsafe { std::env::remove_var("MYHARNESS_HOME"); }
}

/// TC-W18-I02 — backup helper 직접 호출 (max_retention 검증)
#[tokio::test]
#[serial_test::serial(env)]
async fn tc_w18_i02_backup_max_retention_keeps_only_n_files() {
    use myharness_llm::backup_providers_toml;
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("providers.toml");

    // 7개 backup 수동 생성 (max=3)
    for i in 0..7 {
        std::fs::write(&path, format!("v{i}\n")).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
        let _ = backup_providers_toml(&path, 3);
    }

    let backups: Vec<_> = std::fs::read_dir(tmp.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().contains(".backup."))
        .collect();
    assert!(backups.len() <= 3, "max 3개 유지, 실제 {}개", backups.len());
}

#[tokio::test]
#[serial_test::serial(env)]
async fn tc_w16_i01_probe_extracts_three_models() {
    let server = MockServer::start().await;
    let body = serde_json::json!({
        "object": "list",
        "data": [
            {"id": "llama3.1:8b", "object": "model", "owned_by": "ollama"},
            {"id": "qwen2.5:14b", "object": "model", "owned_by": "ollama"},
            {"id": "mistral:7b", "object": "model", "owned_by": "ollama"},
        ]
    });
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;

    // wiremock::MockServer::uri() 가 suffix 안 가지므로 /v1 명시
    let base_url = format!("{}/v1", server.uri());
    let models = probe_local_models(&base_url, None).await.unwrap();

    assert_eq!(models.len(), 3);
    let ids: Vec<&str> = models.iter().map(|m| m.id.as_str()).collect();
    assert!(ids.contains(&"llama3.1:8b"));
    assert!(ids.contains(&"qwen2.5:14b"));
    assert!(ids.contains(&"mistral:7b"));
    assert!(models.iter().all(|m| m.owned_by.as_deref() == Some("ollama")));
}

#[tokio::test]
#[serial_test::serial(env)]
async fn tc_w16_i02_probe_returns_http_error_on_401() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized: invalid token"))
        .mount(&server)
        .await;

    let base_url = format!("{}/v1", server.uri());
    let err = probe_local_models(&base_url, Some("bad-token")).await.unwrap_err();

    match err {
        myharness_llm::add_local::RegisterError::HttpError { status, body, .. } => {
            assert_eq!(status, 401);
            assert!(body.contains("Unauthorized"));
        }
        e => panic!("expected HttpError, got {e:?}"),
    }
}

#[tokio::test]
#[serial_test::serial(env)]
async fn tc_w16_i03_register_writes_providers_toml_end_to_end() {
    let server = MockServer::start().await;
    let body = serde_json::json!({
        "data": [
            {"id": "llama3.1:8b", "owned_by": "ollama"},
            {"id": "qwen2.5:14b", "owned_by": "ollama"},
        ]
    });
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;

    // isolate via tempdir
    let tmp = tempfile::tempdir().unwrap();
    // SAFETY: serial_test::serial(env) 로 다른 env-mutating test 와 직렬화됨
    unsafe { std::env::set_var("MYHARNESS_HOME", tmp.path()); }

    let base_url = format!("{}/v1", server.uri());
    let models = probe_local_models(&base_url, None).await.unwrap();
    let selected = models.iter().find(|m| m.id == "qwen2.5:14b").unwrap().clone();

    let report = register_local_provider(base_url.clone(), None, selected, models).await.unwrap();

    assert_eq!(report.model_id, "qwen2.5:14b");
    assert_eq!(
        report.available_models,
        vec!["llama3.1:8b".to_string(), "qwen2.5:14b".to_string()]
    );

    // providers.toml 검증
    let toml_path = tmp.path().join("providers.toml");
    assert!(toml_path.exists(), "providers.toml must be created");
    let content = std::fs::read_to_string(&toml_path).unwrap();
    assert!(content.contains("qwen2.5:14b"));
    assert!(content.contains("llama3.1:8b"));
    assert!(content.contains(&base_url));

    // registry reload 검증
    let registry = ProviderRegistry::load_from_path(&toml_path).unwrap();
    let local = registry.get(ProviderId::LocalLlm).unwrap();
    assert_eq!(local.base_url, base_url);
    assert_eq!(local.default_model, "qwen2.5:14b");
    assert_eq!(local.available_models.len(), 2);

    // SAFETY: serial_test::serial(env) cleanup
    unsafe { std::env::remove_var("MYHARNESS_HOME"); }
}

// suppress unused import warning
#[allow(dead_code)]
fn _suppress_unused_modelinfo(_m: ModelInfo) {}

// ── W17 (v1.5 OI-1) L2 Integration ─────────────────────────────────────────

/// TC-W17-I01 — 비대화형 모드에서 probe 스킵 → register 만 수행 → providers.toml 갱신.
///
/// wiremock 으로 mock server 띄우지만 **probe 가 호출되지 않음** 을 검증:
///   - mock server 에 어떤 route 도 mount 하지 않음 → 호출 시 connection refused 면 비대화형은 성공 (probe skip)
///   - 단, **비대화형 함수가 probe 를 안 부른다** 는 것을 wiremock 으로 증명하기 위해
///     200 응답하는 mock server 를 띄우고 "이 endpoint 에는 어떤 HTTP 요청도 가지 않는다" 를
///     server 측 unreachable 로 확인하는 게 더 명확. 본 TC 는 단순히 register 자체에 집중.
#[tokio::test]
#[serial_test::serial(env)]
async fn tc_w17_i01_non_interactive_skips_probe_and_writes_toml() {
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("MYHARNESS_HOME", tmp.path()); }

    // wiremock 띄우되 어떤 route 도 mount 안 함 → 어떤 HTTP 요청이 와도 404
    let server = MockServer::start().await;

    let base_url = format!("{}/v1", server.uri());
    let report = register_local_provider_non_interactive(
        base_url.clone(),
        None,
        "ci-model".into(),
    )
    .await
    .unwrap();

    // available_models = [ci-model] 1개 (probe 안 했음의 증거)
    assert_eq!(report.available_models, vec!["ci-model".to_string()]);
    assert_eq!(report.model_id, "ci-model");

    // providers.toml 검증
    let toml_path = tmp.path().join("providers.toml");
    assert!(toml_path.exists());
    let content = std::fs::read_to_string(&toml_path).unwrap();
    assert!(content.contains("ci-model"));
    assert!(content.contains(&base_url));

    // registry reload → default_model 이 ci-model 로 set 됨
    let registry = ProviderRegistry::load_from_path(&toml_path).unwrap();
    let local = registry.get(ProviderId::LocalLlm).unwrap();
    assert_eq!(local.default_model, "ci-model");
    assert_eq!(local.available_models, vec!["ci-model".to_string()]);

    unsafe { std::env::remove_var("MYHARNESS_HOME"); }
}

/// TC-W17-I02 — 비대화형 모드에서 token + base_url + model_id 모두 set → keyring set + register.
///
/// CI 환경 시뮬레이션: stdin/stdout non-tty 일 때 비대화형 함수는 정상 동작.
#[tokio::test]
#[serial_test::serial(env)]
async fn tc_w17_i02_non_interactive_with_token_end_to_end() {
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("MYHARNESS_HOME", tmp.path()); }

    let server = MockServer::start().await;
    let base_url = format!("{}/v1", server.uri());

    let report = register_local_provider_non_interactive(
        base_url.clone(),
        Some("ci-secret-token-abc".into()),
        "gpt-oss:20b".into(),
    )
    .await
    .unwrap();

    assert!(report.token_saved);
    assert_eq!(report.model_id, "gpt-oss:20b");

    // providers.toml 검증
    let toml_path = tmp.path().join("providers.toml");
    assert!(toml_path.exists());

    unsafe { std::env::remove_var("MYHARNESS_HOME"); }
}

// ── W20 (v1.5 D-63 F-3) Ollama native /api/tags cascade ─────────────────────

/// TC-W20-I01 — Ollama native `/api/tags` 200 응답 시 cascade stage 1 에서 성공
#[tokio::test]
async fn tc_w20_i01_probe_ollama_native_api_tags_succeeds() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "models": [
                {"name": "llama3.1:8b", "details": {"family": "llama"}},
                {"name": "qwen2.5:14b", "details": {"family": "qwen2"}}
            ]
        })))
        .mount(&server)
        .await;

    // base = "http://server" (no /v1 suffix, bare host) — Ollama native 의 canonical
    let base_url = server.uri();
    let models = probe_local_models(&base_url, None).await.unwrap();

    assert_eq!(models.len(), 2);
    assert_eq!(models[0].id, "llama3.1:8b");
    assert_eq!(models[0].owned_by.as_deref(), Some("llama"));
    assert_eq!(models[1].id, "qwen2.5:14b");
    assert_eq!(models[1].owned_by.as_deref(), Some("qwen2"));
}

/// TC-W20-I02 — `/api/tags` 404 (Ollama OpenAI compat only or vLLM/LM Studio/llama.cpp)
/// → cascade 가 `/v1/models` 로 fallback 성공
#[tokio::test]
async fn tc_w20_i02_probe_cascade_fallback_to_openai_compat() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/tags"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "vllm-model", "owned_by": "vllm"}]
        })))
        .mount(&server)
        .await;

    let base_url = server.uri();
    let models = probe_local_models(&base_url, None).await.unwrap();

    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "vllm-model");
    assert_eq!(models[0].owned_by.as_deref(), Some("vllm"));
}

/// TC-W20-I03 — 양쪽 다 200 응답 시 native 가 우선 (early return, OpenAI 미호출)
#[tokio::test]
async fn tc_w20_i03_probe_ollama_native_takes_priority_over_openai_compat() {
    let server = MockServer::start().await;

    // native 가 OpenAI 와 다른 모델을 반환 — OpenAI 가 호출되면 다른 결과로 detect 가능
    Mock::given(method("GET"))
        .and(path("/api/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "models": [{"name": "native-model", "details": {"family": "llama"}}]
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "openai-compat-model", "owned_by": "openai"}]
        })))
        .mount(&server)
        .await;

    let base_url = server.uri();
    let models = probe_local_models(&base_url, None).await.unwrap();

    // native 가 우선 → native-model 만 반환 (openai-compat-model 미반환)
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "native-model");
}
