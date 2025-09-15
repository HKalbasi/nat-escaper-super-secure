use clap::Parser;

mod owner;
mod relay;

#[derive(clap::Parser)]
enum Command {
    Relay,
    Owner,
}

const TARGET_URL: &str = "127.0.0.1:10212";

#[tokio::main]
async fn main() {
    let cmd = Command::parse();

    match cmd {
        Command::Relay => {
            relay::run().await.unwrap();
        }
        Command::Owner => {
            owner::run().await.unwrap();
        }
    }
}
