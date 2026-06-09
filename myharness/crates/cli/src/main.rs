//! myharness — yklee 의 개인 코딩 에이전트 CLI/TUI
//!
//! v1 MVP (TASK-005-1 W10):
//! - subcommand: code <action>, env <action>, git <action>, ask (single mode), loop (loop mode)
//! - 3-모드: orchestrator (default) | single | loop (CONCEPT.md §5.10)
//! - sub-agent registry: code-reviewer, code-implementer, env-diagnose, git-operator (CONCEPT.md §5.11)

use std::sync::Arc;

use clap::{Parser, Subcommand};
use myharness_llm::LLMClient;
use myharness_tools::ToolRegistry;
use myharness_tui::{App, AppKey, LoopConfig, LoopRunner, MessageRole, Orchestrator, TtyGuard};

#[derive(Parser, Debug)]
#[command(
    name = "myharness",
    version,
    about = "yklee 의 개인 코딩 에이전트 CLI/TUI"
)]
struct Args {
    #[arg(long, default_value = "orchestrator")]
    mode: String,

    #[arg(long, short = 'y')]
    yes: bool,

    #[arg(long, default_value = "strict")]
    safe_mode: String,

    #[arg(long)]
    goal: Option<String>,

    #[arg(long)]
    success_criteria: Option<String>,

    #[arg(long, default_value_t = 20)]
    max_iterations: u32,

    #[command(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    Code { #[command(subcommand)] action: CodeAction },
    Env { #[command(subcommand)] action: EnvAction },
    Git { #[command(subcommand)] action: GitAction },
    Ask { question: String },
}

#[derive(Subcommand, Debug)]
enum CodeAction {
    Review { target: String },
    Implement { feature: String },
}

#[derive(Subcommand, Debug)]
enum EnvAction {
    Diagnose,
}

#[derive(Subcommand, Debug)]
enum GitAction {
    Commit { message: String },
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();
    tracing::info!(mode = %args.mode, "myharness v0.1.0 (W10)");

    let rt = tokio::runtime::Runtime::new()?;
    let _guard = rt.enter();

    let orch = Orchestrator::new()
        .with_tools(Arc::new(ToolRegistry::default_tools()));
    let llm: Arc<dyn LLMClient> = Arc::new(myharness_llm::client_mock::MockClient::new(
        myharness_llm::provider::ProviderId::Claude,
        "claude-sonnet-4-6",
    ));
    let orch = orch.with_llm(llm);

    match args.cmd {
        Some(Cmd::Code { action }) => match action {
            CodeAction::Review { target } => {
                let input = format!("code review {target}");
                let out = rt.block_on(orch.run(&input))?;
                println!("{out}");
            }
            CodeAction::Implement { feature } => {
                let input = format!("code implement {feature}");
                let out = rt.block_on(orch.run(&input))?;
                println!("{out}");
            }
        },
        Some(Cmd::Env { action }) => match action {
            EnvAction::Diagnose => {
                let out = rt.block_on(orch.run("env diagnose"))?;
                println!("{out}");
            }
        },
        Some(Cmd::Git { action }) => match action {
            GitAction::Commit { message } => {
                let input = format!("git commit \"{message}\"");
                let out = rt.block_on(orch.run(&input))?;
                println!("{out}");
            }
        },
        Some(Cmd::Ask { question }) => {
            let out = rt.block_on(orch.run(&question))?;
            println!("{out}");
        }
        None => match args.mode.as_str() {
            "loop" => {
                let goal = args.goal.unwrap_or_else(|| {
                    eprintln!("--goal required for loop mode");
                    std::process::exit(1);
                });
                let cfg = LoopConfig {
                    goal,
                    success_criteria: args.success_criteria,
                    max_iterations: args.max_iterations,
                };
                let runner = LoopRunner::new(cfg);
                let report = rt.block_on(runner.run(&orch));
                println!("loop finished: stop={:?} iterations={}", report.stop, report.total_iterations);
            }
            "orchestrator" | "single" => {
                let _tty = TtyGuard::enter()?;
                let mut app = App::new("myharness", &args.mode);
                app.push_message(myharness_tui::AppMessage::system(format!(
                    "Mode: {} (type a message, Ctrl+C to quit)",
                    args.mode
                )));
                let backend = ratatui::backend::CrosstermBackend::new(std::io::stdout());
                let mut terminal = ratatui::Terminal::new(backend)?;
                loop {
                    terminal.draw(|f| myharness_tui::draw(f, &mut app))?;
                    let key = AppKey::read()?;
                    app.apply_key(key);
                    if !app.running {
                        break;
                    }
                }
                drop(_tty);
                println!("\n--- session ended ---");
                for m in &app.messages {
                    let prefix = match m.role {
                        MessageRole::User => "you",
                        MessageRole::Assistant => "bot",
                        MessageRole::System => "sys",
                        MessageRole::Tool => "tool",
                        MessageRole::Error => "err",
                    };
                    println!("[{}] {}", prefix, m.content);
                }
            }
            other => {
                eprintln!("unknown mode: {other}");
                std::process::exit(1);
            }
        },
    }

    Ok(())
}
