use std::io::{self, stdout};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use super::App;
use crate::brand::OSC_TITLE;

pub fn run() -> io::Result<()> {
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen)?;
    print!("\x1b]0;{OSC_TITLE}\x07");
    let mut terminal = Terminal::new(CrosstermBackend::new(out))?;
    let mut app = App::default();
    let result = loop_ui(&mut terminal, &mut app);
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    print!("\x1b]0;{OSC_TITLE}\x07");
    result
}

fn loop_ui(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    while app.running {
        terminal.draw(|f| app.draw(f))?;
        if !event::poll(Duration::from_millis(200))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.running = false;
            }
            KeyCode::Char('q') => app.running = false,
            KeyCode::Esc => app.running = false,
            KeyCode::Char(c) => app.input.push(c),
            KeyCode::Backspace => {
                app.input.pop();
            }
            KeyCode::Enter => {
                if app.input == "/quit" || app.input == "/q" {
                    app.running = false;
                }
                app.input.clear();
            }
            _ => {}
        }
    }
    Ok(())
}
