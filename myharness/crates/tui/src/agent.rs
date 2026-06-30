//! `SubAgent` trait + 4 v1 구현 (CONCEPT §5.11).
//!
//! v1: hardcoded system prompt + `allowed_tools`. v1.5+: `~/.myharness/sub-agents/<name>/SYSTEM.md`.

use async_trait::async_trait;

/// A-min (2026-06-30) — SubAgent 의 system prompt 끝에 자동으로 합성되는 tool spec section.
///
/// LLM 은 이 형식 그대로 ```tool_call``` block 을 emit 하면 Orchestrator 가 dispatch 한다.
/// 형식 고정 (regex parser 가 의존): ` ```tool_call\n{...}\n``` `.
///
/// D-102 (2026-06-30) follow-up — stop condition 추가: LLM 무한 루프 방지.
/// D-103 (2026-06-30) follow-up — large file chunked Read + offset overlap dedup.
pub fn tool_spec_section(allowed_tools: &[&str]) -> String {
    const ALL: &[(&str, &str)] = &[
        (
            "Read",
            "Read(file_path: string, offset?: number, limit?: number) — read file content with optional offset/limit. **For large files (>500 lines), use offset+limit to read in chunks of ~200 lines at a time** (e.g. Read(file_path, offset=0, limit=200) then Read(file_path, offset=200, limit=200) etc.). The default full-file Read can waste tokens for 1000+ line files.",
        ),
        (
            "Write",
            "Write(file_path: string, content: string) — write file content (creates parent dirs)",
        ),
        (
            "Edit",
            "Edit(file_path: string, old_string: string, new_string: string, replace_all?: bool) — exact text replace",
        ),
        (
            "Bash",
            "Bash(command: string, timeout_ms?: number) — execute shell command, returns stdout+stderr",
        ),
        (
            "Grep",
            "Grep(pattern: string, path?: string, include?: string, case_insensitive?: bool) — search text",
        ),
        (
            "Glob",
            "Glob(pattern: string, path?: string) — match file paths against glob pattern",
        ),
    ];

    let mut out = String::new();
    out.push_str("\n\n## Tool use (text-based, A-min 2026-06-30)\n\n");
    out.push_str("You can call the tools above by emitting a ```tool_call``` block in your response:\n\n");
    out.push_str("```tool_call\n");
    out.push_str("{\"name\": \"Read\", \"args\": {\"file_path\": \"path/to/file.rs\"}}\n");
    out.push_str("```\n\n");
    out.push_str("The Orchestrator will run the tool, append the result to the conversation, and let you continue. ");
    out.push_str("You may call multiple tools across turns (typically up to ~10 rounds).\n\n");
    // D-102 (2026-06-30) — stop condition: 무한 루프 방지
    out.push_str("**Stop conditions** — respond with plain markdown (no tool_call block) when ANY of:\n");
    out.push_str("1. You have enough information to fully answer the user's request.\n");
    out.push_str("2. You've already called the same tool with essentially the same arguments (the result won't change).\n");
    out.push_str("3. The last 2-3 tool calls returned similar information — don't repeat.\n");
    out.push_str("4. You're about to call a tool that the previous turn already covered.\n\n");
    out.push_str("The Orchestrator also enforces a safety net: same (name, args) repeated 2+ times will break the loop with a synthetic answer. So don't try to repeat — just respond.\n\n");
    // D-103 (2026-06-30) — large file chunked Read + overlap dedup
    out.push_str("**Large files (read strategy)**:\n");
    out.push_str("- For files >500 lines, **always** Read with `offset` and `limit` (default chunk ~200 lines).\n");
    out.push_str("- Use `Glob` first to find files, then chunked Read for large ones.\n");
    out.push_str("- Don't re-read overlapping ranges (offset=0,limit=200 then offset=50,limit=200 is wasteful).\n");
    out.push_str("- If you've read chunks 0-200 and 200-400, you don't need chunks 100-300 — progress forward instead.\n\n");
    out.push_str("Available tools in this session:\n");
    for name in allowed_tools {
        if let Some((_, desc)) = ALL.iter().find(|(n, _)| *n == *name) {
            out.push_str("- ");
            out.push_str(desc);
            out.push('\n');
        }
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubAgentDomain {
    Code,
    Server,
    Environment,
    Utility,
}

impl SubAgentDomain {
    #[must_use] 
    pub fn label(&self) -> &'static str {
        match self {
            SubAgentDomain::Code => "code",
            SubAgentDomain::Server => "server",
            SubAgentDomain::Environment => "env",
            SubAgentDomain::Utility => "util",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubAgentKind {
    CodeReviewer,
    CodeImplementer,
    EnvDiagnose,
    GitOperator,
}

impl SubAgentKind {
    pub const ALL: &'static [SubAgentKind] = &[
        SubAgentKind::CodeReviewer,
        SubAgentKind::CodeImplementer,
        SubAgentKind::EnvDiagnose,
        SubAgentKind::GitOperator,
    ];

    #[must_use] 
    pub fn as_str(&self) -> &'static str {
        match self {
            SubAgentKind::CodeReviewer => "code-reviewer",
            SubAgentKind::CodeImplementer => "code-implementer",
            SubAgentKind::EnvDiagnose => "env-diagnose",
            SubAgentKind::GitOperator => "git-operator",
        }
    }

    #[allow(clippy::should_implement_trait)]
    #[must_use] 
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "code-reviewer" => Some(Self::CodeReviewer),
            "code-implementer" => Some(Self::CodeImplementer),
            "env-diagnose" => Some(Self::EnvDiagnose),
            "git-operator" => Some(Self::GitOperator),
            _ => None,
        }
    }

    #[must_use] 
    pub fn domain(&self) -> SubAgentDomain {
        match self {
            SubAgentKind::CodeReviewer | SubAgentKind::CodeImplementer => SubAgentDomain::Code,
            SubAgentKind::EnvDiagnose => SubAgentDomain::Environment,
            SubAgentKind::GitOperator => SubAgentDomain::Utility,
        }
    }
}

