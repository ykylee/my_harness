//! myharness-tui — ratatui + crossterm 기반 TUI shell + sub-agent registry + orchestrator dispatch
//!
//! 모듈:
//! - [`events`]: crossterm backend + key mapping
//! - [`app`]: App state + ratatui draw logic

#![allow(clippy::items_after_statements)] // const items at end of module is clearer than scattered at top
//! - [`agent`]: `SubAgent` trait + 4 구현 (W10.2)
//! - [`orchestrator`]: 도메인 dispatch + tools/llm 통합 (W10.3)
//! - [`loop_mode`]: ralph-wiggum 패턴 (W10.4)

pub mod agent;
pub mod app;
pub mod events;
pub mod loop_mode;
pub mod orchestrator;

pub use agent::{
    SubAgent, SubAgentDef, SubAgentDomain, SubAgentError, SubAgentKind, SubAgentRegistry,
};
pub use app::{App, AppMessage, MessageRole, draw, render_to_buffer};
pub use events::{AppKey, TtyGuard};
pub use loop_mode::{LoopConfig, LoopIteration, LoopReport, LoopRunner, LoopStop};
pub use orchestrator::{DispatchDecision, DispatchKind, Orchestrator};

/// Crate 버전.
#[must_use]
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

    #[test]
    #[allow(clippy::no_effect_underscore_binding)]
    fn public_api_exports() {
        let _app: App = App::new("x", "orchestrator");
        let _k: AppKey = AppKey::Enter;
        let _msg: AppMessage = AppMessage::user("hi");
        use crate::agent::CodeReviewerAgent;
        let _a: &dyn SubAgent = &CodeReviewerAgent;
        let _o: Orchestrator = Orchestrator::new();
        let _c: LoopConfig = LoopConfig::default();
    }
}
