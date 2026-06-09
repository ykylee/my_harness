use myharness_tools::prelude::*;

#[test]
fn test_anthropic_tools_array_valid() {
    let reg = ToolSchemaRegistry::default_schemas();
    let tools = reg.to_anthropic_tools();
    assert_eq!(tools.len(), 6);

    for tool in &tools {
        assert!(tool.get("name").is_some());
        assert!(tool.get("description").is_some());
        let schema = tool.get("input_schema").unwrap();
        assert_eq!(schema.get("type").and_then(|v| v.as_str()), Some("object"));
        assert!(schema.get("properties").is_some());
    }
}

#[test]
fn test_openai_tools_array_valid() {
    let reg = ToolSchemaRegistry::default_schemas();
    let tools = reg.to_openai_tools();
    assert_eq!(tools.len(), 6);

    for tool in &tools {
        assert_eq!(tool.get("type").and_then(|v| v.as_str()), Some("function"));
        let func = tool.get("function").unwrap();
        assert!(func.get("name").is_some());
        assert!(func.get("parameters").is_some());
    }
}