/// `SubAgent` 정의. v1 hardcoded.
#[derive(Debug, Clone)]
pub struct SubAgentDef {
    pub kind: SubAgentKind,
    pub domain: SubAgentDomain,
    pub display_name: &'static str,
    pub system_prompt: &'static str,
    pub allowed_tools: &'static [&'static str],
}

#[async_trait]
pub trait SubAgent: Send + Sync {
    fn def(&self) -> &SubAgentDef;
    /// user input 받아서 응답. v1 simple: system prompt + user input 결합.
    async fn run(&self, user_input: &str) -> Result<String, SubAgentError>;
}

#[derive(Debug, thiserror::Error)]
pub enum SubAgentError {
    #[error("llm: {0}")]
    Llm(#[from] myharness_llm::LlmError),
    #[error("tool: {0}")]
    Tool(String),
    #[error("not found: {0}")]
    NotFound(String),
}

/// hardcoded sub-agent 4종.
pub struct CodeReviewerAgent;
pub struct CodeImplementerAgent;
pub struct EnvDiagnoseAgent;
pub struct GitOperatorAgent;

impl SubAgentDef {
    #[must_use] 
    pub fn for_kind(kind: SubAgentKind) -> Self {
        match kind {
            SubAgentKind::CodeReviewer => Self {
                kind,
                domain: SubAgentDomain::Code,
                display_name: "Code Reviewer",
                system_prompt: "You are a code reviewer. Analyze the provided code for bugs, style issues, and missing tests. Reply in concise markdown with sections: ## Bugs, ## Style, ## Tests.",
                allowed_tools: &["Read", "Grep", "Glob"],
            },
            SubAgentKind::CodeImplementer => Self {
                kind,
                domain: SubAgentDomain::Code,
                display_name: "Code Implementer",
                system_prompt: "You are a code implementer. Plan and apply multi-file code changes. Reply with: 1) plan summary, 2) files to change, 3) test plan.",
                allowed_tools: &["Read", "Write", "Edit", "Grep", "Glob", "Bash"],
            },
            SubAgentKind::EnvDiagnose => Self {
                kind,
                domain: SubAgentDomain::Environment,
                display_name: "Environment Diagnose",
                system_prompt: "You are an environment diagnoser. Inspect PATH, runtime versions, permissions, common config issues. Reply with: 1) detected issues, 2) suggested fixes, 3) verification commands.",
                allowed_tools: &["Bash", "Read"],
            },
            SubAgentKind::GitOperator => Self {
                kind,
                domain: SubAgentDomain::Utility,
                display_name: "Git Operator",
                system_prompt: "You are a git operator. Help with branches, commits, PRs. Reply with: 1) current state, 2) suggested commands, 3) expected outcome.",
                allowed_tools: &["Bash"],
            },
        }
    }
}

#[async_trait]
impl SubAgent for CodeReviewerAgent {
    fn def(&self) -> &SubAgentDef {
        // 매번 새로 만들지 않고 static 으로
        static DEF: std::sync::OnceLock<SubAgentDef> = std::sync::OnceLock::new();
        DEF.get_or_init(|| SubAgentDef::for_kind(SubAgentKind::CodeReviewer))
    }
    async fn run(&self, user_input: &str) -> Result<String, SubAgentError> {
        let def = self.def();
        Ok(format!(
            "[{}] Reviewing: {}\n\n(Plan) apply code review with tools={:?}",
            def.display_name, user_input, def.allowed_tools
        ))
    }
}

#[async_trait]
impl SubAgent for CodeImplementerAgent {
    fn def(&self) -> &SubAgentDef {
        static DEF: std::sync::OnceLock<SubAgentDef> = std::sync::OnceLock::new();
        DEF.get_or_init(|| SubAgentDef::for_kind(SubAgentKind::CodeImplementer))
    }
    async fn run(&self, user_input: &str) -> Result<String, SubAgentError> {
        let def = self.def();
        Ok(format!(
            "[{}] Planning: {}\n\n(Plan) implement with tools={:?}",
            def.display_name, user_input, def.allowed_tools
        ))
    }
}

#[async_trait]
impl SubAgent for EnvDiagnoseAgent {
    fn def(&self) -> &SubAgentDef {
        static DEF: std::sync::OnceLock<SubAgentDef> = std::sync::OnceLock::new();
        DEF.get_or_init(|| SubAgentDef::for_kind(SubAgentKind::EnvDiagnose))
    }
    async fn run(&self, user_input: &str) -> Result<String, SubAgentError> {
        let def = self.def();
        Ok(format!(
            "[{}] Diagnosing: {}\n\n(Plan) inspect env with tools={:?}",
            def.display_name, user_input, def.allowed_tools
        ))
    }
}

#[async_trait]
impl SubAgent for GitOperatorAgent {
    fn def(&self) -> &SubAgentDef {
        static DEF: std::sync::OnceLock<SubAgentDef> = std::sync::OnceLock::new();
        DEF.get_or_init(|| SubAgentDef::for_kind(SubAgentKind::GitOperator))
    }
    async fn run(&self, user_input: &str) -> Result<String, SubAgentError> {
        let def = self.def();
        Ok(format!(
            "[{}] Operating: {}\n\n(Plan) git ops with tools={:?}",
            def.display_name, user_input, def.allowed_tools
        ))
    }
}

/// `SubAgent` registry — 4개 hardcoded.
pub struct SubAgentRegistry;

impl SubAgentRegistry {
    #[must_use] 
    pub fn all() -> Vec<&'static dyn SubAgent> {
        vec![
            &CodeReviewerAgent,
            &CodeImplementerAgent,
            &EnvDiagnoseAgent,
            &GitOperatorAgent,
        ]
    }

