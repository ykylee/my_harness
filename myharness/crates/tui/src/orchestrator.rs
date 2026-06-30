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
    /// - A-min (2026-06-30) **text-based tool dispatch loop** — LLM 응답에서
    ///   ```tool_call ... ``` block parse → ToolRegistry::get → execute → 결과 message 추가
    ///   → LLM 재호출. 최대 3 round. tool call 없으면 즉시 final 응답.
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

        // llm enhance: if llm_client available, send (system + user) and append.
        // A-min tool dispatch loop: LLM 응답에 tool_call block 있으면 실행 + 재호출.
        if let Some(llm) = &self.llm_client {
            let system_prompt = format!(
                "{}{}",
                agent.def().system_prompt,
                crate::agent::tool_spec_section(agent.def().allowed_tools)
            );
            let mut messages: Vec<myharness_llm::client::Message> =
                vec![myharness_llm::client::Message::user(decision.extracted_input.clone())];

            const MAX_TOOL_ROUNDS: usize = 3;
            for _round in 0..MAX_TOOL_ROUNDS {
                let req = myharness_llm::CompletionRequest {
                    model: String::new(),
                    system: Some(system_prompt.clone()),
                    messages: messages.clone(),
                    max_tokens: Some(2048),
                    temperature: Some(0.2),
                    stop: vec![],
                    stream: false,
                    metadata: serde_json::Value::Null,
                };

                match llm.complete(req).await {
                    Ok(r) if !r.content.is_empty() => {
                        // tool_call block 추출 시도
                        if let Some((name, args)) = extract_tool_call(&r.content) {
                            // tool dispatch
                            let tool_result_text = self
                                .dispatch_tool_call(&name, args)
                                .await
                                .unwrap_or_else(|e| {
                                    format!("[tool-error] {e}")
                                });
                            // 결과를 assistant 응답 + tool 메시지로 메시지 추가 → 다음 round
                            messages.push(myharness_llm::client::Message::assistant(
                                r.content.clone(),
                            ));
                            messages.push(myharness_llm::client::Message::tool(
                                tool_result_text,
                                name.clone(),
                            ));
                            // 중간 응답을 누적 (디버그 가시화)
                            response.push_str(&format!(
                                "\n\n[tool_call] {} → ok\n[LLM-round-{}] dispatching next",
                                name,
                                _round + 1
                            ));
                            continue;
                        }
                        // tool_call 없으면 final 응답
                        response.push_str("\n\n[LLM] ");
                        response.push_str(&r.content);
                        break;
                    }
                    Ok(_) => {
                        // 빈 응답 — loop 중단
                        break;
                    }
                    Err(e) => {
                        if self.fatal_llm_error {
                            return Err(SubAgentError::Llm(e));
                        }
                        use std::fmt::Write as _;
                        let _ = write!(response, "\n\n[LLM-error] {e}");
                        break;
                    }
                }
            }
        }

        Ok(response)
    }

    /// A-min tool dispatch — name + args 받아서 ToolRegistry 에서 찾아 실행.
    /// 결과는 string (성공/실패 모두). permission/cwd 는 `default` mode + cwd=현재.
    async fn dispatch_tool_call(
        &self,
        name: &str,
        args: serde_json::Value,
    ) -> Result<String, SubAgentError> {
        let registry = self
            .tool_registry
            .as_ref()
            .ok_or_else(|| SubAgentError::Tool("no tool registry configured".into()))?;
        let tool = registry
            .get(name)
            .ok_or_else(|| SubAgentError::Tool(format!("tool not found: {name}")))?;
        let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let ctx = myharness_tools::ToolContext::new(
            cwd,
            myharness_tools::PermissionMode::AcceptEdits, // A-min: AcceptEdits 모드 (자동 yes)
        );
        match tool.execute(&ctx, args).await {
            Ok(r) => Ok(r.output),
            Err(e) => Err(SubAgentError::Tool(e.to_string())),
        }
    }
}

