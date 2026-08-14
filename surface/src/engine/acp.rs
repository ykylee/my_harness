//! S4a: NDJSON ACP handshake against `grok agent stdio`. No product UX.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use super::detect::{ensure_version, grok_bin};
use super::spawn::kill_group;

pub const FRAMING: &str = "ndjson";
pub const PROTOCOL_VERSION: u64 = 1;
pub const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(8);

pub fn acp_argv(grok: &str, model: &str, plugin: &Path) -> Vec<String> {
    vec![
        grok.to_string(),
        "agent".into(),
        "-m".into(),
        model.into(),
        "--plugin-dir".into(),
        plugin.display().to_string(),
        "stdio".into(),
    ]
}

pub fn plugin_dir() -> Result<PathBuf, String> {
    if let Ok(p) = std::env::var("MYHARNESS_PLUGIN_DIR")
        && !p.is_empty()
    {
        return Ok(PathBuf::from(p));
    }
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo = here.join("../plugins/myharness");
    if repo.join("plugin.json").is_file() {
        return Ok(repo.canonicalize().unwrap_or(repo));
    }
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    let installed = home.join(".myharness/plugins/myharness");
    if installed.join("plugin.json").is_file() {
        return Ok(installed);
    }
    Err("plugin.json 없음. MYHARNESS_PLUGIN_DIR 또는 plugins/myharness 를 확인하세요".into())
}

pub fn encode_line(v: &Value) -> String {
    format!("{v}\n")
}

pub fn decode_line(line: &str) -> Result<Value, String> {
    serde_json::from_str(line.trim()).map_err(|e| e.to_string())
}

/// Drop secrets from any JSON we persist.
pub fn redact(v: &Value) -> Value {
    match v {
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, val) in map {
                let kl = k.to_ascii_lowercase();
                if kl.contains("key")
                    || kl.contains("token")
                    || kl.contains("secret")
                    || kl.contains("password")
                    || kl.contains("authorization")
                    || k == "args"
                    || k == "env"
                {
                    out.insert(k.clone(), json!("<redacted>"));
                } else {
                    out.insert(k.clone(), redact(val));
                }
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(items.iter().map(redact).collect()),
        other => other.clone(),
    }
}

pub struct Handshake {
    pub framing: &'static str,
    pub protocol_version: Option<u64>,
    pub session_id: Option<String>,
    pub methods_seen: Vec<String>,
    pub initialize: Value,
    pub session_new: Value,
}

pub fn handshake(model: &str) -> Result<Handshake, String> {
    let grok = grok_bin()?;
    let _ver = ensure_version(&grok)?;
    let plugin = plugin_dir()?;
    let argv = acp_argv(&grok, model, &plugin);
    run_handshake(&argv)
}

fn run_handshake(argv: &[String]) -> Result<Handshake, String> {
    let mut cmd = std::process::Command::new(&argv[0]);
    cmd.args(&argv[1..])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    let mut child = cmd.spawn().map_err(|e| format!("acp spawn: {e}"))?;
    let pid = child.id();
    let mut stdin = child.stdin.take().ok_or("acp stdin")?;
    let stdout = child.stdout.take().ok_or("acp stdout")?;

    let (tx, rx) = mpsc::channel::<Value>();
    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            let Ok(line) = line else { break };
            if let Ok(v) = decode_line(&line)
                && tx.send(v).is_err()
            {
                break;
            }
        }
    });

    let init_req = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": PROTOCOL_VERSION,
            "clientInfo": {"name": "myharness", "version": "0.1.0"},
            "capabilities": {"fs": false, "terminal": false}
        }
    });
    write_msg(&mut stdin, &init_req)?;

    let mut methods = vec!["initialize".into()];
    let mut initialize = Value::Null;
    let mut session_new = Value::Null;
    let mut session_id = None;
    let mut protocol_version = None;
    let deadline = Instant::now() + HANDSHAKE_TIMEOUT;
    let mut sent_session = false;

    while Instant::now() < deadline {
        let remain = deadline.saturating_duration_since(Instant::now());
        match rx.recv_timeout(remain.min(Duration::from_millis(250))) {
            Ok(msg) => {
                if let Some(m) = msg.get("method").and_then(|m| m.as_str())
                    && !methods.iter().any(|x| x == m)
                {
                    methods.push(m.to_string());
                }
                if msg.get("id") == Some(&json!(1)) {
                    initialize = redact(&msg);
                    protocol_version = msg
                        .pointer("/result/protocolVersion")
                        .and_then(Value::as_u64);
                    if !sent_session {
                        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                        let sess = json!({
                            "jsonrpc": "2.0",
                            "id": 2,
                            "method": "session/new",
                            "params": { "cwd": cwd, "mcpServers": [] }
                        });
                        write_msg(&mut stdin, &sess)?;
                        methods.push("session/new".into());
                        sent_session = true;
                    }
                }
                if msg.get("id") == Some(&json!(2)) {
                    session_new = redact(&msg);
                    session_id = msg
                        .pointer("/result/sessionId")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                    break;
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    drop(stdin);
    kill_group(pid);
    let _ = child.wait();

    if initialize.is_null() {
        return Err("initialize 응답 없음".into());
    }
    Ok(Handshake {
        framing: FRAMING,
        protocol_version,
        session_id,
        methods_seen: methods,
        initialize,
        session_new,
    })
}

fn write_msg(stdin: &mut impl Write, v: &Value) -> Result<(), String> {
    stdin
        .write_all(encode_line(v).as_bytes())
        .map_err(|e| format!("acp write: {e}"))?;
    stdin.flush().map_err(|e| format!("acp flush: {e}"))
}

pub fn report(h: &Handshake) -> Value {
    json!({
        "framing": h.framing,
        "protocolVersion": h.protocol_version,
        "sessionId": h.session_id,
        "methodsSeen": h.methods_seen,
        "initialize": h.initialize,
        "sessionNew": h.session_new,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn argv_is_agent_stdio_not_print() {
        let argv = acp_argv("grok", "minimax", Path::new("/tmp/p"));
        assert!(argv.windows(2).any(|w| w == ["--plugin-dir", "/tmp/p"]));
        assert!(argv.contains(&"agent".into()));
        assert!(argv.contains(&"stdio".into()));
        assert!(!argv.iter().any(|a| a == "-p"));
        assert!(!argv.iter().any(|a| a == "--always-approve"));
    }

    #[test]
    fn redact_drops_keys_and_mcp_args() {
        let raw = json!({
            "env": [{"name":"X_API_KEY","value":"secret"}],
            "args": ["-api-key","secret"],
            "api_key": "abc",
            "ok": 1
        });
        let r = redact(&raw);
        assert_eq!(r["api_key"], "<redacted>");
        assert_eq!(r["args"], "<redacted>");
        assert_eq!(r["env"], "<redacted>");
        assert_eq!(r["ok"], 1);
    }

    #[test]
    fn ndjson_roundtrip() {
        let v = json!({"jsonrpc":"2.0","id":1});
        let line = encode_line(&v);
        assert!(line.ends_with('\n'));
        assert!(!line.contains("Content-Length"));
        assert_eq!(decode_line(&line).unwrap(), v);
    }
}
