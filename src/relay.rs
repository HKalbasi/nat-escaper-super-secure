use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use crate::TARGET_URL;

pub async fn run() -> anyhow::Result<()> {
    let listener = TcpListener::bind(TARGET_URL).await?;
    println!("Echo server listening on {TARGET_URL}");

    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("New connection from: {}", addr);

        tokio::spawn(async move {
            let mut buf = [0; 1024];

            loop {
                match socket.read(&mut buf).await {
                    Ok(0) => {
                        // Connection closed
                        println!("Connection closed by: {}", addr);
                        break;
                    }
                    Ok(n) => {
                        // Echo the data back
                        if let Err(e) = socket.write_all(&buf[0..n]).await {
                            eprintln!("Failed to write to socket: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to read from socket: {}", e);
                        break;
                    }
                }
            }
        });
    }
}
