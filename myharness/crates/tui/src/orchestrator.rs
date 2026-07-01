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
    /// D-101 (2026-06-30) — A-min tool dispatch loop 최대 round. default 10.
    pub max_tool_rounds: usize,
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl Orchestrator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tool_registry: None,
            llm_client: None,
            fatal_llm_error: false,
            max_tool_rounds: 10, // D-101 (2026-06-30) — A-min follow-up, 기본 10 round
        }
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

    /// D-101 (2026-06-30) — A-min tool dispatch loop 의 max round 설정.
    #[must_use]
    pub fn with_max_tool_rounds(mut self, n: usize) -> Self {
        self.max_tool_rounds = n;
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
                    extracted_input: if extracted.is_empty() {
                        user_input.to_string()
                    } else {
                        extracted
                    },
                };
            }
        }

        // 2) domain keyword 매칭
        let code_kw = [
            "review",
            "refactor",
            "implement",
            "fix",
            "bug",
            "function",
            "class",
        ];
        let env_kw = [
            "environment",
            "env ",
            "path",
            "version",
            "rust ",
            "node ",
            "python ",
        ];
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
                vec![myharness_llm::client::Message::user(
                    decision.extracted_input.clone(),
                )];

            // D-102 (2026-06-30) — 같은 tool_call (name+canonical_args) 가 2회 이상이면 loop break + synthetic final prompt.
            let mut call_counts: std::collections::HashMap<String, u32> =
                std::collections::HashMap::new();

            // D-101 (2026-06-30) — max_tool_rounds default 10 (configurable via with_max_tool_rounds)
            for round in 0..self.max_tool_rounds {
                let req = myharness_llm::CompletionRequest {
                    model: String::new(),
                    system: Some(system_prompt.clone()),
                    messages: messages.clone(),
                    max_tokens: Some(2048),
                    temperature: Some(0.2),
                    stop: vec![],
                    stream: false,
                    metadata: serde_json::Value::Null,
                    tools: tool_specs_for(self.tool_registry.as_deref()),
                };

                match llm.complete(req).await {
                    Ok(r) if !r.content.is_empty() => {
                        // D-108 A-proper native tool calling (v1.5+): if the
                        // provider emitted structured `tool_calls`, dispatch
                        // each one before falling back to the text-based
                        // ```tool_call``` block parse. Native path is
                        // authoritative — when present, the text-based
                        // extractor is skipped.
                        if !r.tool_calls.is_empty() {
                            for tc in &r.tool_calls {
                                // D-102 — canonical signature (key 순서 무관) 로 중복 체크
                                let sig = canonical_tool_call(&tc.name, &tc.arguments);
                                let count = call_counts.entry(sig.clone()).or_insert(0);
                                *count += 1;
                                if *count >= 2 {
                                    tracing::warn!(
                                        sig = %sig,
                                        count = *count,
                                        "tool-loop-detected (native): same tool+args repeated; breaking loop"
                                    );
                                    messages.push(myharness_llm::client::Message::assistant(
                                        r.content.clone(),
                                    ));
                                    messages.push(myharness_llm::client::Message::user(
                                        "[orchestrator] You have called the same tool with the same arguments multiple times.                                          The result will not change. Please provide your final answer based on the tool results so far.                                          Do not call any more tools."
                                            .to_string(),
                                    ));
                                    let final_req = myharness_llm::CompletionRequest {
                                        model: String::new(),
                                        system: Some(system_prompt.clone()),
                                        messages: messages.clone(),
                                        max_tokens: Some(2048),
                                        temperature: Some(0.2),
                                        stop: vec![],
                                        stream: false,
                                        metadata: serde_json::Value::Null,
                                        tools: tool_specs_for(self.tool_registry.as_deref()),
                                    };
                                    match llm.complete(final_req).await {
                                        Ok(fr) if !fr.content.is_empty() => {
                                            response.push_str(&format!(
                                                "\n\n[tool-loop-detected-native] {sig} repeated {count} times, breaking loop"
                                            ));
                                            response.push_str("\n\n[LLM-final] ");
                                            response.push_str(&fr.content);
                                        }
                                        _ => {
                                            response.push_str(&format!(
                                                "\n\n[tool-loop-detected-native] {sig} repeated {count} times, no final response"
                                            ));
                                        }
                                    }
                                    return Ok(response);
                                }

                                // tool dispatch (AcceptEdits + confirm_override true → prompt skip)
                                let (tool_result_text, tool_is_error) = match self
                                    .dispatch_tool_call(&tc.name, tc.arguments.clone())
                                    .await
                                {
                                    Ok(text) => (text, false),
                                    Err(e) => (format!("[tool-error] {e}"), true),
                                };
                                messages.push(myharness_llm::client::Message::assistant(
                                    r.content.clone(),
                                ));
                                messages.push(myharness_llm::client::Message::tool(
                                    tool_result_text.clone(),
                                    tc.name.clone(),
                                ));
                                let truncated = if tool_result_text.len() > 2000 {
                                    format!(
                                        "{}…[truncated {} chars]",
                                        &tool_result_text[..2000],
                                        tool_result_text.len() - 2000
                                    )
                                } else {
                                    tool_result_text.clone()
                                };
                                response.push_str(&format!(
                                    "\n\n[tool_call-native] {} ({}) → {}\n[tool_result] {}\n[LLM-round-{}] dispatching next",
                                    tc.name,
                                    if tool_is_error { "error" } else { "ok" },
                                    if tool_is_error { "see [tool-error]" } else { "see below" },
                                    truncated,
                                    round + 1
                                ));
                            }
                            continue;
                        }
                        // tool_call block 추출 시도 (text-based fallback, A-min / D-100)
                        if let Some((name, args)) = extract_tool_call(&r.content) {
                            // D-102 — canonical signature (key 순서 무관) 로 중복 체크
                            let sig = canonical_tool_call(&name, &args);
                            let count = call_counts.entry(sig.clone()).or_insert(0);
                            *count += 1;
                            if *count >= 2 {
                                // 같은 tool + args 2회 반복 → loop break + synthetic final prompt
                                tracing::warn!(
                                    sig = %sig,
                                    count = *count,
                                    "tool-loop-detected: same tool+args repeated; breaking loop"
                                );
                                messages.push(myharness_llm::client::Message::assistant(
                                    r.content.clone(),
                                ));
                                messages.push(myharness_llm::client::Message::user(
                                    "[orchestrator] You have called the same tool with the same arguments multiple times. \
                                     The result will not change. Please provide your final answer based on the tool results so far. \
                                     Do not call any more tools."
                                        .to_string(),
                                ));
                                let final_req = myharness_llm::CompletionRequest {
                                    model: String::new(),
                                    system: Some(system_prompt.clone()),
                                    messages: messages.clone(),
                                    max_tokens: Some(2048),
                                    temperature: Some(0.2),
                                    stop: vec![],
                                    stream: false,
                                    metadata: serde_json::Value::Null,
                                    tools: tool_specs_for(self.tool_registry.as_deref()),
                                };
                                match llm.complete(final_req).await {
                                    Ok(fr) if !fr.content.is_empty() => {
                                        response.push_str(&format!(
                                            "\n\n[tool-loop-detected] {sig} repeated {count} times, breaking loop"
                                        ));
                                        response.push_str("\n\n[LLM-final] ");
                                        response.push_str(&fr.content);
                                    }
                                    _ => {
                                        response.push_str(&format!(
                                            "\n\n[tool-loop-detected] {sig} repeated {count} times, no final response"
                                        ));
                                    }
                                }
                                break;
                            }

                            // tool dispatch (AcceptEdits + confirm_override true → prompt skip)
                            let (tool_result_text, tool_is_error) =
                                match self.dispatch_tool_call(&name, args).await {
                                    Ok(text) => (text, false),
                                    Err(e) => (format!("[tool-error] {e}"), true),
                                };
                            // 결과를 assistant 응답 + tool 메시지로 메시지 추가 → 다음 round
                            messages
                                .push(myharness_llm::client::Message::assistant(r.content.clone()));
                            messages.push(myharness_llm::client::Message::tool(
                                tool_result_text.clone(),
                                name.clone(),
                            ));
                            // D-101 (2026-06-30) — tool result 를 response 에도 누적 (이전엔 "[tool_call] X → ok" 만)
                            // 너무 길면 첫 2000자만 출력 (response 가독성)
                            let truncated = if tool_result_text.len() > 2000 {
                                format!(
                                    "{}…[truncated {} chars]",
                                    &tool_result_text[..2000],
                                    tool_result_text.len() - 2000
                                )
                            } else {
                                tool_result_text.clone()
                            };
                            response.push_str(&format!(
                                "\n\n[tool_call] {} ({}) → {}\n[tool_result] {}\n[LLM-round-{}] dispatching next",
                                name,
                                if tool_is_error { "error" } else { "ok" },
                                if tool_is_error { "see [tool-error]" } else { "see below" },
                                truncated,
                                round + 1
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
    /// 결과는 (text, is_error). D-101 (2026-06-30) follow-up:
    /// - PermissionMode::AcceptEdits (sub-agent 가 file edit 자동 yes)
    /// - confirm_override=true (Bash prompt skip, 비대화형 환경 hang 방지)
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
        let ctx =
            myharness_tools::ToolContext::new(cwd, myharness_tools::PermissionMode::AcceptEdits)
                .with_confirm_override(true); // D-101: 비대화형 hang 방지
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

/// D-108 (2026-07-01) — collect native `ToolSpec`s from a
/// `ToolRegistry`. v1.5 shipped a name-only spec because the `Tool`
/// trait did not yet expose `description` / `input_schema` (those land
/// in D-108 follow-up). D-109 (2026-07-01) now pulls both from each
/// `Tool` impl so the OpenAI-compat wire format (D-108 follow-up) can
/// emit real `function.description` + `function.parameters` fields.
/// Empty registry → empty vec.
fn tool_specs_for(
    registry: Option<&myharness_tools::ToolRegistry>,
) -> Vec<myharness_llm::client::ToolSpec> {
    let Some(reg) = registry else {
        return Vec::new();
    };
    reg.names()
        .into_iter()
        .map(|n| {
            // D-109: prefer the live `Tool` impl's declared description
            // and JSON Schema. Fall back to the name-only spec if the
            // tool is somehow missing from the registry.
            match reg.get(&n) {
                Some(tool) => myharness_llm::client::ToolSpec {
                    name: n,
                    description: Some(tool.description().to_string()),
                    input_schema: tool.input_schema(),
                },
                None => myharness_llm::client::ToolSpec::new(n, ""),
            }
        })
        .collect()
}

/// D-102 (2026-06-30) — tool_call canonical signature.
/// key 순서 무관 (BTreeMap 정렬) 으로 중복 체크 가능.
fn canonical_tool_call(name: &str, args: &serde_json::Value) -> String {
    let canonical = canonicalize_json(args.clone());
    let args_str = serde_json::to_string(&canonical).unwrap_or_default();
    format!("{name}|{args_str}")
}

/// D-102 — JSON value 의 object key 를 재귀적으로 BTreeMap 으로 정렬 (canonical form).
fn canonicalize_json(v: serde_json::Value) -> serde_json::Value {
    match v {
        serde_json::Value::Object(map) => {
            let mut sorted: std::collections::BTreeMap<String, serde_json::Value> =
                std::collections::BTreeMap::new();
            for (k, val) in map {
                sorted.insert(k, canonicalize_json(val));
            }
            serde_json::Value::Object(
                sorted
                    .into_iter()
                    .collect::<serde_json::Map<String, serde_json::Value>>(),
            )
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(canonicalize_json).collect())
        }
        other => other,
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
        assert_eq!(
            ("Code Review foo").strip_prefix_ci("code "),
            Some("Review foo")
        );
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
        let o = Orchestrator::new().with_llm(c.clone()).with_tools(reg);
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
        let o = Orchestrator::new().with_llm(c.clone()).with_tools(reg);
        let out = o.run("code review").await.unwrap();
        assert!(out.contains("plain answer"));
        assert!(!out.contains("[tool_call]"));
        assert_eq!(c.calls.lock().unwrap().len(), 1);
    }

    // === D-101 (2026-06-30) — A-min follow-up polish tests ===

    #[test]
    fn default_max_tool_rounds_is_ten() {
        let o = Orchestrator::new();
        assert_eq!(o.max_tool_rounds, 10);
    }

    #[test]
    fn with_max_tool_rounds_overrides() {
        let o = Orchestrator::new().with_max_tool_rounds(5);
        assert_eq!(o.max_tool_rounds, 5);
    }

    #[tokio::test]
    async fn run_with_tool_call_appends_tool_result_in_response() {
        // D-101 — tool result stdout 이 response 에 visible 해야 함.
        // 1st response: tool_call Bash "echo hello"
        // 2nd response: final answer
        let c = Arc::new(MockClient::new(ProviderId::Claude, "claude-sonnet-4-6"));
        c.push(MockResponse::Text(
            r#"```tool_call
{"name": "Bash", "args": {"command": "echo hello-d101"}}
```"#
                .into(),
        ));
        c.push(MockResponse::Text("done.".into()));
        let reg = Arc::new(myharness_tools::ToolRegistry::default_tools());
        let o = Orchestrator::new().with_llm(c.clone()).with_tools(reg);
        let out = o.run("env diagnose").await.unwrap();
        assert!(out.contains("[tool_call] Bash"));
        assert!(out.contains("[tool_result]"));
        assert!(
            out.contains("hello-d101"),
            "tool stdout visible in response: {out}"
        );
        assert!(out.contains("done."));
        assert_eq!(c.calls.lock().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn run_with_max_tool_rounds_two_stops_after_two() {
        // LLM 이 매번 tool_call 만 emit → max_tool_rounds=2 에서 2 round 후 break
        let c = Arc::new(MockClient::new(ProviderId::Claude, "claude-sonnet-4-6"));
        c.push(MockResponse::Text(
            r#"```tool_call
{"name": "Bash", "args": {"command": "echo r1"}}
```"#
                .into(),
        ));
        c.push(MockResponse::Text(
            r#"```tool_call
{"name": "Bash", "args": {"command": "echo r2"}}
```"#
                .into(),
        ));
        // 3rd 응답은 안 옴 (max round 초과로 dispatch 중단)
        let reg = Arc::new(myharness_tools::ToolRegistry::default_tools());
        let o = Orchestrator::new()
            .with_llm(c.clone())
            .with_tools(reg)
            .with_max_tool_rounds(2);
        let out = o.run("env diagnose").await.unwrap();
        assert!(out.contains("r1"));
        assert!(out.contains("r2"));
        assert_eq!(c.calls.lock().unwrap().len(), 2); // 2 round 후 break, 3번째 호출 안 함
    }

    // === D-102 (2026-06-30) — LLM 무한 루프 방지 (canonical dedup + synthetic final prompt) ===

    #[test]
    fn canonical_tool_call_same_args_same_sig() {
        let args1 = serde_json::json!({"command": "ls", "timeout_ms": 1000});
        let args2 = serde_json::json!({"command": "ls", "timeout_ms": 1000});
        assert_eq!(
            canonical_tool_call("Bash", &args1),
            canonical_tool_call("Bash", &args2)
        );
    }

    #[test]
    fn canonical_tool_call_key_order_doesnt_matter() {
        let args1 = serde_json::json!({"command": "ls", "timeout_ms": 1000});
        let args2 = serde_json::json!({"timeout_ms": 1000, "command": "ls"});
        assert_eq!(
            canonical_tool_call("Bash", &args1),
            canonical_tool_call("Bash", &args2)
        );
    }

    #[test]
    fn canonical_tool_call_different_args_different_sig() {
        let args1 = serde_json::json!({"command": "ls"});
        let args2 = serde_json::json!({"command": "pwd"});
        assert_ne!(
            canonical_tool_call("Bash", &args1),
            canonical_tool_call("Bash", &args2)
        );
    }

    #[tokio::test]
    async fn run_with_repeated_tool_call_breaks_loop_at_2nd() {
        // D-102 — 같은 tool + 같은 args 가 2회 반복 시 loop break + synthetic final prompt
        // LLM 응답 시퀀스:
        //   1st: tool_call Bash echo dup (dispatch)
        //   2nd: tool_call Bash echo dup (canonical sig 일치 → break + synthetic final prompt)
        //   3rd: synthetic final prompt 응답 ("final answer")
        let c = Arc::new(MockClient::new(ProviderId::Claude, "claude-sonnet-4-6"));
        c.push(MockResponse::Text(
            r#"```tool_call
{"name": "Bash", "args": {"command": "echo dup"}}
```"#
                .into(),
        ));
        c.push(MockResponse::Text(
            r#"```tool_call
{"name": "Bash", "args": {"command": "echo dup"}}
```"#
                .into(),
        ));
        c.push(MockResponse::Text("FINAL: enough info, done.".into()));
        let reg = Arc::new(myharness_tools::ToolRegistry::default_tools());
        let o = Orchestrator::new().with_llm(c.clone()).with_tools(reg);
        let out = o.run("env diagnose").await.unwrap();
        assert!(out.contains("[tool-loop-detected]"));
        assert!(out.contains("repeated"));
        assert!(out.contains("FINAL: enough info, done."));
        assert_eq!(c.calls.lock().unwrap().len(), 3); // 2 tool rounds + 1 final prompt
    }

    #[tokio::test]
    async fn run_with_repeated_tool_call_different_args_continues() {
        // canonical sig 가 다르면 (args 가 다름) 계속 진행
        let c = Arc::new(MockClient::new(ProviderId::Claude, "claude-sonnet-4-6"));
        c.push(MockResponse::Text(
            r#"```tool_call
{"name": "Bash", "args": {"command": "echo r1"}}
```"#
                .into(),
        ));
        c.push(MockResponse::Text(
            r#"```tool_call
{"name": "Bash", "args": {"command": "echo r2"}}
```"#
                .into(),
        ));
        c.push(MockResponse::Text("done.".into()));
        let reg = Arc::new(myharness_tools::ToolRegistry::default_tools());
        let o = Orchestrator::new().with_llm(c.clone()).with_tools(reg);
        let out = o.run("env diagnose").await.unwrap();
        assert!(!out.contains("[tool-loop-detected]"));
        assert!(out.contains("done."));
        assert_eq!(c.calls.lock().unwrap().len(), 3); // 2 tool rounds + 1 final (no tool_call)
    }

    // --- A-proper native tool calling (D-108) ---------------------------------

    fn native_call(
        id: &str,
        name: &str,
        args: serde_json::Value,
    ) -> myharness_llm::client::ToolCall {
        myharness_llm::client::ToolCall {
            id: id.into(),
            name: name.into(),
            arguments: args,
        }
    }

    #[tokio::test]
    async fn run_with_native_tool_call_dispatches_and_continues() {
        // LLM emits a structured `tool_calls` (not a text block).
        // The orchestrator must dispatch it and continue to the next
        // round, then accept the final text response.
        let c = Arc::new(MockClient::new(ProviderId::Claude, "claude-sonnet-4-6"));
        c.push(MockResponse::ToolCalls {
            text_before: "I will check the env.".into(),
            calls: vec![native_call(
                "call_1",
                "Bash",
                serde_json::json!({"command": "echo r1"}),
            )],
        });
        c.push(MockResponse::Text("all done.".into()));
        let reg = Arc::new(myharness_tools::ToolRegistry::default_tools());
        let o = Orchestrator::new().with_llm(c.clone()).with_tools(reg);
        let out = o.run("env diagnose").await.unwrap();
        assert!(out.contains("[tool_call-native] Bash (ok)"), "out: {out}");
        assert!(out.contains("[LLM] all done."), "out: {out}");
        assert_eq!(c.calls.lock().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn run_with_native_tool_call_request_includes_tools() {
        // The completion request sent to the LLM should carry the
        // registered tool names as `ToolSpec`s.
        let c = Arc::new(MockClient::new(ProviderId::Claude, "claude-sonnet-4-6"));
        c.push(MockResponse::Text("ok".into()));
        let reg = Arc::new(myharness_tools::ToolRegistry::default_tools());
        let o = Orchestrator::new().with_llm(c.clone()).with_tools(reg);
        let _ = o.run("hi").await.unwrap();
        let calls = c.calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        let names: Vec<&str> = calls[0].tools.iter().map(|t| t.name.as_str()).collect();
        for required in ["Bash", "Edit", "Glob", "Grep", "Read", "Write"] {
            assert!(
                names.contains(&required),
                "missing tool {required} in {names:?}"
            );
        }
    }

    #[tokio::test]
    async fn run_with_native_tool_calls_takes_precedence_over_text() {
        // If the LLM emits BOTH text containing a ```tool_call``` block
        // AND structured `tool_calls`, native wins (text-based extractor
        // is skipped). We verify by counting dispatch markers.
        let c = Arc::new(MockClient::new(ProviderId::Claude, "claude-sonnet-4-6"));
        c.push(MockResponse::ToolCalls {
            text_before: r#"```tool_call
{"name": "Bash", "args": {"command": "echo text"}}
```"#
                .into(),
            calls: vec![native_call(
                "call_native",
                "Bash",
                serde_json::json!({"command": "echo native"}),
            )],
        });
        c.push(MockResponse::Text("ok".into()));
        let reg = Arc::new(myharness_tools::ToolRegistry::default_tools());
        let o = Orchestrator::new().with_llm(c.clone()).with_tools(reg);
        let out = o.run("hi").await.unwrap();
        // Exactly one dispatch — the native one, not the text one.
        let native_count = out.matches("[tool_call-native]").count();
        let text_count = out.matches("[tool_call] Bash (ok)").count();
        assert_eq!(native_count, 1, "expected 1 native dispatch, out: {out}");
        assert_eq!(
            text_count, 0,
            "expected 0 text-based dispatches, out: {out}"
        );
    }

    #[tokio::test]
    async fn run_with_native_repeated_tool_call_breaks_loop() {
        // D-102 dedup applies to native path too: same tool+args twice
        // → synthetic final prompt + break, no infinite loop.
        let c = Arc::new(MockClient::new(ProviderId::Claude, "claude-sonnet-4-6"));
        let args = serde_json::json!({"command": "echo same"});
        c.push(MockResponse::ToolCalls {
            text_before: "first".into(),
            calls: vec![native_call("c1", "Bash", args.clone())],
        });
        c.push(MockResponse::ToolCalls {
            text_before: "second".into(),
            calls: vec![native_call("c2", "Bash", args.clone())],
        });
        c.push(MockResponse::Text("final answer".into()));
        let reg = Arc::new(myharness_tools::ToolRegistry::default_tools());
        let o = Orchestrator::new()
            .with_llm(c.clone())
            .with_tools(reg)
            .with_max_tool_rounds(5);
        let out = o.run("hi").await.unwrap();
        assert!(out.contains("[tool-loop-detected-native]"), "out: {out}");
        assert!(out.contains("final answer"), "out: {out}");
    }

    #[test]
    fn tool_specs_for_empty_registry() {
        assert!(tool_specs_for(None).is_empty());
    }

    #[test]
    fn tool_specs_for_default_registry() {
        let reg = myharness_tools::ToolRegistry::default_tools();
        let specs = tool_specs_for(Some(&reg));
        let names: Vec<&str> = specs.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"Bash"));
        assert!(names.contains(&"Edit"));
        assert_eq!(specs.len(), 6);
    }

    // D-109 (2026-07-01) — the helper must surface per-tool
    // description + input_schema so the OpenAI-compat wire format
    // (D-108 follow-up) can emit real `function.description` and
    // `function.parameters` fields.
    #[test]
    fn d109_tool_specs_carry_description_and_schema() {
        let reg = myharness_tools::ToolRegistry::default_tools();
        let specs = tool_specs_for(Some(&reg));
        let read = specs
            .iter()
            .find(|s| s.name == "Read")
            .expect("Read spec present");
        let desc = read.description.as_deref().unwrap_or("");
        assert!(!desc.is_empty(), "Read description must be non-empty");
        assert!(
            desc.contains("Read"),
            "Read description should describe the tool: {desc}"
        );
        let params = &read.input_schema;
        assert_eq!(params["type"], "object");
        let required = params["required"].as_array().unwrap();
        assert_eq!(
            required,
            &vec![serde_json::Value::String("file_path".to_string())]
        );

        let edit = specs
            .iter()
            .find(|s| s.name == "Edit")
            .expect("Edit spec present");
        let edit_props = edit.input_schema["properties"].as_object().unwrap();
        assert!(edit_props.contains_key("line_anchored"));
        assert!(edit_props.contains_key("block_anchored"));
        assert!(edit_props.contains_key("pure_edit"));
    }
    // --- D-121: SubAgent dispatch 통합 test (옵션 g-추가) ---

    /// 6개 prefix 각각이 정확한 SubAgentKind + DispatchKind::Direct 로 라우팅되는지.
    /// 회귀 시 prefix table 변경/오타 시 즉시 감지.
    #[test]
    fn d121_dispatch_prefix_each_subagent() {
        let o = Orchestrator::new();
        // code review
        let d = o.dispatch("code review this PR");
        assert_eq!(d.kind, SubAgentKind::CodeReviewer);
        assert_eq!(d.dispatch, DispatchKind::Direct);
        assert_eq!(d.extracted_input, "this PR");
        // code implement
        let d = o.dispatch("code implement foo.rs");
        assert_eq!(d.kind, SubAgentKind::CodeImplementer);
        assert_eq!(d.dispatch, DispatchKind::Direct);
        assert_eq!(d.extracted_input, "foo.rs");
        // code refactor
        let d = o.dispatch("code refactor lib.rs");
        assert_eq!(d.kind, SubAgentKind::CodeImplementer);
        assert_eq!(d.dispatch, DispatchKind::Direct);
        assert_eq!(d.extracted_input, "lib.rs");
        // env diagnose
        let d = o.dispatch("env diagnose");
        assert_eq!(d.kind, SubAgentKind::EnvDiagnose);
        assert_eq!(d.dispatch, DispatchKind::Direct);
        // prefix 만 있고 extracted 가 비면 user_input 그대로
        assert_eq!(d.extracted_input, "env diagnose");
        // git (with trailing space)
        let d = o.dispatch("git status");
        assert_eq!(d.kind, SubAgentKind::GitOperator);
        assert_eq!(d.dispatch, DispatchKind::Direct);
        assert_eq!(d.extracted_input, "status");
        // git-operator (with hyphen)
        let d = o.dispatch("git-operator help");
        assert_eq!(d.kind, SubAgentKind::GitOperator);
        assert_eq!(d.dispatch, DispatchKind::Direct);
        assert_eq!(d.extracted_input, "help");
    }

    /// prefix 매칭 실패 후 domain keyword 로 fallback 되는 path 검증.
    /// code_kw match -> CodeReviewer (kind 만 검증, not CodeImplementer).
    /// env_kw / git_kw 도 각각 정확한 도메인으로 라우팅.
    #[test]
    fn d121_dispatch_domain_keyword_fallback() {
        let o = Orchestrator::new();
        // code_kw: fix / bug / function etc
        let d = o.dispatch("please fix the bug in foo.rs");
        assert_eq!(d.kind, SubAgentKind::CodeReviewer);
        assert_eq!(d.dispatch, DispatchKind::DomainKeyword);
        assert_eq!(d.extracted_input, "please fix the bug in foo.rs");
        // env_kw: path / version
        let d = o.dispatch("check the rust version");
        assert_eq!(d.kind, SubAgentKind::EnvDiagnose);
        assert_eq!(d.dispatch, DispatchKind::DomainKeyword);
        // git_kw: commit / branch / merge
        let d = o.dispatch("create a commit for WIP");
        assert_eq!(d.kind, SubAgentKind::GitOperator);
        assert_eq!(d.dispatch, DispatchKind::DomainKeyword);
    }

    /// 어떤 prefix/keyword 도 안 잡으면 default (CodeReviewer + DispatchKind::Default).
    /// 회귀 시 default 분기 누락 / 잘못된 kind 로 빠지는 경우 감지.
    #[test]
    fn d121_dispatch_default_fallback() {
        let o = Orchestrator::new();
        let d = o.dispatch("hello world");
        assert_eq!(d.kind, SubAgentKind::CodeReviewer);
        assert_eq!(d.dispatch, DispatchKind::Default);
        assert_eq!(d.extracted_input, "hello world");
        // 특수문자 / 공백 / 비ASCII
        let d = o.dispatch("!!! ? ?");
        assert_eq!(d.kind, SubAgentKind::CodeReviewer);
        assert_eq!(d.dispatch, DispatchKind::Default);
    }

    /// `SubAgentRegistry::all()` 정확히 4개 + `for_kind` 4종 모두 Some + 도메인 분류 일치.
    /// 회귀 시 agent 추가/제거, domain 분류 변경 시 즉시 감지.
    #[test]
    fn d121_subagent_registry_4_unique() {
        use crate::agent::{SubAgentDomain, SubAgentRegistry};
        let all = SubAgentRegistry::all();
        assert_eq!(all.len(), 4, "expected exactly 4 SubAgents");
        // 4종 모두 등장
        let names: Vec<&'static str> = all.iter().map(|a| a.def().kind.as_str()).collect();
        assert!(names.contains(&"code-reviewer"));
        assert!(names.contains(&"code-implementer"));
        assert!(names.contains(&"env-diagnose"));
        assert!(names.contains(&"git-operator"));
        // for_kind 4종 모두 Some
        for kind in [
            SubAgentKind::CodeReviewer,
            SubAgentKind::CodeImplementer,
            SubAgentKind::EnvDiagnose,
            SubAgentKind::GitOperator,
        ] {
            assert!(SubAgentRegistry::for_kind(kind).is_some(), "{kind:?} missing");
        }
        // domain 분류
        let code = SubAgentRegistry::by_domain(SubAgentDomain::Code);
        assert_eq!(code.len(), 2, "Code domain = 2 (CodeReviewer + CodeImplementer)");
        let env = SubAgentRegistry::by_domain(SubAgentDomain::Environment);
        assert_eq!(env.len(), 1, "Environment domain = 1 (EnvDiagnose)");
        let util = SubAgentRegistry::by_domain(SubAgentDomain::Utility);
        assert_eq!(util.len(), 1, "Utility domain = 1 (GitOperator)");
        // by_name 도 동작
        assert!(SubAgentRegistry::by_name("code-reviewer").is_some());
        assert!(SubAgentRegistry::by_name("nonexistent").is_none());
    }
}
