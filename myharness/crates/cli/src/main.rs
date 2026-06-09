//! myharness — yklee 의 개인 코딩 에이전트 CLI/TUI
//!
//! v1 MVP (TASK-005-1 W13):
//! - subcommand: code <action>, env <action>, git <action>, ask, loop, task <start|end>, auth <login|logout|status>

use std::sync::Arc;

use clap::{Parser, Subcommand};
use myharness_auth::{
    find_provider, AuthManager, AuthStatus, LoginOutcome, OAUTH_PROVIDERS,
};
use myharness_core::{
    EventLog, HandoffDoc, PermissionMode, PermissionPolicy, RiskKind, TaskEndReport,
    TaskStartReport, TaskStatus,
};
use myharness_llm::LLMClient;
use myharness_tools::ToolRegistry;
use myharness_tui::{App, AppKey, LoopConfig, LoopRunner, MessageRole, Orchestrator, TtyGuard};

mod refreshing_client;

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
    Task { #[command(subcommand)] action: TaskAction },
    Handoff {
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
    },
    /// OAuth 인증 (W13) — MiniMax / OpenAI / Google
    Auth { #[command(subcommand)] action: AuthAction },
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

#[derive(Subcommand, Debug)]
enum AuthAction {
    /// OAuth 로그인. 기본: URL 출력 + browser 자동 open + polling + save.
    /// `--no-browser`: URL 출력 + polling + save (user 가 직접 browser paste).
    /// `--non-interactive`: URL 출력만 + 즉시 종료 (CI/스크립트용).
    /// OpenAI/Google: redirect flow. MiniMax: DeviceCodeFlow (W14).
    Login {
        /// provider id: minimax | openai | google
        provider: String,
        /// OAuth callback port (OpenAI/Google redirect flow 용, default 0 = OS 자동 할당)
        #[arg(long, default_value_t = 0)]
        port: u16,
        /// browser 자동 open 안 함. URL 출력 후 polling + save 계속 (user 가 직접 paste).
        #[arg(long)]
        no_browser: bool,
        /// polling + save 안 함. URL 출력만 + 즉시 종료 (CI/스크립트).
        #[arg(long)]
        non_interactive: bool,
    },
    /// OAuth 토큰 삭제
    Logout {
        provider: String,
    },
    /// 현재 OAuth 토큰 상태 확인
    Status {
        provider: String,
    },
    /// 등록된 OAuth provider 목록
    List,
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

    tracing::info!(mode = %mode, policy = ?policy.mode, "myharness v0.1.0 (W13)");

    let rt = tokio::runtime::Runtime::new()?;
    let _guard = rt.enter();

    match args.cmd {
        Some(Cmd::Task { action }) => return run_task(action),
        Some(Cmd::Handoff { from, to }) => return run_handoff(from, to),
        Some(Cmd::Auth { action }) => return rt.block_on(run_auth(action)),
        _ => {}
    }

    let orch = Orchestrator::new()
        .with_tools(Arc::new(ToolRegistry::default_tools()));
    let llm: Arc<dyn LLMClient> = resolve_llm_client();
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
        Some(Cmd::Task { .. }) | Some(Cmd::Handoff { .. }) | Some(Cmd::Auth { .. }) => unreachable!(),
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

async fn run_auth(action: AuthAction) -> anyhow::Result<()> {
    match action {
        AuthAction::List => {
            println!("registered OAuth providers:");
            for p in OAUTH_PROVIDERS.iter() {
                println!("  - {} ({}) — authorize={}, token={}", p.id(), p.display_name(), p.authorize_endpoint(), p.token_endpoint());
            }
        }
        AuthAction::Login { provider, port, no_browser, non_interactive } => {
            // MiniMax 는 W14 부터 Device Authorization Grant (D-52 follow-up) 사용.
            // 다른 provider (OpenAI, Google) 는 표준 Authorization Code + PKCE redirect flow.
            if provider == "minimax" {
                let mgr = AuthManager::new().map_err(|e| anyhow::anyhow!("{e}"))?;
                let outcome = mgr.login_minimax_device(!no_browser, non_interactive)
                    .await
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                print_login_outcome(&outcome);
            } else {
                // OpenAI/Google: redirect flow. non_interactive 는 동일 (URL 만 return).
                let p = find_provider(&provider)
                    .ok_or_else(|| anyhow::anyhow!("provider '{provider}' not found (try `myharness auth list`)"))?;
                let mgr = AuthManager::new().map_err(|e| anyhow::anyhow!("{e}"))?;
                let effective_port = if port == 0 { 0 } else { port };
                let interactive = !no_browser && !non_interactive;
                let outcome = mgr.login(p.clone(), interactive, effective_port)
                    .await
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                print_login_outcome(&outcome);
            }
        }
        AuthAction::Logout { provider } => {
            let mgr = AuthManager::new().map_err(|e| anyhow::anyhow!("{e}"))?;
            mgr.logout(&provider).map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("logged out: {provider}");
        }
        AuthAction::Status { provider } => {
            let mgr = AuthManager::new().map_err(|e| anyhow::anyhow!("{e}"))?;
            let s = mgr.status(&provider).map_err(|e| anyhow::anyhow!("{e}"))?;
            print_auth_status(&s);
        }
    }
    Ok(())
}

fn print_login_outcome(o: &LoginOutcome) {
    println!("provider: {}", o.provider);
    if o.token.access_token.is_empty() {
        // non-interactive — URL 만 return
        println!("\nTo complete login, open this URL in your browser:\n  {}\n", o.auth_url);
    } else {
        println!("login success");
        println!("  access_token: {}...", &o.token.access_token.chars().take(8).collect::<String>());
        if let Some(rt) = &o.token.refresh_token {
            println!("  refresh_token: {}...", &rt.chars().take(8).collect::<String>());
        }
        if let Some(exp) = o.token.expires_at {
            println!("  expires_at: {}", exp.format("%Y-%m-%dT%H:%M:%SZ"));
        }
        println!("\nToken saved to ~/.myharness/oauth/{}.toml", o.provider);
    }
}

fn print_auth_status(s: &AuthStatus) {
    println!("provider: {}", s.provider);
    if s.has_token {
        println!("  has_token: true");
        if let Some(p) = &s.access_token_preview {
            println!("  access_token_preview: {p}...");
        }
        println!("  refresh_token: {}", s.refresh_token_present);
        if let Some(e) = s.expires_at {
            println!("  expires_at: {}", e.format("%Y-%m-%dT%H:%M:%SZ"));
        }
        if let Some(sc) = &s.scope {
            println!("  scope: {sc}");
        }
    } else {
        println!("  has_token: false");
        println!("  run `myharness auth {} login` to authenticate", s.provider);
    }
}

/// W15.a — OAuth token store 우선 + env var fallback (opencode multi-source credential chain).
/// W15.b — OAuth 경로(1번)를 `RefreshingLlmClient` 로 wrap → 401 시 자동 refresh + 1회 retry.
///
/// 우선순위:
/// 1. `~/.myharness/oauth/minimax.toml` 의 OAuth access_token (env var 보다 우선, opencode 패턴)
///    + `RefreshingLlmClient` 가 401 시 자동 refresh (W15.b)
/// 2. `MINIMAX_API_KEY` env var (regular API key)
/// 3. `ANTHROPIC_API_KEY` env var
/// 4. MockClient fallback
fn resolve_llm_client() -> Arc<dyn LLMClient> {
    use myharness_auth::TokenStore;
    use myharness_llm::provider::ProviderId;
    use refreshing_client::RefreshingLlmClient;

    // 1) OAuth token store 우선.
    if let Ok(store) = TokenStore::new() {
        if let Ok(stored) = store.load("minimax") {
            if !stored.token.is_expired() {
                let base_url = std::env::var("MINIMAX_API_HOST")
                    .unwrap_or_else(|_| "https://api.minimax.io/v1".into());
                let model = std::env::var("MINIMAX_MODEL").unwrap_or_else(|_| "MiniMax-M3".into());
                tracing::info!(
                    "using MiniMax OAuth token (base_url={}, model={}, expires_at={})",
                    base_url,
                    model,
                    stored
                        .token
                        .expires_at
                        .map(|e| e.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                        .unwrap_or_else(|| "unknown".into())
                );
                let inner: Arc<dyn LLMClient> = Arc::new(
                    myharness_llm::OpenAiCompatProvider::new(
                        &base_url,
                        &stored.token.access_token,
                        &model,
                        ProviderId::Minimax,
                    )
                    .expect("failed to init MiniMax OpenAI-compat client"),
                );

                // W15.b: 401 자동 refresh + 1회 retry 를 위해 RefreshingLlmClient 로 wrap.
                // store 가 비어있거나 ensure_fresh 실패 시 graceful fallback (W15.a WARN 유지).
                if let Ok(auth) = AuthManager::new() {
                    if let Some(provider) = find_provider("minimax") {
                        return Arc::new(RefreshingLlmClient::new(
                            inner,
                            "minimax",
                            &base_url,
                            &model,
                            Arc::new(auth),
                            Arc::new(store),
                            provider,
                        ));
                    }
                }
                // wrap 실패 (provider 못 찾음 / auth init 실패) → inner 그대로 반환
                // (W15.a 의 동작과 호환, refresh 없이 호출)
                return inner;
            }
            tracing::warn!(
                "OAuth token for minimax is expired; falling back to env var. \
                 run `myharness auth minimax login` to refresh."
            );
        }
    }

    // 2) MINIMAX_API_KEY env var.
    if let Ok(api_key) = std::env::var("MINIMAX_API_KEY") {
        let base_url = std::env::var("MINIMAX_API_HOST")
            .unwrap_or_else(|_| "https://api.minimax.io/v1".into());
        let model = std::env::var("MINIMAX_MODEL").unwrap_or_else(|_| "MiniMax-M3".into());
        tracing::info!("using MiniMax env var (base_url={}, model={})", base_url, model);
        return Arc::new(
            myharness_llm::OpenAiCompatProvider::new(
                &base_url,
                &api_key,
                &model,
                ProviderId::Minimax,
            )
            .expect("failed to init MiniMax OpenAI-compat client"),
        );
    }

    // 3) ANTHROPIC_API_KEY env var.
    if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
        let model = std::env::var("ANTHROPIC_MODEL").unwrap_or_else(|_| "claude-sonnet-4-6".into());
        tracing::info!("using Anthropic env var (model={})", model);
        return Arc::new(
            myharness_llm::AnthropicProvider::new(&api_key)
                .expect("failed to init Anthropic provider"),
        );
    }

    // 4) MockClient fallback.
    tracing::warn!("no LLM credential found; falling back to MockClient");
    Arc::new(myharness_llm::client_mock::MockClient::new(
        myharness_llm::provider::ProviderId::Claude,
        "claude-sonnet-4-6",
    ))
}
