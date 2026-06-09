use myharness_tools::prelude::*;

#[test]
fn test_5_providers_wire_format_valid() {
    let reg = ToolSchemaRegistry::default_schemas();

    // Anthropic
    let anthropic = ProviderCompat::Anthropic.wire_format_tools(&reg);
    assert!(anthropic[0].get("input_schema").is_some());

    // OpenAI strict
    let openai = ProviderCompat::OpenAI.wire_format_tools(&reg);
    let func = &openai[0]["function"];
    assert_eq!(func["strict"], serde_json::Value::Bool(true));
    assert_eq!(
        func["parameters"]["additionalProperties"],
        serde_json::Value::Bool(false)
    );
    assert!(func["parameters"]["required"].is_array());

    // DeepSeek
    let ds = ProviderCompat::DeepSeek.wire_format_tools(&reg);
    let func = &ds[0]["function"];
    assert!(func.get("strict").is_none() || func["strict"].is_null());

    // Ollama
    let ollama = ProviderCompat::Ollama.wire_format_tools(&reg);
    let func = &ollama[0]["function"];
    assert!(func["parameters"].get("$schema").is_none());

    // LiteLlm = OpenAI strict
    let ll = ProviderCompat::LiteLlm.wire_format_tools(&reg);
    let func = &ll[0]["function"];
    assert_eq!(func["strict"], serde_json::Value::Bool(true));
}

#[test]
fn test_provider_compat_6_providers() {
    let reg = ToolSchemaRegistry::default_schemas();
    let providers = [
        ProviderCompat::Anthropic,
        ProviderCompat::OpenAI,
        ProviderCompat::DeepSeek,
        ProviderCompat::Ollama,
        ProviderCompat::LlamaCpp,
        ProviderCompat::LiteLlm,
    ];
    for p in &providers {
        let tools = p.wire_format_tools(&reg);
        assert_eq!(tools.len(), 6, "{:?} returned wrong count", p);
    }
}

#[test]
fn test_6_providers_wire_format_with_dollar_schema_handling() {
    let reg = ToolSchemaRegistry::default_schemas();
    let providers = [
        ProviderCompat::Anthropic,
        ProviderCompat::OpenAI,
        ProviderCompat::DeepSeek,
        ProviderCompat::Ollama,
        ProviderCompat::LlamaCpp,
        ProviderCompat::LiteLlm,
    ];
    for p in &providers {
        let tools = p.wire_format_tools(&reg);
        assert_eq!(tools.len(), 6);
        match p {
            ProviderCompat::OpenAI | ProviderCompat::LiteLlm => {
                for tool in &tools {
                    assert!(
                        tool["function"]["parameters"].get("$schema").is_none(),
                        "{:?} should strip $schema",
                        p
                    );
                }
            }
            ProviderCompat::DeepSeek => {
                for tool in &tools {
                    assert!(
                        tool["function"]["parameters"].get("$schema").is_some(),
                        "DeepSeek should keep $schema (draft-07)"
                    );
                }
            }
            _ => {}
        }
    }
}

#[test]
fn test_provider_response_notes_integration() {
    let notes = ProviderCompat::LlamaCpp.response_notes();
    assert!(
        notes.contains("tool"),
        "llama.cpp response_notes should mention finish_reason 'tool'"
    );
    assert!(
        ProviderCompat::Anthropic
            .response_notes()
            .contains("Anthropic")
    );
    assert!(!ProviderCompat::OpenAI.response_notes().contains("Beta"));
}
