use anyhow::{Context, bail};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::TARGET_URL;

async fn connect_via_proxy(proxy_addr: &str, target_addr: &str) -> anyhow::Result<TcpStream> {
    // 1. Connect to the proxy
    let mut stream = TcpStream::connect(proxy_addr).await?;

    // 2. Send CONNECT request
    let request = format!(
        "CONNECT {} HTTP/1.1\r\nHost: {}\r\n\r\n",
        target_addr, target_addr
    );
    stream.write_all(request.as_bytes()).await?;

    // 3. Read and parse proxy response
    let mut response = Vec::new();
    let mut buffer = [0u8; 1024];

    loop {
        let n = stream.read(&mut buffer).await?;
        response.extend_from_slice(&buffer[..n]);

        // Look for the end of HTTP headers
        if response.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
    }

    // 4. Check if connection was successful
    let response_str = String::from_utf8_lossy(&response);
    if !response_str.starts_with("HTTP/1.1 200") && !response_str.starts_with("HTTP/1.0 200") {
        bail!("Proxy connection failed: {}", response_str);
    }

    println!("Successfully established TCP tunnel through proxy!");
    Ok(stream)
}

pub async fn run() -> anyhow::Result<()> {
    let proxy = std::env::var("http_proxy")?;
    let proxy = proxy
        .strip_prefix("http://")
        .context("http_proxy didn't start with http://")?;
    let target = TARGET_URL; // Can be any TCP service!

    // Get the tunneled connection
    let mut stream = connect_via_proxy(&proxy, target).await?;

    // Now use it like a regular TCP connection!
    // For HTTP:
    let http_request = "GET / HTTP/1.1\r\nHost: example.com\r\nConnection: close\r\n\r\n";
    stream.write_all(http_request.as_bytes()).await?;

    // For raw TCP (any protocol):
    let custom_data = b"Your custom protocol data here";
    stream.write_all(custom_data).await?;

    // Read response
    let mut response = Vec::new();
    let mut buffer = [0u8; 1024];
    loop {
        let n = stream.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
        response.extend_from_slice(&buffer[..n]);
    }

    println!("Received: {}", String::from_utf8_lossy(&response));
    Ok(())
}
