//! Local HTTP callback server (loopback redirect URI).
//!
//! headless 환경 (TUI/CUI only) 에서 OAuth 가 browser → callback 으로 redirect 할 때
//! 우리 쪽에서 receive. bind: localhost:port, await GET /callback?code=...&state=...

use std::net::SocketAddr;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

use crate::flow::{CallbackParams, OAuthError};

/// callback server 시작 (loopback port).
pub struct CallbackServer {
    pub local_addr: SocketAddr,
    pub redirect_path: String,
    /// single request 후 자동 shutdown
    rx: Option<oneshot::Receiver<CallbackParams>>,
}

impl CallbackServer {
    ///
    /// # Errors
    ///
    /// This function returns an error if the underlying operation fails.
    /// `127.0.0.1:port` 에 listen + path (예: "/callback") 매치.
    /// port=0 → OS 가 빈 포트 할당. `local_addr` 로 실제 port 확인.
    pub async fn start(port: u16, path: impl Into<String>) -> Result<Self, OAuthError> {
        let listener = TcpListener::bind(("127.0.0.1", port)).await?;
        let local_addr = listener.local_addr()?;
        let path = path.into();
        let path_for_task = path.clone();
        let (tx, rx) = oneshot::channel::<CallbackParams>();
        tokio::spawn(async move {
            // 단일 connection 만 받음 (OAuth flow 한 번)
            if let Ok((mut sock, _)) = listener.accept().await {
                let _ = handle_connection(&mut sock, &path_for_task, tx).await;
            }
        });
        // 즉시 return — caller 가 await rx.
        Ok(Self { local_addr, redirect_path: path, rx: Some(rx) })
    }

    ///
    /// # Errors
    ///
    /// This function returns an error if the underlying operation fails.
    /// caller 가 next `code+state` 받을 때까지 wait (timeout).
    pub async fn wait_for_callback(mut self, timeout: Duration) -> Result<CallbackParams, OAuthError> {
        let rx = self.rx.take().ok_or_else(|| {
            OAuthError::CallbackServer("rx already consumed".into())
        })?;
        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(params)) => Ok(params),
            Ok(Err(_)) => Err(OAuthError::CallbackServer("channel closed".into())),
            Err(_) => Err(OAuthError::CallbackServer("timeout".into())),
        }
    }
}

async fn handle_connection(
    sock: &mut tokio::net::TcpStream,
    expected_path: &str,
    tx: oneshot::Sender<CallbackParams>,
) -> Result<(), OAuthError> {
    let mut buf = vec![0u8; 4096];
    let n = sock.read(&mut buf).await?;
    let req = String::from_utf8_lossy(&buf[..n]).to_string();
    // "GET /callback?code=...&state=... HTTP/1.1\r\n..."
    let request_line = req.lines().next().unwrap_or("");
    let path_and_query = request_line
        .split_whitespace()
        .nth(1)
        .unwrap_or("/")
        .to_string();
    let (path, query) = match path_and_query.split_once('?') {
        Some((p, q)) => (p.to_string(), q.to_string()),
        None => (path_and_query, String::new()),
    };
    // 성공/실패 모두 client 에 HTML 응답
    let (status, body) = if path == expected_path {
        let mut code = None;
        let mut state = None;
        for pair in query.split('&') {
            if let Some((k, v)) = pair.split_once('=') {
                let k_decoded = url_decode(k);
                let v_decoded = url_decode(v);
                if k_decoded == "code" { code = Some(v_decoded.clone()); }
                if k_decoded == "state" { state = Some(v_decoded); }
            }
        }
        match (code, state) {
            (Some(c), Some(s)) => {
                let _ = tx.send(CallbackParams { code: c, state: s });
                (200, "<h1>200 OK</h1><p>You can close this window.</p>")
            }
            _ => (400, "<h1>400 Bad Request</h1>"),
        }
    } else {
        (404, "<h1>404 Not Found</h1>")
    };
    let response = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {len}\r\nConnection: close\r\n\r\n{body}",
        reason = match status { 200 => "OK", 400 => "Bad Request", 404 => "Not Found", _ => "Error" },
        len = body.len(),
    );
    sock.write_all(response.as_bytes()).await?;
    sock.shutdown().await?;
    Ok(())
}

fn url_decode(s: &str) -> String {
    // 매우 단순: '+' → ' ', %XX → byte
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'+' {
            out.push(' ');
            i += 1;
        } else if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(b) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(b as char);
                i += 3;
            } else {
                out.push(bytes[i] as char);
                i += 1;
            }
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_decode_basic() {
        assert_eq!(url_decode("hello"), "hello");
        assert_eq!(url_decode("hello+world"), "hello world");
        assert_eq!(url_decode("a%20b"), "a b");
        assert_eq!(url_decode("a%2Fb"), "a/b");
    }

    #[tokio::test]
    async fn callback_server_starts_on_random_port() {
        let s = CallbackServer::start(0, "/callback").await.unwrap();
        assert_eq!(s.local_addr.ip().to_string(), "127.0.0.1");
        assert!(s.local_addr.port() > 0);
    }

    #[tokio::test]
    async fn callback_server_receives_code_and_state() {
        let s = CallbackServer::start(0, "/callback").await.unwrap();
        let port = s.local_addr.port();
        // 127.0.0.1:port 로 GET /callback?code=abc&state=xyz 요청
        tokio::spawn(async move {
            let () = tokio::time::sleep(Duration::from_millis(50)).await;
            let _ = reqwest::get(format!("http://127.0.0.1:{port}/callback?code=abc&state=xyz")).await;
        });
        let params = s.wait_for_callback(Duration::from_secs(2)).await.unwrap();
        assert_eq!(params.code, "abc");
        assert_eq!(params.state, "xyz");
    }

    #[tokio::test]
    async fn callback_server_timeout() {
        let s = CallbackServer::start(0, "/callback").await.unwrap();
        let r = s.wait_for_callback(Duration::from_millis(200)).await;
        assert!(matches!(r, Err(OAuthError::CallbackServer(_))));
    }

    #[tokio::test]
    async fn callback_server_wrong_path() {
        let s = CallbackServer::start(0, "/callback").await.unwrap();
        let port = s.local_addr.port();
        tokio::spawn(async move {
            let () = tokio::time::sleep(Duration::from_millis(50)).await;
            let _ = reqwest::get(format!("http://127.0.0.1:{port}/wrong")).await;
        });
        // 404 → callback 안 옴 → timeout
        let r = s.wait_for_callback(Duration::from_millis(500)).await;
        assert!(r.is_err());
    }
}
