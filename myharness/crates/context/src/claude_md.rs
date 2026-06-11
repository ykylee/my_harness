//! project root + parent walk 으로 CLAUDE.md 자동 발견.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredContext {
    pub path: PathBuf,
    pub source: ContextSource,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContextSource {
    /// cwd/CLAUDE.md — 최우선
    ProjectLocal,
    /// cwd/.myharness/CLAUDE.md
    ProjectDotDir,
    /// 상위 dir walk
    ProjectParent,
    /// ~/.myharness/CLAUDE.md — 글로벌 fallback
    Global,
}

impl ContextSource {
    #[must_use]
    pub fn priority(&self) -> u32 {
        match self {
            ContextSource::ProjectLocal => 0,
            ContextSource::ProjectDotDir => 1,
            ContextSource::ProjectParent => 2,
            ContextSource::Global => 3,
        }
    }

    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            ContextSource::ProjectLocal => "project-local",
            ContextSource::ProjectDotDir => "project-dotdir",
            ContextSource::ProjectParent => "project-parent",
            ContextSource::Global => "global",
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ContextLoader {
    /// 추가 검색 dir (예: workspace 내 다른 crate 의 CLAUDE.md)
    pub extra_roots: Vec<PathBuf>,
    /// 글로벌 fallback 사용 여부
    pub use_global: bool,
}

impl ContextLoader {
    #[must_use]
    pub fn new() -> Self {
        Self {
            use_global: true,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn with_extra_root(mut self, path: PathBuf) -> Self {
        self.extra_roots.push(path);
        self
    }

    #[must_use]
    pub fn without_global(mut self) -> Self {
        self.use_global = false;
        self
    }

    #[must_use]
    /// cwd 기준 우선순위대로 CLAUDE.md 발견. 발견 결과를 priority 오름차순으로 반환.
    pub fn discover(&self, cwd: &Path) -> Vec<DiscoveredContext> {
        let mut out = Vec::new();

        // 1) cwd/CLAUDE.md
        let local = cwd.join("CLAUDE.md");
        if let Some(c) = read_if_exists(&local, ContextSource::ProjectLocal) {
            out.push(c);
        }

        // 2) cwd/.myharness/CLAUDE.md
        let dotdir = cwd.join(".myharness").join("CLAUDE.md");
        if let Some(c) = read_if_exists(&dotdir, ContextSource::ProjectDotDir) {
            out.push(c);
        }

        // 3) parent walk
        if let Some(mut p) = cwd.parent() {
            loop {
                let candidate = p.join("CLAUDE.md");
                if let Some(c) = read_if_exists(&candidate, ContextSource::ProjectParent) {
                    out.push(c);
                    break;
                }
                match p.parent() {
                    Some(parent) => p = parent,
                    None => break,
                }
            }
        }

        // 4) extra roots
        for root in &self.extra_roots {
            let candidate = root.join("CLAUDE.md");
            if let Some(c) = read_if_exists(&candidate, ContextSource::ProjectParent) {
                out.push(c);
            }
        }

        // 5) global
        if self.use_global
            && let Some(home) = dirs::home_dir()
        {
            let global = home.join(".myharness").join("CLAUDE.md");
            if let Some(c) = read_if_exists(&global, ContextSource::Global) {
                out.push(c);
            }
        }

        out.sort_by_key(|c| c.source.priority());
        out
    }

    #[must_use]
    /// 발견된 contexts 를 단일 system prompt 로 합치기. 우선순위 순으로 `---\n# <label>\n<content>` 형식.
    pub fn merge_to_system_prompt(contexts: &[DiscoveredContext]) -> String {
        if contexts.is_empty() {
            return String::new();
        }
        let mut out = String::new();
        out.push_str("## Project context (CLAUDE.md files)\n\n");
        for c in contexts {
            use std::fmt::Write;
            let _ = write!(
                out,
                "### {}\n({})\n\n{}\n\n",
                c.source.label(),
                c.path.display(),
                c.content.trim_end()
            );
        }
        out
    }
}

fn read_if_exists(path: &Path, source: ContextSource) -> Option<DiscoveredContext> {
    let content = std::fs::read_to_string(path).ok()?;
    Some(DiscoveredContext {
        path: path.to_path_buf(),
        source,
        content,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_md(dir: &Path, name: &str, content: &str) -> PathBuf {
        let p = dir.join(name);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, content).unwrap();
        p
    }

    #[test]
    fn discover_finds_local_first() {
        let dir = tempfile::tempdir().unwrap();
        write_md(dir.path(), "CLAUDE.md", "local rules");
        let c = ContextLoader::new().without_global().discover(dir.path());
        assert!(!c.is_empty());
        assert_eq!(c[0].source, ContextSource::ProjectLocal);
        assert!(c[0].content.contains("local rules"));
    }

    #[test]
    fn discover_priority_ordering() {
        let parent = tempfile::tempdir().unwrap();
        let child = parent.path().join("child");
        std::fs::create_dir_all(&child).unwrap();
        write_md(&child, "CLAUDE.md", "local");
        write_md(parent.path(), "CLAUDE.md", "parent");
        let c = ContextLoader::new().without_global().discover(&child);
        assert_eq!(c.len(), 2);
        assert_eq!(c[0].source, ContextSource::ProjectLocal);
        assert_eq!(c[1].source, ContextSource::ProjectParent);
    }

    #[test]
    fn discover_skips_missing_files() {
        let dir = tempfile::tempdir().unwrap();
        let c = ContextLoader::new().without_global().discover(dir.path());
        assert!(c.is_empty());
    }

    #[test]
    fn discover_dotdir_comes_after_local() {
        let dir = tempfile::tempdir().unwrap();
        write_md(dir.path(), "CLAUDE.md", "local");
        write_md(&dir.path().join(".myharness"), "CLAUDE.md", "dotdir");
        let c = ContextLoader::new().without_global().discover(dir.path());
        assert_eq!(c.len(), 2);
        assert_eq!(c[0].source, ContextSource::ProjectLocal);
        assert_eq!(c[1].source, ContextSource::ProjectDotDir);
    }

    #[test]
    fn discover_global_fallback() {
        let dir = tempfile::tempdir().unwrap();
        // global 은 실제 home 의 ~/.myharness/CLAUDE.md — 테스트 시 부재 가능
        let c = ContextLoader::new().discover(dir.path());
        // local/parent 없으면 global 만 (있으면 더 많이)
        // global 파일이 없으면 empty
        let has_global = c.iter().any(|x| x.source == ContextSource::Global);
        if !std::path::Path::new(&dirs::home_dir().unwrap().join(".myharness").join("CLAUDE.md")).exists() {
            assert!(!has_global);
        }
    }

    #[test]
    fn discover_extra_roots() {
        let cwd = tempfile::tempdir().unwrap();
        let extra = tempfile::tempdir().unwrap();
        write_md(extra.path(), "CLAUDE.md", "from extra");
        let c = ContextLoader::new()
            .with_extra_root(extra.path().to_path_buf())
            .without_global()
            .discover(cwd.path());
        assert!(c.iter().any(|x| x.content.contains("from extra")));
    }

    #[test]
    fn merge_to_system_prompt_empty() {
        let s = ContextLoader::merge_to_system_prompt(&[]);
        assert!(s.is_empty());
    }

    #[test]
    fn merge_to_system_prompt_includes_labels() {
        let v = vec![DiscoveredContext {
            path: PathBuf::from("/tmp/CLAUDE.md"),
            source: ContextSource::ProjectLocal,
            content: "rule 1".into(),
        }];
        let s = ContextLoader::merge_to_system_prompt(&v);
        assert!(s.contains("project-local"));
        assert!(s.contains("rule 1"));
        assert!(s.contains("/tmp/CLAUDE.md"));
    }

    #[test]
    fn source_priority_ordering() {
        let mut v = vec![ContextSource::Global, ContextSource::ProjectLocal, ContextSource::ProjectParent];
        v.sort_by_key(super::ContextSource::priority);
        assert_eq!(
            v,
            vec![
                ContextSource::ProjectLocal,
                ContextSource::ProjectParent,
                ContextSource::Global,
            ]
        );
    }
}
