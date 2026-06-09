//! myharness-tui — ratatui + crossterm 기반 TUI primitive
//!
//! v1 MVP skeleton (TASK-005-1 W2).
//! 본 구현은 TASK-005-1 W3~W11 에서 진행.

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_set() {
        assert!(!version().is_empty());
    }
}
