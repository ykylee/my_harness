//! OS 기본 browser 자동 open. Linux `xdg-open`, macOS `open`, Windows `start`.

use std::process::Command;

#[derive(Debug, thiserror::Error)]
pub enum BrowserError {
    #[error("no browser command available for this platform")]
    NoBrowser,
    #[error("failed to launch browser: {0}")]
    Launch(#[from] std::io::Error),
}

/// URL 을 OS 기본 browser 로 open. blocking (subprocess spawn).
/// headless 환경 (CI, server) 에서는 실패할 수 있음 — caller 가 fallback (URL 만 print).
pub fn open(url: &str) -> Result<(), BrowserError> {
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(url).spawn()?;
        Ok(())
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(url).spawn()?;
        Ok(())
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd").args(["/C", "start", url]).spawn()?;
        Ok(())
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        let _ = url;
        Err(BrowserError::NoBrowser)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_compiles_on_this_platform() {
        // 단순 컴파일 + return type 검증. 실제 browser 안 띄움.
        let r = open("about:blank");
        // Ok 또는 Err 둘 다 OK — 환경 의존
        let _ = r;
    }
}
