//! myharness — yklee 의 개인 코딩 에이전트 CLI/TUI
//!
//! v1 MVP skeleton (TASK-005-1 W2).
//! 본 구현은 TASK-005-1 W3~W11 에서 진행.

use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "myharness",
    version,
    about = "yklee 의 개인 코딩 에이전트 CLI/TUI"
)]
struct Args {
    /// 모드 (orchestrator | single | loop) — CONCEPT.md §5.10
    #[arg(long, default_value = "orchestrator")]
    mode: String,

    /// 모든 권한 prompt 자동 y (CI / 비대화형 환경)
    #[arg(long, short = 'y')]
    yes: bool,

    /// Bash sanitizer 모드 (strict | permissive | off)
    #[arg(long, default_value = "strict")]
    safe_mode: String,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();
    tracing::info!(mode = %args.mode, "myharness v0.1.0 (TASK-005-1 W2 skeleton)");

    println!("myharness v0.1.0 — mode={}", args.mode);
    println!("v1 MVP skeleton. 본 구현은 TASK-005-1 W3~W11 에서 진행 예정.");

    Ok(())
}
