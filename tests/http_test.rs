use port_fan::fingerprint;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;

const CANNED_RESPONSE: &str = "\
HTTP/1.0 200 OK\r\n\
Server: mock-test/1.0\r\n\
Content-Type: text/html; charset=utf-8\r\n\
\r\n\
<body>ok</body>\
";

async fn mock_http_server() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let _ = stream.write_all(CANNED_RESPONSE.as_bytes()).await;
        }
    });

    addr
}

#[tokio::test]
async fn fingerprint_parses_mock_http_response() {
    let addr = mock_http_server().await;
    tokio::time::sleep(Duration::from_millis(10)).await;

    let fp = fingerprint(addr, Duration::from_secs(2))
        .await
        .expect("fingerprint should succeed");

    assert_eq!(fp.status, 200);
    assert_eq!(fp.server.as_deref(), Some("mock-test/1.0"));
    assert_eq!(
        fp.content_type.as_deref(),
        Some("text/html; charset=utf-8")
    );
}

#[tokio::test]
async fn fingerprint_times_out_on_unreachable_port() {
    let addr: SocketAddr = "127.0.0.1:1".parse().unwrap();
    let err = fingerprint(addr, Duration::from_millis(200))
        .await
        .expect_err("should time out or fail to connect");
    assert!(err.to_string().contains("timed out") || err.to_string().contains("connect"));
}
