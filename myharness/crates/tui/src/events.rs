//! tui::events — crossterm 기반 입력 + lifecycle (raw mode init/restore).

use std::io::{self, Stdout};

use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, read};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};

/// TTY lifecycle guard. drop 시 raw mode + alternate screen 자동 복원.
pub struct TtyGuard {
    stdout: Stdout,
    active: bool,
}

impl TtyGuard {
    pub fn enter() -> Result<Self> {
        let mut stdout = io::stdout();
        enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen)?;
        Ok(Self { stdout, active: true })
    }

    pub fn leave(&mut self) -> Result<()> {
        if self.active {
            execute!(self.stdout, LeaveAlternateScreen)?;
            disable_raw_mode()?;
            self.active = false;
        }
        Ok(())
    }
}

impl Drop for TtyGuard {
    fn drop(&mut self) {
        let _ = self.leave();
    }
}

/// user 가 입력한 key event 중 우리가 처리하는 것들.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppKey {
    Char(char),
    Enter,
    Backspace,
    Esc,
    Up,
    Down,
    Left,
    Right,
    Tab,
    CtrlC,
    Other,
}

impl AppKey {
    pub fn from_crossterm(event: Event) -> Option<Self> {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                ..
            }) => Some(AppKey::CtrlC),
            Event::Key(KeyEvent { kind: KeyEventKind::Press, code, .. }) => match code {
                KeyCode::Char(c) => Some(AppKey::Char(c)),
                KeyCode::Enter => Some(AppKey::Enter),
                KeyCode::Backspace => Some(AppKey::Backspace),
                KeyCode::Esc => Some(AppKey::Esc),
                KeyCode::Up => Some(AppKey::Up),
                KeyCode::Down => Some(AppKey::Down),
                KeyCode::Left => Some(AppKey::Left),
                KeyCode::Right => Some(AppKey::Right),
                KeyCode::Tab => Some(AppKey::Tab),
                _ => Some(AppKey::Other),
            },
            _ => None,
        }
    }

    /// blocking read (실제 terminal 용)
    pub fn read() -> Result<AppKey> {
        loop {
            let event = read()?;
            if let Some(k) = AppKey::from_crossterm(event) {
                return Ok(k);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ctrl_c_maps_to_ctrlc() {
        let ev = Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            ..KeyEvent::new(KeyCode::Null, KeyModifiers::NONE)
        });
        assert_eq!(AppKey::from_crossterm(ev), Some(AppKey::CtrlC));
    }

    #[test]
    fn char_key() {
        let ev = Event::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
        assert_eq!(AppKey::from_crossterm(ev), Some(AppKey::Char('a')));
    }

    #[test]
    fn enter_key() {
        let ev = Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(AppKey::from_crossterm(ev), Some(AppKey::Enter));
    }

    #[test]
    fn backspace_key() {
        let ev = Event::Key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        assert_eq!(AppKey::from_crossterm(ev), Some(AppKey::Backspace));
    }

    #[test]
    fn non_key_event_returns_none() {
        // Resize 이벤트는 무시
        let ev = Event::Resize(80, 24);
        assert_eq!(AppKey::from_crossterm(ev), None);
    }
}
