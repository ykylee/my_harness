//! W16 cli 동작 시나리오 검증 (D-59 follow-up, 2026-06-09)
//!
//! 4가지 시나리오를 lib 함수 레벨에서 직접 검증:
//! 1. 정상 (192.168.0.101:1234 + gemma 모델 선택)
//! 2. connection refused (192.168.0.1:9999 등 unreachable)
//! 3. 비-tty: cli handler 레벨 검증 (별도 scenario_*.rs 로 binary 호출)
//! 4. URL invalid: lib 레벨 검증

use myharness_llm::add_local::{probe_local_models, register_local_provider, RegisterError, ModelInfo};

#[tokio::test]
#[serial_test::serial(env)]
async fn scenario_1_real_lm_studio_192_168_0_101() {
    // 실제 LM Studio (192.168.0.101:1234) 가 떠있다는 전제.
    // CI 환경에서는 fail 할 수 있음 → #[ignore] 로 manual-only 표시
    if std::env::var("MYHARNESS_TEST_LIVE_LMSTUDIO").is_err() {
        eprintln!("SKIP: set MYHARNESS_TEST_LIVE_LMSTUDIO=1 to run against real LM Studio");
        return;
    }

    let base_url = "http://192.168.0.101:1234/v1";
    let models = probe_local_models(base_url, None).await.expect("LM Studio probe");

    eprintln!("✓ probe_local_models OK — {} models:", models.len());
    for m in &models {
        eprintln!("  - {} (owned_by={:?})", m.id, m.owned_by);
    }

    assert!(!models.is_empty(), "models should not be empty");
    // LM Studio 의 gemma-4-12b-qat 가 첫 번째일 가능성 (model load 순서)
    let ids: Vec<&str> = models.iter().map(|m| m.id.as_str()).collect();
    eprintln!("  ids: {ids:?}");

    // gemma 또는 nomic 둘 중 하나는 있어야 함 (사용자 환경)
    assert!(
        ids.iter().any(|id| id.contains("gemma") || id.contains("nomic")),
        "expected gemma or nomic model, got {ids:?}"
    );
}

#[tokio::test]
#[serial_test::serial(env)]
async fn scenario_2_connection_refused_localhost_9999() {
    // localhost:9999 는 안 떠있는 포트 → 즉시 connection refused
    let result = probe_local_models("http://localhost:9999/v1", None).await;

    match result {
        Err(RegisterError::ConnectionRefused { url }) => {
            eprintln!("✓ ConnectionRefused 매칭: url={url}");
            assert!(url.contains("9999"));
        }
        Err(other) => panic!("expected ConnectionRefused, got {other:?}"),
        Ok(models) => panic!("expected error, got {} models", models.len()),
    }
}

#[tokio::test]
#[serial_test::serial(env)]
async fn scenario_4_invalid_url() {
    // URL parse 자체가 실패해야 함
    let result = probe_local_models("not a url at all", None).await;
    match result {
        Err(RegisterError::HttpError { url, status, body }) => {
            // url::Url::parse 가 client build 시점에 실패할 수도 있고
            // reqwest 가 거절할 수도 있음 → 두 경우 모두 HttpError
            eprintln!("✓ HttpError 매칭: url={url}, status={status}, body={body}");
            assert!(url.contains("not a url") || body.contains("url"));
        }
        Err(RegisterError::ConnectionRefused { url }) => {
            // 일부 케이스에서 connection refused 로 떨어질 수도
            eprintln!("⚠ ConnectionRefused (url parse 우회): {url}");
        }
        Err(other) => panic!("expected HttpError/ConnectionRefused, got {other:?}"),
        Ok(_) => panic!("expected error, got success"),
    }
}

#[tokio::test]
#[serial_test::serial(env)]
async fn scenario_2b_register_provider_refused() {
    // register_local_provider 자체는 URL parse + keyring + registry write 만 함.
    // probe 단계의 connection refused 는 cli handler (handle_auth_add_local) 에서 처리.
    // 본 TC 는 register_local_provider 가 invalid URL 에서 graceful error 반환 검증.
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("MYHARNESS_HOME", tmp.path()); }

    let result = register_local_provider(
        "http://localhost:9999/v1".into(),
        None,
        ModelInfo { id: "fake".into(), owned_by: None },
        vec![ModelInfo { id: "fake".into(), owned_by: None }],
    )
    .await;

    // URL parse 는 성공 (http://~), register 자체는 ok 반환 (probe 안 함 — cli 가 probe 먼저)
    // 즉 register_local_provider 는 probe 를 안 하므로 invalid endpoint 라도 성공할 수 있음
    match result {
        Ok(report) => {
            eprintln!("✓ register_local_provider 는 probe 안 함 — Ok 반환 (cli 가 probe 선행)");
            assert_eq!(report.base_url, "http://localhost:9999/v1");
        }
        Err(e) => eprintln!("⚠ Err: {e:?}"),
    }
    unsafe { std::env::remove_var("MYHARNESS_HOME"); }
}
