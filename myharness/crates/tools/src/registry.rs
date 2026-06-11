use std::collections::HashMap;
use std::sync::Arc;

use crate::bash::BashTool;
use crate::edit::EditTool;
use crate::glob_::GlobTool;
use crate::grep::GrepTool;
use crate::read::ReadTool;
use crate::tool::Tool;
use crate::write::WriteTool;

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    #[must_use] 
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    #[must_use] 
    pub fn default_tools() -> Self {
        let mut reg = Self::new();
        reg.register(Arc::new(ReadTool));
        reg.register(Arc::new(WriteTool));
        reg.register(Arc::new(EditTool));
        reg.register(Arc::new(BashTool));
        reg.register(Arc::new(GrepTool));
        reg.register(Arc::new(GlobTool));
        reg
    }

    #[must_use] 
    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    #[must_use] 
    pub fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.tools.keys().cloned().collect();
        names.sort();
        names
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_tools_have_all_names() {
        let reg = ToolRegistry::default_tools();
        let names = reg.names();
        assert!(names.contains(&"Bash".to_string()));
        assert!(names.contains(&"Edit".to_string()));
        assert!(names.contains(&"Glob".to_string()));
        assert!(names.contains(&"Grep".to_string()));
        assert!(names.contains(&"Read".to_string()));
        assert!(names.contains(&"Write".to_string()));
        assert_eq!(names.len(), 6);
    }

    #[test]
    fn test_get_returns_tool() {
        let reg = ToolRegistry::default_tools();
        let tool = reg.get("Read");
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().name(), "Read");
    }

    #[test]
    fn test_get_unknown_returns_none() {
        let reg = ToolRegistry::default_tools();
        assert!(reg.get("nonexistent").is_none());
    }
}
