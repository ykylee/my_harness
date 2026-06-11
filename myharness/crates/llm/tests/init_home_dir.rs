//! §5.12 `init_home_dir` 통합 테스트 (tempdir 격리).
//!
//! 기존 `w16_add_local.rs/w16_scenarios.rs` 와 같은 패턴으로,
//! `MYHARNESS_HOME` 을 tempdir 로 강제 + `init_home_dir` 호출 후
//! §5.12 spec 의 11개 디렉토리가 모두 존재하는지 검증.

use std::path::Path;

use myharness_llm::init_home_dir;
use serial_test::serial;

fn run_with_temp_home<F: FnOnce(&Path)>(f: F) {
    let dir = tempfile::tempdir().expect("tempdir");
    // tempdir 의 path 를 MYHARNESS_HOME 으로 강제
    // (set_var 는 unsafe in Rust 2024 edition — safety: serial test, env restored)
    let prev = std::env::var("MYHARNESS_HOME").ok();
    unsafe {
        std::env::set_var("MYHARNESS_HOME", dir.path());
    }
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(dir.path())));
    unsafe {
        match prev {
            Some(v) => std::env::set_var("MYHARNESS_HOME", v),
            None => std::env::remove_var("MYHARNESS_HOME"),
        }
    }
    if let Err(e) = result {
        std::panic::resume_unwind(e);
    }
}

#[test]
#[serial]
fn init_home_dir_creates_all_5_12_dirs() {
    run_with_temp_home(|home| {
        let got = init_home_dir().expect("init_home_dir");
        assert_eq!(got, home, "init returns the same path as MYHARNESS_HOME");

        // 7 top-level
        for sub in [
            "config",
            "state",
            "memory",
            "handoff",
            "compression",
            "sub-agents",
            "auth",
        ] {
            let p = home.join(sub);
            assert!(p.is_dir(), "missing top-level dir: {p:?}");
        }

        // state subdir (CONCEPT §5.12)
        assert!(home.join("state").join("auth").is_dir());

        // 2 additional
        for sub in ["runtime", "cache"] {
            let p = home.join(sub);
            assert!(p.is_dir(), "missing additional dir: {p:?}");
        }
    });
}

#[test]
#[serial]
fn init_home_dir_is_idempotent() {
    run_with_temp_home(|_home| {
        init_home_dir().expect("first init");
        init_home_dir().expect("second init (idempotent)");
    });
}

#[test]
#[serial]
fn init_home_dir_preserves_existing_files() {
    run_with_temp_home(|home| {
        let state_dir = home.join("state");
        std::fs::create_dir_all(&state_dir).unwrap();
        let marker = state_dir.join("marker.txt");
        std::fs::write(&marker, "preserved").unwrap();

        init_home_dir().expect("init");

        assert!(marker.exists(), "existing file must be preserved");
        let content = std::fs::read_to_string(&marker).unwrap();
        assert_eq!(content, "preserved");
    });
}
