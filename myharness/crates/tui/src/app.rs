//! `tui::app` — App state + ratatui draw logic.
//!
//! 최소 TUI shell: welcome header + message list + input box.

#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss, clippy::cast_sign_loss, clippy::cast_possible_wrap)]
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Widget};
use ratatui::Frame;

use crate::events::AppKey;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
    Error,
}

#[derive(Debug, Clone)]
pub struct AppMessage {
    pub role: MessageRole,
    pub content: String,
}

impl AppMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self { role: MessageRole::User, content: content.into() }
    }
    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: MessageRole::Assistant, content: content.into() }
    }
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: MessageRole::System, content: content.into() }
    }
    pub fn tool(content: impl Into<String>) -> Self {
        Self { role: MessageRole::Tool, content: content.into() }
    }
    pub fn error(content: impl Into<String>) -> Self {
        Self { role: MessageRole::Error, content: content.into() }
    }
}

/// TUI app state.
pub struct App {
    pub title: String,
    pub mode: String,
    pub messages: Vec<AppMessage>,
    pub input: String,
    pub input_cursor: usize,
    pub running: bool,
    /// viewport height (auto-detected per frame)
    pub viewport_h: u16,
}

impl App {
    pub fn new(title: impl Into<String>, mode: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            mode: mode.into(),
            messages: vec![AppMessage::system("Welcome to myharness. Type a message and press Enter.")],
            input: String::new(),
            input_cursor: 0,
            running: true,
            viewport_h: 24,
        }
    }

    pub fn push_message(&mut self, msg: AppMessage) {
        self.messages.push(msg);
    }

    pub fn apply_key(&mut self, key: AppKey) {
        match key {
            AppKey::Char(c) => {
                self.input.insert(self.input_cursor, c);
                self.input_cursor += 1;
            }
            AppKey::Backspace => {
                if self.input_cursor > 0 {
                    self.input_cursor -= 1;
                    self.input.remove(self.input_cursor);
                }
            }
            AppKey::Enter => {
                if !self.input.is_empty() {
                    let submitted = std::mem::take(&mut self.input);
                    self.input_cursor = 0;
                    self.push_message(AppMessage::user(submitted));
                }
            }
            AppKey::Left => {
                if self.input_cursor > 0 {
                    self.input_cursor -= 1;
                }
            }
            AppKey::Right => {
                if self.input_cursor < self.input.len() {
                    self.input_cursor += 1;
                }
            }
            AppKey::CtrlC => {
                self.running = false;
            }
            AppKey::Esc | AppKey::Up | AppKey::Down | AppKey::Tab | AppKey::Other => {}
        }
    }

    #[must_use] 
    pub fn submitted_messages(&self) -> Vec<&str> {
        self.messages
            .iter()
            .filter(|m| m.role == MessageRole::User)
            .map(|m| m.content.as_str())
            .collect()
    }
}

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // header
            Constraint::Min(3),         // messages
            Constraint::Length(3),      // input
            Constraint::Length(1),      // status
        ])
        .split(area);

    // header
    let header = Paragraph::new(Line::from(vec![
        Span::styled(&app.title, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled(format!("[{}]", app.mode), Style::default().fg(Color::Yellow)),
    ]))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    // messages
    let items: Vec<ListItem> = app
        .messages
        .iter()
        .map(|m| {
            let (prefix, color) = match m.role {
                MessageRole::System => ("[sys] ", Color::DarkGray),
                MessageRole::User => ("[you] ", Color::Green),
                MessageRole::Assistant => ("[bot] ", Color::Cyan),
                MessageRole::Tool => ("[tool] ", Color::Magenta),
                MessageRole::Error => ("[err] ", Color::Red),
            };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Span::raw(m.content.as_str()),
            ]))
        })
        .collect();
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("messages"))
        .style(Style::default());
    f.render_widget(list, chunks[1]);

    // input
    let input = Paragraph::new(app.input.as_str())
        .block(Block::default().borders(Borders::ALL).title("input"));
    f.render_widget(input, chunks[2]);
    // cursor
    f.set_cursor_position((
        chunks[2].x + 1 + app.input_cursor as u16,
        chunks[2].y + 1,
    ));

    // status
    let status = Paragraph::new(format!(
        "{} msg | cursor={}/{} | Ctrl+C to quit",
        app.messages.len(),
        app.input_cursor,
        app.input.len()
    ))
    .style(Style::default().fg(Color::DarkGray));
    f.render_widget(status, chunks[3]);
}

