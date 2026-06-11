//! Orchestrator — user input → 도메인 dispatch → sub-agent spawn → tools/llm 통합.
//!
//! v1 simple 의 dispatch 규칙:
//! 1) 입력 prefix 매칭: "code review" → code-reviewer, "code implement" → code-implementer,
//!    "env diagnose" → env-diagnose, "git ..." → git-operator
//! 2) 매칭 안 되면 domain keyword 매칭: "review"/"refactor" → code, "deploy"/"server" → server, etc.
//! 3) 그래도 매칭 안 되면 default: code-reviewer (코드 작업 기본값)

use std::sync::Arc;

use myharness_llm::LLMClient;
use myharness_tools::ToolRegistry;

use crate::agent::{SubAgentError, SubAgentKind, SubAgentRegistry};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DispatchKind {
    Direct,
    DomainKeyword,
    Default,
}

#[derive(Debug, Clone)]
pub struct DispatchDecision {
    pub kind: SubAgentKind,
    pub dispatch: DispatchKind,
    pub extracted_input: String,
}

pub struct Orchestrator {
    pub tool_registry: Option<Arc<ToolRegistry>>,
    pub llm_client: Option<Arc<dyn LLMClient>>,
    /// W11.3 (D-49) — LLM err 를 fatal 로 처리. false (default) 면 [LLM-error] 로 wrap 후 Ok.
    pub fatal_llm_error: bool,
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl Orchestrator {
    #[must_use]
    pub fn new() -> Self {
        Self { tool_registry: None, llm_client: None, fatal_llm_error: false }
    }

    #[must_use]
    pub fn with_tools(mut self, registry: Arc<ToolRegistry>) -> Self {
        self.tool_registry = Some(registry);
        self
    }

    #[must_use]
    pub fn with_llm(mut self, client: Arc<dyn LLMClient>) -> Self {
        self.llm_client = Some(client);
        self
    }

    /// W11.3 (D-49) — LLM err 를 fatal 로 처리. true 면 err 그대로 surface.
    #[must_use]
    pub fn with_fatal_llm_error(mut self, fatal: bool) -> Self {
        self.fatal_llm_error = fatal;
        self
    }

    /// user input → sub-agent dispatch 결정.
    #[must_use]
    pub fn dispatch(&self, user_input: &str) -> DispatchDecision {
        let lower = user_input.to_ascii_lowercase();
        // 1) prefix 매칭
        let prefix_match = [
            ("code review", SubAgentKind::CodeReviewer),
            ("code implement", SubAgentKind::CodeImplementer),
            ("code refactor", SubAgentKind::CodeImplementer),
            ("env diagnose", SubAgentKind::EnvDiagnose),
            ("git ", SubAgentKind::GitOperator),
            ("git-operator", SubAgentKind::GitOperator),
        ];
        for (prefix, kind) in prefix_match {
            if lower.starts_with(prefix) {
                let extracted = user_input
                    .strip_prefix_ci(prefix)
                    .unwrap_or(user_input)
                    .trim()
                    .to_string();
                return DispatchDecision {
                    kind,
                    dispatch: DispatchKind::Direct,
                    extracted_input: if extracted.is_empty() { user_input.to_string() } else { extracted },
                };
            }
        }

        // 2) domain keyword 매칭
        let code_kw = ["review", "refactor", "implement", "fix", "bug", "function", "class"];
        let env_kw = ["environment", "env ", "path", "version", "rust ", "node ", "python "];
        let git_kw = ["commit", "branch", "merge", "pr", "push", "pull", "rebase"];

        if code_kw.iter().any(|k| lower.contains(k)) {
            return DispatchDecision {
                kind: SubAgentKind::CodeReviewer,
                dispatch: DispatchKind::DomainKeyword,
                extracted_input: user_input.to_string(),
            };
        }
        if env_kw.iter().any(|k| lower.contains(k)) {
            return DispatchDecision {
                kind: SubAgentKind::EnvDiagnose,
                dispatch: DispatchKind::DomainKeyword,
                extracted_input: user_input.to_string(),
            };
        }
        if git_kw.iter().any(|k| lower.contains(k)) {
            return DispatchDecision {
                kind: SubAgentKind::GitOperator,
                dispatch: DispatchKind::DomainKeyword,
                extracted_input: user_input.to_string(),
            };
        }

        // 3) default
        DispatchDecision {
            kind: SubAgentKind::CodeReviewer,
            dispatch: DispatchKind::Default,
            extracted_input: user_input.to_string(),
        }
    }

    /// dispatch + `sub-agent.run()`. tools/llm 통합:
    /// - tools 가 있으면 `def.allowed_tools` 와 registry 교집합 검증 (log warn if mismatched)
    /// - `llm_client` 가 있으면 system prompt + user input 으로 LLM 호출 추가
    ///
    /// # Errors
    /// 서브 에이전트를 찾을 수 없거나 LLM 호출 중 오류 발생 시 `SubAgentError` 를 반환합니다.
    pub async fn run(&self, user_input: &str) -> Result<String, SubAgentError> {
        let decision = self.dispatch(user_input);
        let agent = SubAgentRegistry::for_kind(decision.kind)
            .ok_or_else(|| SubAgentError::NotFound(decision.kind.as_str().into()))?;

        // tools check: def.allowed_tools 가 registry 에 모두 존재하는지
        if let Some(registry) = &self.tool_registry {
            let names = registry.names();
            for tool in agent.def().allowed_tools {
                if !names.contains(&tool.to_string()) {
                    tracing::warn!(tool = %tool, "sub-agent allowed tool missing from registry");
                }
            }
        }

        // base sub-agent response
        let mut response = agent.run(&decision.extracted_input).await?;

        // llm enhance: if llm_client available, send (system + user) and append
        if let Some(llm) = &self.llm_client {
            let req = myharness_llm::CompletionRequest {
                model: String::new(),
                system: Some(agent.def().system_prompt.to_string()),
                messages: vec![myharness_llm::client::Message::user(decision.extracted_input.clone())],
                max_tokens: Some(256),
                temperature: Some(0.2),
                stop: vec![],
                stream: false,
                metadata: serde_json::Value::Null,
            };
            match llm.complete(req).await {
                Ok(r) if !r.content.is_empty() => {
                    response.push_str("\n\n[LLM] ");
                    response.push_str(&r.content);
                }
                Ok(_) => {}
                Err(e) => {
                    if self.fatal_llm_error {
                        return Err(SubAgentError::Llm(e));
                    }
                    use std::fmt::Write as _;
                    let _ = write!(response, "\n\n[LLM-error] {e}");
                }
            }
        }

        Ok(response)
    }
}

// helper extension trait for case-insensitive prefix strip
trait StripPrefixCi {
    fn strip_prefix_ci(&self, prefix: &str) -> Option<&str>;
}
impl<'a> StripPrefixCi for &'a str {
    fn strip_prefix_ci(&self, prefix: &str) -> Option<&'a str> {
        if self.len() < prefix.len() {
            return None;
        }
        if self[..prefix.len()].eq_ignore_ascii_case(prefix) {
            Some(&self[prefix.len()..])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use myharness_llm::client_mock::{MockClient, MockResponse};
    use myharness_llm::provider::ProviderId;

    #[test]
    fn dispatch_direct_code_review() {
        let o = Orchestrator::new();
        let d = o.dispatch("code review foo.rs");
        assert_eq!(d.kind, SubAgentKind::CodeReviewer);
        assert_eq!(d.dispatch, DispatchKind::Direct);
        assert_eq!(d.extracted_input, "foo.rs");
    }

    #[test]
    fn dispatch_direct_env_diagnose() {
        let o = Orchestrator::new();
        let d = o.dispatch("env diagnose rust");
        assert_eq!(d.kind, SubAgentKind::EnvDiagnose);
    }

    #[test]
    fn dispatch_direct_git() {
        let o = Orchestrator::new();
        let d = o.dispatch("git commit \"fix bug\"");
        assert_eq!(d.kind, SubAgentKind::GitOperator);
    }

    #[test]
    fn dispatch_domain_keyword_code() {
        let o = Orchestrator::new();
        let d = o.dispatch("refactor this function");
        assert_eq!(d.kind, SubAgentKind::CodeReviewer);
        assert_eq!(d.dispatch, DispatchKind::DomainKeyword);
    }

    #[test]
    fn dispatch_domain_keyword_env() {
        let o = Orchestrator::new();
        let d = o.dispatch("check rust version");
        assert_eq!(d.kind, SubAgentKind::EnvDiagnose);
    }

    #[test]
    fn dispatch_domain_keyword_git() {
        let o = Orchestrator::new();
        let d = o.dispatch("rebase feature branch");
        assert_eq!(d.kind, SubAgentKind::GitOperator);
    }

    #[test]
    fn dispatch_default_falls_back_to_code_reviewer() {
        let o = Orchestrator::new();
        let d = o.dispatch("hello world");
        assert_eq!(d.kind, SubAgentKind::CodeReviewer);
        assert_eq!(d.dispatch, DispatchKind::Default);
    }

    #[tokio::test]
    async fn run_without_llm_returns_base() {
        let o = Orchestrator::new();
        let out = o.run("code review foo.rs").await.unwrap();
        assert!(out.contains("Code Reviewer"));
        assert!(!out.contains("[LLM]"));
    }

    #[tokio::test]
    async fn run_with_llm_appends_response() {
        let c = Arc::new(MockClient::new(ProviderId::Claude, "claude-sonnet-4-6"));
        c.push(MockResponse::Text("llm says hi".into()));
        let o = Orchestrator::new().with_llm(c.clone());
        let out = o.run("code review foo.rs").await.unwrap();
        assert!(out.contains("Code Reviewer"));
        assert!(out.contains("[LLM] llm says hi"));
    }

    #[tokio::test]
    async fn run_with_llm_error_surfaces_in_response() {
        let c = Arc::new(MockClient::new(ProviderId::Claude, "claude-sonnet-4-6"));
        c.push(MockResponse::Error("401".into()));
        let o = Orchestrator::new().with_llm(c);
        let out = o.run("code review").await.unwrap();
        assert!(out.contains("[LLM-error]"));
    }

    #[tokio::test]
    async fn run_with_fatal_llm_error_returns_err() {
        let c = Arc::new(MockClient::new(ProviderId::Claude, "claude-sonnet-4-6"));
        c.push(MockResponse::Error("401".into()));
        let o = Orchestrator::new().with_llm(c).with_fatal_llm_error(true);
        let r = o.run("code review").await;
        assert!(r.is_err());
        assert!(matches!(r.unwrap_err(), SubAgentError::Llm(_)));
    }

    #[tokio::test]
    async fn run_with_tool_registry_warns_on_missing() {
        let reg = Arc::new(myharness_tools::ToolRegistry::default_tools());
        let o = Orchestrator::new().with_tools(reg);
        // 모든 sub-agent 의 allowed_tools 가 registry 에 있는지 (default 6 tool 이므로 OK)
        let out = o.run("code review foo.rs").await.unwrap();
        assert!(!out.is_empty());
    }

    #[test]
    fn strip_prefix_ci_basic() {
        assert_eq!(("Code Review foo").strip_prefix_ci("code "), Some("Review foo"));
        assert_eq!(("xyz").strip_prefix_ci("code "), None);
    }
}
