use std::sync::Arc;

use anyhow::{Context, bail};
use tokio::io::{AsyncReadExt, AsyncWriteExt, copy_bidirectional};
use tokio::net::TcpStream;
use tokio::task::JoinSet;
use tokio_stream_multiplexor::StreamMultiplexorConfig;

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
    println!("Received from proxy: {response_str}");

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
    let stream = connect_via_proxy(&proxy, target).await?;

    let listener = Arc::new(tokio_stream_multiplexor::StreamMultiplexor::new(
        stream,
        StreamMultiplexorConfig::default(),
    ));

    let mut task_tracker = JoinSet::new();

    for port in [8000, 8082] {
        let listener = listener.clone();
        task_tracker.spawn(async move {
            let listener = listener.bind(port).await?;
            loop {
                let mut up_socket = listener.accept().await?;
                let mut down_socket = TcpStream::connect(("127.0.0.1", port)).await?;
                tokio::spawn(async move {
                    if let Err(e) = copy_bidirectional(&mut up_socket, &mut down_socket).await {
                        println!("{e}");
                    }
                });
            }

            #[allow(unreachable_code)]
            anyhow::Ok(())
        });
    }

    while let Some(res) = task_tracker.join_next().await {
        res??;
    }

    Ok(())
}
