use std::io::ErrorKind;
use std::net::SocketAddr;
use std::time::Duration;

use tokio::net::TcpStream;
use tokio::time::timeout;

fn is_host_unreachable(err: &std::io::Error) -> bool {
    matches!(
        err.kind(),
        ErrorKind::HostUnreachable
            | ErrorKind::NetworkUnreachable
            | ErrorKind::AddrNotAvailable
    )
}

/// Attempt a single TCP connect to `addr`, bounded by `connect_timeout`.
///
/// Returns `Ok(true)` when the connection succeeds, `Ok(false)` on connection
/// refused or timeout, and `Err` when the host is unreachable.
pub async fn tcp_connect(addr: SocketAddr, connect_timeout: Duration) -> Result<bool, anyhow::Error> {
    match timeout(connect_timeout, TcpStream::connect(addr)).await {
        Ok(Ok(_stream)) => Ok(true),
        Ok(Err(err)) => {
            if is_host_unreachable(&err) {
                return Err(err.into());
            }
            Ok(false)
        }
        Err(_elapsed) => Ok(false),
    }
}
