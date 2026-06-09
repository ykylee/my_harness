//! W16 add-local 의 L2 Integration TC (D-59, §7.2 / TC_INTEGRATION.md §W16-AddLocal).
//!
//! mock strategy: wiremock + tempfile + MYHARNESS_HOME env override.

use myharness_llm::add_local::{
    probe_local_models, register_local_provider, register_local_provider_non_interactive,
};
use myharness_llm::{ModelInfo, ProviderId, ProviderRegistry};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

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