/// render to Buffer (snapshot test 용).
pub fn render_to_buffer(area: Rect, app: &mut App) -> Buffer {
    let mut buf = Buffer::empty(area);
    // manual layout: draw widgets directly into buffer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(area);

    // header
    let header = Paragraph::new(Line::from(vec![
        Span::styled(&app.title, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled(format!("[{}]", app.mode), Style::default().fg(Color::Yellow)),
    ]))
    .block(Block::default().borders(Borders::ALL));
    header.render(chunks[0], &mut buf);

    // messages
    let items: Vec<ListItem> = app
        .messages
        .iter()
        .map(|m| {
            let (prefix, color) = match m.role {
                MessageRole::System => ("[sys] ", Color::DarkGray),
                MessageRole::User => ("[you] ", Color::Green),
                MessageRole::Assistant => ("[bot] ", Color::Cyan),
                MessageRole::Tool => ("[tool] ", Color::Magenta),
                MessageRole::Error => ("[err] ", Color::Red),
            };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Span::raw(m.content.as_str()),
            ]))
        })
        .collect();
    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("messages"));
    list.render(chunks[1], &mut buf);

    // input
    let input = Paragraph::new(app.input.as_str())
        .block(Block::default().borders(Borders::ALL).title("input"));
    input.render(chunks[2], &mut buf);

    // status
    let status = Paragraph::new(format!("{} msg", app.messages.len()))
        .style(Style::default().fg(Color::DarkGray));
    status.render(chunks[3], &mut buf);

    buf
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn new_app_has_welcome_message() {
        let app = App::new("myharness", "orchestrator");
        assert_eq!(app.messages.len(), 1);
        assert_eq!(app.messages[0].role, MessageRole::System);
        assert!(app.running);
    }

    #[test]
    fn apply_char_appends_to_input() {
        let mut app = App::new("x", "single");
        app.apply_key(AppKey::Char('h'));
        app.apply_key(AppKey::Char('i'));
        assert_eq!(app.input, "hi");
        assert_eq!(app.input_cursor, 2);
    }

    #[test]
    fn apply_backspace_removes_char() {
        let mut app = App::new("x", "single");
        app.apply_key(AppKey::Char('a'));
        app.apply_key(AppKey::Char('b'));
        app.apply_key(AppKey::Backspace);
        assert_eq!(app.input, "a");
        assert_eq!(app.input_cursor, 1);
    }

    #[test]
    fn apply_enter_pushes_user_message() {
        let mut app = App::new("x", "single");
        app.apply_key(AppKey::Char('h'));
        app.apply_key(AppKey::Char('i'));
        app.apply_key(AppKey::Enter);
        assert_eq!(app.input, "");
        assert_eq!(app.input_cursor, 0);
        assert_eq!(app.messages.len(), 2); // welcome + user
        assert_eq!(app.messages[1].role, MessageRole::User);
        assert_eq!(app.messages[1].content, "hi");
    }

    #[test]
    fn apply_ctrlc_stops_app() {
        let mut app = App::new("x", "single");
        app.apply_key(AppKey::CtrlC);
        assert!(!app.running);
    }

    #[test]
    fn left_right_cursor_movement() {
        let mut app = App::new("x", "single");
        app.apply_key(AppKey::Char('a'));
        app.apply_key(AppKey::Char('b'));
        app.apply_key(AppKey::Char('c'));
        // cursor at end (3)
        app.apply_key(AppKey::Left);
        assert_eq!(app.input_cursor, 2);
        app.apply_key(AppKey::Right);
        assert_eq!(app.input_cursor, 3);
    }

    #[test]
    fn submitted_messages_collects_user_inputs() {
        let mut app = App::new("x", "single");
        app.apply_key(AppKey::Char('h'));
        app.apply_key(AppKey::Enter);
        app.apply_key(AppKey::Char('w'));
        app.apply_key(AppKey::Enter);
        let subs = app.submitted_messages();
        assert_eq!(subs, vec!["h", "w"]);
    }

    #[test]
    fn render_to_test_backend_has_messages() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new("myharness", "orchestrator");
        terminal.draw(|f| draw(f, &mut app)).unwrap();
        let buf = terminal.backend().buffer().clone();
        // welcome 메시지가 어딘가에 그려져야 함
        let text: String = buf.content.iter().map(|c| c.symbol().chars().next().unwrap_or(' ')).collect();
        assert!(text.contains("Welcome"));
    }

    #[test]
    fn render_via_buffer_helper() {
        let mut app = App::new("myharness", "orchestrator");
        app.push_message(AppMessage::assistant("hello"));
        let buf = render_to_buffer(Rect::new(0, 0, 80, 24), &mut app);
        let text: String = buf.content.iter().map(|c| c.symbol().chars().next().unwrap_or(' ')).collect();
        assert!(text.contains("myharness"));
        assert!(text.contains("hello"));
    }

    #[test]
    fn empty_enter_does_not_push() {
        let mut app = App::new("x", "single");
        app.apply_key(AppKey::Enter);
        assert_eq!(app.messages.len(), 1);
    }
    // --- D-118: TUI shell render snapshot-style 회귀 가드 ---

    /// Buffer 의 모든 cell symbol 을 평문화하는 helper.
    fn buffer_text(buf: &Buffer) -> String {
        buf.content.iter().map(|c| c.symbol().chars().next().unwrap_or(' ')).collect()
    }

    /// 5 가지 role 의 prefix 가 모두 정상 렌더되는지 검증.
    /// 회귀 시 role -> prefix 매핑이 깨지면 즉시 감지.
    #[test]
    fn d118_render_renders_all_role_prefixes() {
        let mut app = App::new("harness", "loop");
        app.push_message(AppMessage::system("sys-info"));
        app.push_message(AppMessage::user("user-msg"));
        app.push_message(AppMessage::assistant("bot-msg"));
        app.push_message(AppMessage::tool("tool-msg"));
        app.push_message(AppMessage::error("err-msg"));
        let buf = render_to_buffer(Rect::new(0, 0, 80, 24), &mut app);
        let text = buffer_text(&buf);
        // 80 cols x 24 rows 안에서 모든 prefix 등장.
        assert!(text.contains("[sys]"),  "system prefix missing in {text:?}");
        assert!(text.contains("[you]"), "user prefix missing in {text:?}");
        assert!(text.contains("[bot]"), "assistant prefix missing in {text:?}");
        assert!(text.contains("[tool]"),"tool prefix missing in {text:?}");
        assert!(text.contains("[err]"), "error prefix missing in {text:?}");
        assert!(text.contains("sys-info"));
        assert!(text.contains("user-msg"));
        assert!(text.contains("bot-msg"));
        assert!(text.contains("tool-msg"));
        assert!(text.contains("err-msg"));
    }

    /// title + mode 헤더 line 이 buffer 상단에 모두 그려지는지 검증.
    /// 회귀 시 header layout 이 깨지면 (e.g. 색상만 바뀌고 text 사라짐) 즉시 감지.
    #[test]
    fn d118_render_header_includes_title_and_mode() {
        let mut app = App::new("myharness-tui", "orchestrator");
        let buf = render_to_buffer(Rect::new(0, 0, 80, 12), &mut app);
        let text = buffer_text(&buf);
        assert!(text.contains("myharness-tui"), "title missing in {text:?}");
        assert!(text.contains("[orchestrator]"), "mode tag missing in {text:?}");
    }

    /// 여러 메시지 push 후 status 줄 (`{} msg`) 이 정확한 카운트를 표시하는지.
    /// 회귀 시 status 위젯이 count 를 빠뜨리면 감지.
    #[test]
    fn d118_render_status_reflects_message_count() {
        let mut app = App::new("h", "single");
        // welcome 1 + 추가 4 = 5
        for i in 0..4 {
            app.push_message(AppMessage::assistant(format!("a{i}")));
        }
        let buf = render_to_buffer(Rect::new(0, 0, 80, 12), &mut app);
        let text = buffer_text(&buf);
        assert!(text.contains("5 msg"), "status should read '5 msg' in {text:?}");
        assert!(!text.contains("4 msg"));
    }

    /// `draw` (Terminal 경로) 와 `render_to_buffer` (직접 buffer 경로) 의
    /// 동일 input 에 대한 text 출력이 일치해야 함 — 두 경로 중 한쪽만 깨지는
    /// 회귀 (예: TestBackend 와 Layout 분기 차이) 를 잡는다.
    #[test]
    fn d118_draw_and_render_to_buffer_agree_on_text() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app_draw = App::new("agree", "loop");
        app_draw.push_message(AppMessage::user("hello"));
        app_draw.push_message(AppMessage::assistant("world"));
        terminal.draw(|f| draw(f, &mut app_draw)).unwrap();
        let buf_draw = terminal.backend().buffer().clone();

        let mut app_helper = App::new("agree", "loop");
        app_helper.push_message(AppMessage::user("hello"));
        app_helper.push_message(AppMessage::assistant("world"));
        let buf_helper = render_to_buffer(Rect::new(0, 0, 80, 24), &mut app_helper);

        let t_helper = buffer_text(&buf_helper);
        let t_draw = buffer_text(&buf_draw);
        for token in ["agree", "[loop]", "[you]", "hello", "[bot]", "world"] {
            assert!(t_helper.contains(token), "helper missing {token}: {t_helper:?}");
            assert!(t_draw.contains(token),    "draw missing {token}: {t_draw:?}");
        }
    }
}
