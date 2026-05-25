use anyhow::{anyhow, Context, Result};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpFingerprint {
    pub server: Option<String>,
    pub status: u16,
    pub content_type: Option<String>,
}

const REQUEST: &[u8] = b"GET / HTTP/1.0\r\nHost: x\r\n\r\n";

pub async fn fingerprint(addr: SocketAddr, timeout: Duration) -> Result<HttpFingerprint, anyhow::Error> {
    time::timeout(timeout, fingerprint_inner(addr))
        .await
        .map_err(|_| anyhow!("connection timed out after {:?}", timeout))?
}

async fn fingerprint_inner(addr: SocketAddr) -> Result<HttpFingerprint> {
    let mut stream = TcpStream::connect(addr)
        .await
        .with_context(|| format!("failed to connect to {addr}"))?;

    stream
        .write_all(REQUEST)
        .await
        .context("failed to write HTTP request")?;

    let mut buf = Vec::with_capacity(4096);
    let mut chunk = [0u8; 1024];
    loop {
        let n = stream
            .read(&mut chunk)
            .await
            .context("failed to read HTTP response")?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..n]);
        if buf.windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
        if buf.len() >= 64 * 1024 {
            break;
        }
    }

    parse_response(&buf)
}

fn parse_response(buf: &[u8]) -> Result<HttpFingerprint> {
    let text = std::str::from_utf8(buf).context("response is not valid UTF-8")?;
    let mut lines = text.split("\r\n");
    let status_line = lines
        .next()
        .filter(|line| !line.is_empty())
        .ok_or_else(|| anyhow!("empty HTTP response"))?;

    let status = parse_status_code(status_line)?;
    let mut server = None;
    let mut content_type = None;

    for line in lines {
        if line.is_empty() {
            break;
        }
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.trim();
        if name.eq_ignore_ascii_case("server") {
            server = Some(value.to_string());
        } else if name.eq_ignore_ascii_case("content-type") {
            content_type = Some(value.to_string());
        }
    }

    Ok(HttpFingerprint {
        server,
        status,
        content_type,
    })
}

fn parse_status_code(status_line: &str) -> Result<u16> {
    let mut parts = status_line.split_whitespace();
    let _version = parts
        .next()
        .ok_or_else(|| anyhow!("missing HTTP version in status line"))?;
    let code = parts
        .next()
        .ok_or_else(|| anyhow!("missing status code in status line"))?;
    code.parse::<u16>()
        .with_context(|| format!("invalid status code {code:?}"))
}
