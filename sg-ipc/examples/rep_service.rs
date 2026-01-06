use anyhow::Context;
use clap::Parser;
use sg_ipc::RepService;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(required = true)]
    domain_socket_path: PathBuf,
}

async fn callback(msg: String) -> Result<String, anyhow::Error> {
    println!("Received message: {msg}");
    Ok(format!("Reply from server for {msg}").to_string())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), anyhow::Error> {
    let args = Cli::parse();
    let domain_socket_path = args
        .domain_socket_path
        .to_str()
        .context("Invalid domain socket path")?;

    let rep_service = RepService::new(domain_socket_path);
    rep_service.start(callback).context("join handler")?;

    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    Ok(())
}
