//! myharness owned surface (D-140). S2 = 12 verbs + task. S3 TUI = ephemeral -p.

use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;
use std::process::Command;

use clap::{CommandFactory, Parser, Subcommand};

use myharness::brand::strip_chrome;
use myharness::domain::prompt_for;
use myharness::engine::detect::{ensure_version, grok_bin};
use myharness::engine::print::{PRINT_CMD_COMMENT, format_cmd, oneshot_argv};
use myharness::engine::spawn::run_turn;
use myharness::tui;
use myharness::workflow::{task_end, task_start, valid_status};

const AFTER_HELP: &str = "\
Usage:
  myharness                         # TTY 크롬 TUI (인트리)
  myharness code review <target>
  myharness code implement \"<feature>\"
  myharness code test <path>
  myharness code commit \"<message>\"
  myharness server status [host]
  myharness server logs <service> [N]
  myharness server deploy <env>
  myharness server config <action>
  myharness env setup <stack>
  myharness env install <pkgs>
  myharness env shell <cmd>
  myharness env diagnose
  myharness task start --id <ID> --title \"<title>\"
  myharness task end --id <ID> --status done|blocked [--summary s]
  myharness setup-model [--print-snippet] [--force]
  myharness engine                  # 엔진 TUI (브랜딩 노출). 기본 경로 아님

엔진은 화면 뒤에 둔다. 인자 없이 실행해도 엔진 TUI 를 띄우지 않는다.";

#[derive(Parser, Debug)]
#[command(
    name = "myharness",
    about = "3-도메인 하네스. 화면은 myharness.",
    after_help = AFTER_HELP
)]
struct Cli {
    #[arg(long)]
    print_cmd: bool,
    #[arg(long, env = "MYHARNESS_MODEL", default_value = "minimax")]
    model: String,
    #[arg(long)]
    yes: bool,
    #[arg(long)]
    plain: bool,
    #[arg(long)]
    mode: Option<String>,
    #[arg(long)]
    goal: Option<String>,
    #[command(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    Code {
        #[command(subcommand)]
        verb: CodeVerb,
    },
    Server {
        #[command(subcommand)]
        verb: ServerVerb,
    },
    Env {
        #[command(subcommand)]
        verb: EnvVerb,
    },
    Task {
        #[command(subcommand)]
        verb: TaskVerb,
    },
    SetupModel {
        #[arg(long)]
        print_snippet: bool,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        dest: Option<PathBuf>,
    },
    #[command(about = "엔진 TUI (브랜딩 노출). 기본 경로 아님")]
    Engine {
        #[command(subcommand)]
        verb: Option<EngineVerb>,
    },
}

