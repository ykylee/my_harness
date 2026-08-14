use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use crate::brand::{MODEL_ALIAS, WORDMARK, remap_tool};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Sys,
    You,
    Mh,
    Tool,
    Err,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub role: Role,
    pub text: String,
}

impl Message {
    pub fn sys(text: impl Into<String>) -> Self {
        Self {
            role: Role::Sys,
            text: text.into(),
        }
    }
}

pub struct App {
    pub domain: String,
    pub model_alias: String,
    pub messages: Vec<Message>,
    pub input: String,
    pub session: String,
    pub perm: String,
    pub engine: String,
    pub running: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            domain: "code".into(),
            model_alias: MODEL_ALIAS.into(),
            messages: vec![Message::sys(
                "3-도메인 하네스. 턴마다 새 한 방. 제품 아님. /quit",
            )],
            input: String::new(),
            session: "ephemeral".into(),
            perm: "plan".into(),
            engine: "idle".into(),
            running: true,
        }
    }
}

const HIST_MAX_LINES: usize = 6;
const HIST_LINE_CHARS: usize = 400;
const HIST_TOTAL_CHARS: usize = 2_000;

impl App {
    pub fn push(&mut self, role: Role, text: impl Into<String>) {
        self.messages.push(Message {
            role,
            text: text.into(),
        });
    }

    /// Recent [you]/[mh] only. N=6, ≤400/line, ≤2000 total (S3 budget).
    pub fn wrap_turn(&self, current: &str) -> String {
        let mut lines: Vec<String> = self
            .messages
            .iter()
            .filter(|m| matches!(m.role, Role::You | Role::Mh))
            .rev()
            .take(HIST_MAX_LINES)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .map(|m| {
                let tag = match m.role {
                    Role::You => "[you]",
                    Role::Mh => "[mh]",
                    _ => unreachable!(),
                };
                let mut body = m.text.clone();
                if body.chars().count() > HIST_LINE_CHARS {
                    body = body.chars().take(HIST_LINE_CHARS).collect();
                }
                format!("{tag} {body}")
            })
            .collect();
        let mut total = 0;
        lines.retain(|l| {
            if total + l.len() > HIST_TOTAL_CHARS {
                return false;
            }
            total += l.len();
            true
        });
        let hist = lines.join("\n");
        format!(
            "턴마다 새 한 방. 제품 아님. 한국어로 결론과 다음 행동만.\n{hist}\n이번: {current}"
        )
    }

    pub fn push_tool(&mut self, raw_name: &str, detail: &str) {
        self.messages.push(Message {
            role: Role::Tool,
            text: format!("{}  {detail}", remap_tool(raw_name)),
        });
    }

    pub fn draw(&self, frame: &mut Frame<'_>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(3),
                Constraint::Length(3),
                Constraint::Length(1),
            ])
            .split(frame.area());

        let header = Line::from(vec![
            Span::styled(
                format!(" {WORDMARK} "),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" ".repeat(8)),
            Span::styled(
                format!("{} · {}", self.domain, self.model_alias),
                Style::default().fg(Color::Gray),
            ),
        ]);
        frame.render_widget(
            Paragraph::new(header).block(Block::default().borders(Borders::ALL)),
            chunks[0],
        );

        let items: Vec<ListItem> = self
            .messages
            .iter()
            .map(|m| {
                let tag = match m.role {
                    Role::Sys => "[sys]",
                    Role::You => "[you]",
                    Role::Mh => "[mh]",
                    Role::Tool => "[tool]",
                    Role::Err => "[err]",
                };
                ListItem::new(format!("{tag}  {}", m.text))
            })
            .collect();
        frame.render_widget(
            List::new(items).block(Block::default().borders(Borders::ALL)),
            chunks[1],
        );

        let prompt = format!("{} › {}", self.domain, self.input);
        frame.render_widget(
            Paragraph::new(prompt).block(Block::default().borders(Borders::ALL)),
            chunks[2],
        );

        let status = format!(
            " task:none   perm:{}   session:{}   engine:{}",
            self.perm, self.session, self.engine
        );
        frame.render_widget(Paragraph::new(status), chunks[3]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_skips_tool_and_err() {
        let mut app = App::default();
        app.push(Role::You, "hi");
        app.push(Role::Tool, "secret-tool");
        app.push(Role::Err, "boom");
        app.push(Role::Mh, "ok");
        let w = app.wrap_turn("next");
        assert!(w.contains("[you] hi"));
        assert!(w.contains("[mh] ok"));
        assert!(!w.contains("secret-tool"));
        assert!(!w.contains("boom"));
        assert!(w.contains("이번: next"));
    }

    #[test]
    fn wrap_caps_line_len() {
        let mut app = App::default();
        app.push(Role::You, "x".repeat(800));
        let w = app.wrap_turn("n");
        let you = w.lines().find(|l| l.starts_with("[you]")).unwrap();
        assert!(you.chars().count() <= 6 + HIST_LINE_CHARS);
    }
}
