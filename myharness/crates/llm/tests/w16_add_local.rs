//! W16 add-local 의 L2 Integration TC (D-59, §7.2 / TC_INTEGRATION.md §W16-AddLocal).
//!
//! mock strategy: wiremock + tempfile + MYHARNESS_HOME env override.

use myharness_llm::add_local::{probe_local_models, register_local_provider};
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