#[derive(Subcommand, Debug)]
enum EngineVerb {
    /// Hidden handshake probe. Not advertised in --help.
    #[command(hide = true, name = "acp-probe")]
    AcpProbe {
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
enum CodeVerb {
    Review { target: Vec<String> },
    Implement { spec: Vec<String> },
    Test { path: Vec<String> },
    Commit { message: Vec<String> },
}

#[derive(Subcommand, Debug)]
enum ServerVerb {
    Status { host: Vec<String> },
    Logs { args: Vec<String> },
    Deploy { env: Vec<String> },
    Config { action: Vec<String> },
}

#[derive(Subcommand, Debug)]
enum EnvVerb {
    Setup { stack: Vec<String> },
    Install { pkgs: Vec<String> },
    Shell { cmd: Vec<String> },
    Diagnose,
}

#[derive(Subcommand, Debug)]
enum TaskVerb {
    Start {
        #[arg(long)]
        id: String,
        #[arg(long, default_value = "")]
        title: String,
    },
    End {
        #[arg(long)]
        id: String,
        #[arg(long, default_value = "done")]
        status: String,
        #[arg(long, default_value = "")]
        summary: String,
    },
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        None => {
            if io::stdin().is_terminal() && !cli.plain && surface_is_tui() {
                tui::run()
            } else {
                Cli::command().print_help()?;
                println!();
                Ok(())
            }
        }
        Some(Cmd::Engine { verb }) => match verb {
            None => run_engine(cli.print_cmd, &cli.model),
            Some(EngineVerb::AcpProbe { out }) => run_acp_probe(&cli.model, out),
        },
        Some(Cmd::Task { verb }) => run_task(verb),
        Some(Cmd::SetupModel {
            print_snippet,
            force,
            dest,
        }) => run_setup(print_snippet, force, dest),
        Some(other) => run_domain(
            cli.print_cmd,
            &cli.model,
            cli.yes,
            cli.mode.as_deref(),
            cli.goal.as_deref(),
            other,
        ),
    }
}

/// S3–S4a: cargo run / tui / ephemeral / DEBUG_EPHEMERAL=1 → chrome TUI (-p).
/// S4b will switch tui/auto to ACP; ephemeral stays -p.
fn surface_is_tui() -> bool {
    if std::env::var("MYHARNESS_DEBUG_EPHEMERAL").ok().as_deref() == Some("1") {
        return true;
    }
    match std::env::var("MYHARNESS_SURFACE").ok().as_deref() {
        Some("plain") | Some("legacy") => false,
        Some("tui") | Some("ephemeral") | Some("auto") | None => true,
        Some(_) => true,
    }
}

fn run_acp_probe(model: &str, out: Option<PathBuf>) -> io::Result<()> {
    match myharness::engine::acp::handshake(model) {
        Ok(h) => {
            let report = myharness::engine::acp::report(&h);
            let text = serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".into());
            if let Some(path) = out {
                if let Some(dir) = path.parent() {
                    std::fs::create_dir_all(dir)?;
                }
                std::fs::write(&path, format!("{text}\n"))?;
                println!("myharness: acp-probe → {}", path.display());
            } else {
                println!("{text}");
            }
            if h.protocol_version != Some(1) || h.session_id.is_none() {
                eprintln!("myharness: handshake 불완전 (protocol/session)");
                std::process::exit(1);
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("myharness: acp-probe 실패: {e}");
            std::process::exit(2);
        }
    }
}

fn run_engine(print_cmd: bool, model: &str) -> io::Result<()> {
    eprintln!("myharness: 엔진 TUI 는 벤더 브랜딩이 보입니다. 기본 경로는 인자 없는 myharness 입니다.");
    let grok = grok_bin().unwrap_or_else(|e| {
        eprintln!("myharness: {e}");
        std::process::exit(2);
    });
    if let Err(e) = ensure_version(&grok) {
        eprintln!("myharness: {e}");
        std::process::exit(2);
    }
    if print_cmd {
        println!("# inherit TTY (engine pager)");
        println!("{grok} -m {model}");
        return Ok(());
    }
    let status = Command::new(&grok).arg("-m").arg(model).status()?;
    std::process::exit(status.code().unwrap_or(1));
}

fn run_task(verb: TaskVerb) -> io::Result<()> {
    let home = dirs_home();
    match verb {
        TaskVerb::Start { id, title } => {
            let title = if title.is_empty() { id.clone() } else { title };
            let p = task_start(&home, &id, &title)?;
            println!("myharness: task start → {}", p.display());
        }
        TaskVerb::End { id, status, summary } => {
            if !valid_status(&status) {
                eprintln!("myharness: task status 는 planned|in_progress|blocked|done");
                std::process::exit(2);
            }
            let p = task_end(&home, &id, &status, &summary)?;
            println!("myharness: task end ({status}) → {}", p.display());
        }
    }
    Ok(())
}

fn run_setup(print_snippet: bool, force: bool, dest: Option<PathBuf>) -> io::Result<()> {
    let snippet = include_str!("../../plugins/myharness/examples/minimax.toml");
    if print_snippet {
        print!("{snippet}");
        return Ok(());
    }
    let dest = dest.unwrap_or_else(|| dirs_home().join(".grok/config.toml"));
    if dest.exists() {
        let body = std::fs::read_to_string(&dest)?;
        if body.contains("[model.minimax]") && !force {
            println!(
                "myharness: [model.minimax] 이미 있음 → {} (덮으려면 --force)",
                dest.display()
            );
            return Ok(());
        }
    }
    if let Some(dir) = dest.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&dest)?;
    writeln!(f, "\n# --- myharness setup-model ---\n{snippet}")?;
    println!("myharness: MiniMax 블록 추가 → {}", dest.display());
    Ok(())
}

fn run_domain(
    print_cmd: bool,
    model: &str,
    yes: bool,
    mode: Option<&str>,
    goal: Option<&str>,
    cmd: Cmd,
) -> io::Result<()> {
    let (domain, verb, rest) = match cmd {
        Cmd::Code { verb } => match verb {
            CodeVerb::Review { target } => ("code", "review", target),
            CodeVerb::Implement { spec } => ("code", "implement", spec),
            CodeVerb::Test { path } => ("code", "test", path),
            CodeVerb::Commit { message } => ("code", "commit", message),
        },
        Cmd::Server { verb } => match verb {
            ServerVerb::Status { host } => ("server", "status", host),
            ServerVerb::Logs { args } => ("server", "logs", args),
            ServerVerb::Deploy { env } => {
                if !yes && !io::stdin().is_terminal() {
                    eprintln!("myharness: server deploy 는 --yes 가 필요합니다");
                    std::process::exit(2);
                }
                ("server", "deploy", env)
            }
            ServerVerb::Config { action } => ("server", "config", action),
        },
        Cmd::Env { verb } => match verb {
            EnvVerb::Setup { stack } => ("env", "setup", stack),
            EnvVerb::Install { pkgs } => ("env", "install", pkgs),
            EnvVerb::Shell { cmd } => ("env", "shell", cmd),
            EnvVerb::Diagnose => ("env", "diagnose", vec![]),
        },
        _ => unreachable!(),
    };
    let mut prompt = prompt_for(domain, verb, &rest).expect("known verb");
    if mode == Some("loop") || goal.is_some() {
        let goal = goal.unwrap_or("반복 완료까지");
        prompt = format!("목표: {goal}. {prompt}");
    }
    let grok = grok_bin().unwrap_or_else(|e| {
        eprintln!("myharness: {e}");
        std::process::exit(2);
    });
    if let Err(e) = ensure_version(&grok) {
        eprintln!("myharness: {e}");
        std::process::exit(2);
    }
    let argv = oneshot_argv(&grok, model, &prompt);
    if print_cmd {
        println!("{PRINT_CMD_COMMENT}");
        println!("{}", format_cmd(&argv));
        return Ok(());
    }
    let out = run_turn(&argv, std::time::Duration::from_secs(300))?;
    println!("{}", strip_chrome(&out.stdout));
    if !out.stderr.is_empty() {
        let cleaned = strip_chrome(&out.stderr);
        if !cleaned.is_empty() {
            eprintln!("{cleaned}");
        }
    }
    if out.timed_out {
        eprintln!("myharness: 엔진 타임아웃");
        std::process::exit(1);
    }
    if out.code.unwrap_or(1) != 0 {
        std::process::exit(out.code.unwrap_or(1));
    }
    Ok(())
}

fn dirs_home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}
