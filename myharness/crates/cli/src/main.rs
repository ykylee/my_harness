//! myharness — yklee 의 개인 코딩 에이전트 CLI/TUI
//!
//! v1 MVP (TASK-005-1 W11):
//! - subcommand: code <action>, env <action>, git <action>, ask, loop, task <start|end>

use std::sync::Arc;

use clap::{Parser, Subcommand};
use myharness_core::{
    EventLog, HandoffDoc, PermissionMode, PermissionPolicy, RiskKind, TaskEndReport,
    TaskStartReport, TaskStatus,
};
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
    /// standard_ai_workflow 호환 task 관리
    Task { #[command(subcommand)] action: TaskAction },
    /// handoff 생성 (다음 세션/agent 전달용)
    Handoff {
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
    },
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

#[derive(Subcommand, Debug)]
enum TaskAction {
    Start {
        #[arg(long)]
        id: String,
        #[arg(long)]
        title: String,
        #[arg(long)]
        intent: String,
    },
    End {
        #[arg(long)]
        id: String,
        #[arg(long)]
        title: String,
        #[arg(long)]
        summary: String,
        #[arg(long, value_delimiter = ',')]
        risks: Vec<String>,
        #[arg(long, value_delimiter = ',')]
        follow_up: Vec<String>,
    },
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();
    let mode = args.mode.as_str();
    let _safe_mode = args.safe_mode.as_str();
    let policy = PermissionPolicy::new(parse_permission_mode(&args.mode))
        .with_auto_approve(args.yes);

    tracing::info!(mode = %mode, policy = ?policy.mode, "myharness v0.1.0 (W11)");

    let rt = tokio::runtime::Runtime::new()?;
    let _guard = rt.enter();

    match args.cmd {
        Some(Cmd::Task { action }) => {
            return run_task(action);
        }
        Some(Cmd::Handoff { from, to }) => {
            return run_handoff(from, to);
        }
        _ => {}
    }

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
        Some(Cmd::Task { .. }) | Some(Cmd::Handoff { .. }) => unreachable!(),
        None => match mode {
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
                let mut app = App::new("myharness", mode);
                app.push_message(myharness_tui::AppMessage::system(format!(
                    "Mode: {mode} (type a message, Ctrl+C to quit)"
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
                    println!("[{prefix}] {}", m.content);
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

fn parse_permission_mode(s: &str) -> PermissionMode {
    match s {
        "accept-edits" | "acceptEdits" => PermissionMode::AcceptEdits,
        "plan" => PermissionMode::Plan,
        "bypass-permissions" | "bypassPermissions" => PermissionMode::BypassPermissions,
        _ => PermissionMode::Default,
    }
}

fn run_task(action: TaskAction) -> anyhow::Result<()> {
    match action {
        TaskAction::Start { id, title, intent } => {
            let log = EventLog::new();
            run_task_start(&id, &title, &intent, log)?;
        }
        TaskAction::End { id, title, summary, risks, follow_up } => {
            let log = EventLog::new();
            run_task_end(&id, &title, &summary, &risks, &follow_up, log)?;
        }
    }
    Ok(())
}

fn run_task_start(id: &str, title: &str, intent: &str, mut log: EventLog) -> anyhow::Result<()> {
    log.info(format!("task start: {id}"));
    let report = TaskStartReport::new(id, title, intent);
    println!("{}", report.to_korean());
    Ok(())
}

fn run_task_end(
    id: &str,
    title: &str,
    summary: &str,
    risks: &[String],
    follow_up: &[String],
    mut log: EventLog,
) -> anyhow::Result<()> {
    log.info(format!("task end: {id}"));
    let mut report = TaskEndReport::new(id, title, summary).with_status(TaskStatus::Done);
    for r in risks {
        // risk format: "kind:description" (kind=environment|context|dependency|general)
        if let Some((kind, desc)) = r.split_once(':') {
            let rk = match kind {
                "environment" => RiskKind::Environment,
                "context" => RiskKind::Context,
                "dependency" => RiskKind::Dependency,
                _ => RiskKind::General,
            };
            report.add_risk(rk, desc.trim().to_string());
        } else {
            report.add_risk(RiskKind::General, r.clone());
        }
    }
    for f in follow_up {
        // follow_up format: "id|title|description" (| 가독성 ↑). 또는 "id:title:description"
        // | 가 있으면 splitn(3, '|'), 아니면 ':' 로 시도
        let (id, title, desc) = if f.contains('|') {
            let parts: Vec<&str> = f.splitn(3, '|').collect();
            (parts[0].to_string(), parts[1].to_string(), parts.get(2).unwrap_or(&"").to_string())
        } else if f.matches(':').count() >= 2 {
            let parts: Vec<&str> = f.splitn(3, ':').collect();
            (parts[0].to_string(), parts[1].to_string(), parts[2].to_string())
        } else {
            ("FOLLOWUP".to_string(), "follow-up".to_string(), f.clone())
        };
        report.add_follow_up(id, title, desc);
    }
    println!("{}", report.to_korean());
    Ok(())
}

fn run_handoff(from: String, to: String) -> anyhow::Result<()> {
    let h = HandoffDoc::new(from, to);
    println!("{}", h.to_korean());
    Ok(())
}
