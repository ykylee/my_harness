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
                "3-도메인 하네스. /code /server /env /task /help  (S1 크롬 — 엔진 없음)",
            )],
            input: String::new(),
            session: "none".into(),
            perm: "default".into(),
            engine: "idle".into(),
            running: true,
        }
    }
}

impl App {
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
