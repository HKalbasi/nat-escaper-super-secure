use std::sync::Arc;
use tokio::io::copy_bidirectional;
use tokio::net::TcpListener;
use tokio::task::JoinSet;
use tokio_stream_multiplexor::StreamMultiplexorConfig;

use crate::TARGET_URL;

pub async fn run() -> anyhow::Result<()> {
    let listener = TcpListener::bind(TARGET_URL).await?;
    println!("Server listening on {TARGET_URL}");

    let (socket, addr) = listener.accept().await?;
    println!("New connection from: {}", addr);

    let socket = Arc::new(tokio_stream_multiplexor::StreamMultiplexor::new(
        socket,
        StreamMultiplexorConfig::default(),
    ));

    let mut task_tracker = JoinSet::new();

    for port in [8000, 8082] {
        let socket = socket.clone();
        task_tracker.spawn(async move {
            let listener = TcpListener::bind(("127.0.0.1", port)).await?;

            loop {
                let (mut up_socket, _) = listener.accept().await?;
                let mut down_socket = socket.connect(port).await?;
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
