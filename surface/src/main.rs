//! myharness owned surface (D-140). S1 = chrome only, no engine spawn.

use std::io::{self, IsTerminal};

use clap::{Parser, Subcommand};

use myharness::brand;
use myharness::tui;

#[derive(Parser, Debug)]
#[command(name = "myharness", about = "3-도메인 하네스. 화면은 myharness.")]
struct Cli {
    #[command(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// 벤더 TUI (브랜딩 노출). 기본 경로 아님.
    Engine,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Some(Cmd::Engine) => {
            eprintln!(
                "myharness: 엔진 TUI 는 벤더 브랜딩이 보입니다. S1 에서는 연결하지 않습니다."
            );
            std::process::exit(2);
        }
        None => {
            if io::stdin().is_terminal() {
                tui::run()
            } else {
                println!("{}", brand::WORDMARK);
                println!("3-도메인 하네스. TTY 에서 크롬 TUI. 12 동사는 S2.");
                Ok(())
            }
        }
    }
}
