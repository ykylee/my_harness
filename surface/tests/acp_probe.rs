use myharness::engine::acp::{decode_line, redact};
use serde_json::Value;

fn load(name: &str) -> Value {
    let p = format!(
        "{}/tests/fixtures/acp/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    let raw = std::fs::read_to_string(p).unwrap();
    serde_json::from_str(&raw).unwrap()
}

#[test]
fn fixtures_are_ndjson_jsonrpc() {
    for name in [
        "initialize_request.json",
        "initialize_response.json",
        "session_new_request.json",
        "session_new_response.json",
        "session_update.json",
    ] {
        let v = load(name);
        assert_eq!(v["jsonrpc"], "2.0", "{name}");
    }
}

#[test]
fn initialize_is_protocol_1() {
    let req = load("initialize_request.json");
    assert_eq!(req["method"], "initialize");
    assert_eq!(req["params"]["protocolVersion"], 1);
    assert_eq!(req["params"]["clientInfo"]["name"], "myharness");
    let res = load("initialize_response.json");
    assert_eq!(res["result"]["protocolVersion"], 1);
}

#[test]
fn session_new_has_id() {
    let req = load("session_new_request.json");
    assert_eq!(req["method"], "session/new");
    let res = load("session_new_response.json");
    assert!(res["result"]["sessionId"].as_str().is_some());
    assert_eq!(res["result"]["models"]["currentModelId"], "minimax");
}

#[test]
fn fixtures_have_no_secret_material() {
    let root = format!("{}/tests/fixtures/acp", env!("CARGO_MANIFEST_DIR"));
    for ent in std::fs::read_dir(root).unwrap() {
        let ent = ent.unwrap();
        let body = std::fs::read_to_string(ent.path()).unwrap();
        let lower = body.to_ascii_lowercase();
        assert!(!lower.contains("api_key\":"), "{}", ent.path().display());
        assert!(!body.contains("MINIMAX_API_KEY="));
        assert!(!body.contains("-api-key"));
    }
}

#[test]
fn decode_rejects_content_length_header() {
    let err = decode_line("Content-Length: 12").unwrap_err();
    assert!(!err.is_empty());
}

#[test]
fn redact_does_not_keep_api_key() {
    let v = serde_json::json!({"api_key":"secret","n":1});
    let r = redact(&v);
    assert_eq!(r["api_key"], "<redacted>");
}
