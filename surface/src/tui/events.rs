use std::io::{self, stdout};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use super::App;
use crate::brand::{OSC_TITLE, strip_chrome};
use crate::engine::detect::{ensure_version, grok_bin};
use crate::engine::print::ephemeral_argv;
use crate::engine::spawn::{TurnResult, run_turn};
use super::Role;

pub const EPHEMERAL_TIMEOUT: Duration = Duration::from_secs(60);

pub fn run() -> io::Result<()> {
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen)?;
    print!("\x1b]0;{OSC_TITLE}\x07");
    let mut terminal = Terminal::new(CrosstermBackend::new(out))?;
    let mut app = App::default();
    prime_engine(&mut app);
    let result = loop_ui(&mut terminal, &mut app);
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    print!("\x1b]0;{OSC_TITLE}\x07");
    result
}

fn prime_engine(app: &mut App) {
    match grok_bin().and_then(|b| ensure_version(&b).map(|v| (b, v))) {
        Ok((_bin, _ver)) => app.engine = "ready".into(),
        Err(e) => {
            app.engine = "error".into();
            app.push(Role::Err, e);
        }
    }
}

fn model_flag() -> String {
    std::env::var("MYHARNESS_MODEL").unwrap_or_else(|_| "minimax".into())
}

fn start_turn(app: &mut App) -> Option<Receiver<io::Result<TurnResult>>> {
    if app.engine == "busy" {
        return None;
    }
    let text = app.input.trim().to_string();
    app.input.clear();
    if text.is_empty() {
        return None;
    }
    if text == "/quit" || text == "/q" {
        app.running = false;
        return None;
    }
    let grok = match grok_bin() {
        Ok(g) => g,
        Err(e) => {
            app.engine = "error".into();
            app.push(Role::Err, e);
            return None;
        }
    };
    let prompt = app.wrap_turn(&text);
    app.push(Role::You, text);
    let argv = ephemeral_argv(&grok, &model_flag(), &prompt);
    app.engine = "busy".into();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(run_turn(&argv, EPHEMERAL_TIMEOUT));
    });
    Some(rx)
}

fn apply_turn(app: &mut App, result: io::Result<TurnResult>) {
    match result {
        Ok(turn) if turn.timed_out => {
            app.engine = "error".into();
            app.push(Role::Err, "엔진 타임아웃 (60s). YOLO 로 풀지 않음.");
        }
        Ok(turn) => {
            let text = strip_chrome(&turn.stdout);
            if !text.is_empty() {
                app.push(Role::Mh, text);
            }
            let err = strip_chrome(&turn.stderr);
            if !err.is_empty() {
                app.push(Role::Err, err);
            }
            app.engine = if turn.code.unwrap_or(1) == 0 {
                "ready".into()
            } else {
                "error".into()
            };
        }
        Err(e) => {
            app.engine = "error".into();
            app.push(Role::Err, e.to_string());
        }
    }
    print!("\x1b]0;{OSC_TITLE}\x07");
}

fn loop_ui(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    let mut pending: Option<Receiver<io::Result<TurnResult>>> = None;
    while app.running {
        if let Some(rx) = pending.as_ref() {
            match rx.try_recv() {
                Ok(result) => {
                    apply_turn(app, result);
                    pending = None;
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    app.engine = "error".into();
                    app.push(Role::Err, "엔진 스레드 종료");
                    pending = None;
                }
            }
        }
        terminal.draw(|f| app.draw(f))?;
        if !event::poll(Duration::from_millis(80))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.running = false;
            }
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.running = false;
            }
            KeyCode::Esc => app.running = false,
            KeyCode::Char(c) => app.input.push(c),
            KeyCode::Backspace => {
                app.input.pop();
            }
            KeyCode::Enter => {
                if pending.is_none() {
                    pending = start_turn(app);
                }
            }
            _ => {}
        }
    }
    Ok(())
}
