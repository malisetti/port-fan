use std::net::SocketAddr;
use std::time::Duration;

#[tokio::test]
async fn tcp_connect_local_port_22_typically_refused() {
    let addr: SocketAddr = "127.0.0.1:22".parse().unwrap();
    let connected = portfan::tcp::tcp_connect(addr, Duration::from_secs(2))
        .await
        .expect("connect attempt should not error on refused");
    assert!(
        !connected,
        "127.0.0.1:22 should be refused or closed without a successful connect"
    );
}

#[tokio::test]
async fn tcp_connect_local_port_1_refused() {
    let addr: SocketAddr = "127.0.0.1:1".parse().unwrap();
    let connected = portfan::tcp::tcp_connect(addr, Duration::from_secs(2))
        .await
        .expect("connect attempt should not error on refused");
    assert!(!connected, "127.0.0.1:1 should be refused");
}
