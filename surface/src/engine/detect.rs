use std::process::Command;

pub const MIN_GROK: &str = "1.0.3";
pub const INSTALL_HINT: &str = "curl -fsSL https://x.ai/cli/install.sh | bash";

pub fn grok_bin() -> Result<String, String> {
    if let Ok(p) = std::env::var("MYHARNESS_GROK")
        && !p.is_empty()
    {
        return Ok(p);
    }
    which("grok").ok_or_else(|| format!("grok 이 없습니다. 설치: {INSTALL_HINT}"))
}

fn which(name: &str) -> Option<String> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let cand = dir.join(name);
        if cand.is_file() {
            return Some(cand.display().to_string());
        }
    }
    None
}

pub fn ensure_version(bin: &str) -> Result<String, String> {
    let out = Command::new(bin)
        .arg("--version")
        .output()
        .map_err(|e| format!("grok --version 실패: {e}"))?;
    let text = String::from_utf8_lossy(&out.stdout);
    let ver = text.split_whitespace().nth(1).unwrap_or("").to_string();
    if ver.is_empty() {
        return Err("grok --version 을 파싱하지 못했습니다".into());
    }
    if !version_ge(&ver, MIN_GROK) {
        return Err(format!("grok {ver} < 최소 {MIN_GROK}. 업데이트: grok update"));
    }
    Ok(ver)
}

pub fn version_ge(have: &str, need: &str) -> bool {
    let parse = |s: &str| -> Vec<u32> {
        s.split(|c: char| !c.is_ascii_digit())
            .filter(|p| !p.is_empty())
            .take(3)
            .filter_map(|p| p.parse().ok())
            .collect()
    };
    let a = parse(have);
    let b = parse(need);
    a >= b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_gate() {
        assert!(version_ge("1.0.3", MIN_GROK));
        assert!(version_ge("1.0.4", MIN_GROK));
        assert!(version_ge("1.1.0", MIN_GROK));
        assert!(version_ge("1.0.3-dev", MIN_GROK));
        assert!(!version_ge("1.0.2", MIN_GROK));
        assert!(!version_ge("1.0.0", MIN_GROK));
    }
}
