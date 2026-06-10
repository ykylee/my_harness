//! SubAgent trait + 4 v1 구현 (CONCEPT §5.11).
//!
//! v1: hardcoded system prompt + allowed_tools. v1.5+: `~/.myharness/sub-agents/<name>/SYSTEM.md`.

use async_trait::async_trait;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubAgentDomain {
    Code,
    Server,
    Environment,
    Utility,
}

impl SubAgentDomain {
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

    pub fn as_str(&self) -> &'static str {
        match self {
            SubAgentKind::CodeReviewer => "code-reviewer",
            SubAgentKind::CodeImplementer => "code-implementer",
            SubAgentKind::EnvDiagnose => "env-diagnose",
            SubAgentKind::GitOperator => "git-operator",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "code-reviewer" => Some(Self::CodeReviewer),
            "code-implementer" => Some(Self::CodeImplementer),
            "env-diagnose" => Some(Self::EnvDiagnose),
            "git-operator" => Some(Self::GitOperator),
            _ => None,
        }
    }

    pub fn domain(&self) -> SubAgentDomain {
        match self {
            SubAgentKind::CodeReviewer | SubAgentKind::CodeImplementer => SubAgentDomain::Code,
            SubAgentKind::EnvDiagnose => SubAgentDomain::Environment,
            SubAgentKind::GitOperator => SubAgentDomain::Utility,
        }
    }
}

/// SubAgent 정의. v1 hardcoded.
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

/// SubAgent registry — 4개 hardcoded.
pub struct SubAgentRegistry;

impl SubAgentRegistry {
    pub fn all() -> Vec<&'static dyn SubAgent> {
        vec![
            &CodeReviewerAgent,
            &CodeImplementerAgent,
            &EnvDiagnoseAgent,
            &GitOperatorAgent,
        ]
    }

    pub fn for_kind(kind: SubAgentKind) -> Option<&'static dyn SubAgent> {
        match kind {
            SubAgentKind::CodeReviewer => Some(&CodeReviewerAgent),
            SubAgentKind::CodeImplementer => Some(&CodeImplementerAgent),
            SubAgentKind::EnvDiagnose => Some(&EnvDiagnoseAgent),
            SubAgentKind::GitOperator => Some(&GitOperatorAgent),
        }
    }

    pub fn by_domain(domain: SubAgentDomain) -> Vec<&'static dyn SubAgent> {
        Self::all().into_iter().filter(|a| a.def().domain == domain).collect()
    }

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
        let mut names: Vec<_> = SubAgentKind::ALL.iter().map(|k| k.as_str()).collect();
        names.sort();
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
}
