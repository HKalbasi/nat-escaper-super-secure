use clap::Parser;

mod owner;
mod relay;

#[derive(clap::Parser)]
enum Command {
    Relay,
    Owner,
}

const TARGET_URL: &str = "141.11.246.113:10212";

#[tokio::main]
async fn main() {
    let cmd = Command::parse();

    match cmd {
        Command::Relay => {
            if let Err(e) = relay::run().await {
                println!("{e}");
                restart_program();
            }
        }
        Command::Owner => {
            if let Err(e) = owner::run().await {
                println!("{e}");
                restart_program();
            }
        }
    }
}

fn restart_program() {
    let current_exe = std::env::current_exe().expect("Failed to get executable path");
    let args: Vec<String> = std::env::args().collect();

    let _ = exec::execvp(current_exe, args);
    println!("failed to restart program");
}