/// A-min (2026-06-30) — LLM 응답에서 첫 번째 ```tool_call``` block 추출.
/// 형식: ```tool_call\n{"name": "...", "args": {...}}\n```\n
/// 발견 안되면 None.
fn extract_tool_call(content: &str) -> Option<(String, serde_json::Value)> {
    let start = content.find("```tool_call")?;
    let after = &content[start..];
    let open = after.find('{')?;
    // balanced braces 추출 (단순 카운팅 — nested object 1단까지)
    let bytes = after.as_bytes();
    let mut depth = 0usize;
    let mut end = None;
    for (i, &b) in bytes.iter().enumerate().skip(open) {
        if b == b'{' {
            depth += 1;
        } else if b == b'}' {
            depth -= 1;
            if depth == 0 {
                end = Some(i + 1);
                break;
            }
        }
    }
    let end = end?;
    let json_str = &after[open..end];
    let parsed: serde_json::Value = serde_json::from_str(json_str).ok()?;
    let name = parsed.get("name")?.as_str()?.to_string();
    let args = parsed.get("args").cloned().unwrap_or(serde_json::json!({}));
    Some((name, args))
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

    // === A-min (2026-06-30) — text-based tool dispatch tests ===

    #[test]
    fn extract_tool_call_basic() {
        let content = r#"Sure, reading the file.

```tool_call
{"name": "Read", "args": {"file_path": "foo.rs"}}
```

Then I'll analyze."#;
        let (name, args) = extract_tool_call(content).unwrap();
        assert_eq!(name, "Read");
        assert_eq!(args.get("file_path").unwrap().as_str().unwrap(), "foo.rs");
    }

    #[test]
    fn extract_tool_call_missing_returns_none() {
        assert!(extract_tool_call("just plain text response").is_none());
        assert!(extract_tool_call("```\n{\"name\": \"Read\"}\n```").is_none()); // wrong fence
    }

    #[test]
    fn extract_tool_call_nested_args() {
        let content = r#"```tool_call
{"name": "Bash", "args": {"command": "ls -la", "timeout_ms": 5000}}
```"#;
        let (name, args) = extract_tool_call(content).unwrap();
        assert_eq!(name, "Bash");
        assert_eq!(args.get("command").unwrap().as_str().unwrap(), "ls -la");
        assert_eq!(args.get("timeout_ms").unwrap().as_u64().unwrap(), 5000);
    }

    #[tokio::test]
    async fn run_with_llm_tool_call_dispatches_and_continues() {
        // 1st response: tool_call (Read foo.rs)
        // 2nd response: final answer
        let c = Arc::new(MockClient::new(ProviderId::Claude, "claude-sonnet-4-6"));
        c.push(MockResponse::Text(
            r#"```tool_call
{"name": "Read", "args": {"file_path": "Cargo.toml"}}
```"#
            .into(),
        ));
        c.push(MockResponse::Text("After reading: looks good.".into()));
        let reg = Arc::new(myharness_tools::ToolRegistry::default_tools());
        let o = Orchestrator::new()
            .with_llm(c.clone())
            .with_tools(reg);
        let out = o.run("code review").await.unwrap();
        assert!(out.contains("[tool_call] Read"));
        assert!(out.contains("After reading: looks good."));
        // 2 LLM calls
        assert_eq!(c.calls.lock().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn run_with_llm_no_tool_call_returns_immediately() {
        let c = Arc::new(MockClient::new(ProviderId::Claude, "claude-sonnet-4-6"));
        c.push(MockResponse::Text("plain answer, no tools needed".into()));
        let reg = Arc::new(myharness_tools::ToolRegistry::default_tools());
        let o = Orchestrator::new()
            .with_llm(c.clone())
            .with_tools(reg);
        let out = o.run("code review").await.unwrap();
        assert!(out.contains("plain answer"));
        assert!(!out.contains("[tool_call]"));
        assert_eq!(c.calls.lock().unwrap().len(), 1);
    }
}