    #[must_use] 
    pub fn for_kind(kind: SubAgentKind) -> Option<&'static dyn SubAgent> {
        match kind {
            SubAgentKind::CodeReviewer => Some(&CodeReviewerAgent),
            SubAgentKind::CodeImplementer => Some(&CodeImplementerAgent),
            SubAgentKind::EnvDiagnose => Some(&EnvDiagnoseAgent),
            SubAgentKind::GitOperator => Some(&GitOperatorAgent),
        }
    }

    #[must_use] 
    pub fn by_domain(domain: SubAgentDomain) -> Vec<&'static dyn SubAgent> {
        Self::all().into_iter().filter(|a| a.def().domain == domain).collect()
    }

    #[must_use] 
    pub fn by_name(name: &str) -> Option<&'static dyn SubAgent> {
        let kind = SubAgentKind::from_str(name)?;
        Self::for_kind(kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_kind_six_unique_strings() {
        let mut names: Vec<_> = SubAgentKind::ALL.iter().map(super::SubAgentKind::as_str).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 4);
    }

    #[test]
    fn kind_from_str_roundtrip() {
        for k in SubAgentKind::ALL {
            assert_eq!(SubAgentKind::from_str(k.as_str()), Some(*k));
        }
    }

    #[test]
    fn unknown_kind_returns_none() {
        assert_eq!(SubAgentKind::from_str("nonexistent"), None);
    }

    #[test]
    fn domain_classification() {
        assert_eq!(SubAgentKind::CodeReviewer.domain(), SubAgentDomain::Code);
        assert_eq!(SubAgentKind::CodeImplementer.domain(), SubAgentDomain::Code);
        assert_eq!(SubAgentKind::EnvDiagnose.domain(), SubAgentDomain::Environment);
        assert_eq!(SubAgentKind::GitOperator.domain(), SubAgentDomain::Utility);
    }

    #[test]
    fn def_has_allowed_tools() {
        let d = SubAgentDef::for_kind(SubAgentKind::CodeReviewer);
        assert!(d.allowed_tools.contains(&"Read"));
        assert!(!d.allowed_tools.contains(&"Write"));
    }

    #[test]
    fn def_system_prompt_nonempty() {
        for k in SubAgentKind::ALL {
            let d = SubAgentDef::for_kind(*k);
            assert!(!d.system_prompt.is_empty());
        }
    }

    #[tokio::test]
    async fn code_reviewer_run_returns_substantive_response() {
        let agent = CodeReviewerAgent;
        let out = agent.run("review foo.rs").await.unwrap();
        assert!(out.contains("Code Reviewer"));
        assert!(out.contains("foo.rs"));
    }

    #[tokio::test]
    async fn env_diagnose_run() {
        let agent = EnvDiagnoseAgent;
        let out = agent.run("check rust version").await.unwrap();
        assert!(out.contains("Environment"));
    }

    #[test]
    fn registry_all_returns_four() {
        assert_eq!(SubAgentRegistry::all().len(), 4);
    }

    #[test]
    fn registry_by_domain_filters() {
        let code = SubAgentRegistry::by_domain(SubAgentDomain::Code);
        assert_eq!(code.len(), 2);
        for a in &code {
            assert_eq!(a.def().domain, SubAgentDomain::Code);
        }
    }

    #[test]
    fn registry_by_name_lookup() {
        let a = SubAgentRegistry::by_name("code-reviewer").unwrap();
        assert_eq!(a.def().kind, SubAgentKind::CodeReviewer);
    }

    #[test]
    fn registry_by_name_unknown() {
        assert!(SubAgentRegistry::by_name("nonexistent").is_none());
    }

    // === D-103 (2026-06-30) — tool_spec_section large file strategy ===

    #[test]
    fn tool_spec_section_includes_large_file_strategy() {
        let spec = tool_spec_section(&["Read", "Bash"]);
        // D-102 — stop conditions
        assert!(spec.contains("Stop conditions"));
        assert!(spec.contains("enough information"));
        // D-103 — large file strategy
        assert!(spec.contains("Large files"));
        assert!(spec.contains("offset+limit") || spec.contains("offset"));
        assert!(spec.contains("chunk"));
    }

    #[test]
    fn tool_spec_section_read_description_warns_chunked() {
        let spec = tool_spec_section(&["Read"]);
        assert!(spec.contains(">500 lines") || spec.contains("large files"));
        assert!(spec.contains("~200"));
    }

    #[test]
    fn tool_spec_section_lists_only_allowed_tools() {
        let spec = tool_spec_section(&["Read", "Bash"]);
        assert!(spec.contains("- Read("));
        assert!(spec.contains("- Bash("));
        assert!(!spec.contains("- Write("));
        assert!(!spec.contains("- Edit("));
        assert!(!spec.contains("- Glob("));
    }
}
