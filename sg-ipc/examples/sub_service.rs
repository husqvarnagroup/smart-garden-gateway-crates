use anyhow::Context;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(required = true)]
    domain_socket_path: PathBuf,
}

async fn callback(msg: String) {
    println!("Received message: {}", msg);
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), anyhow::Error> {
    let args = Cli::parse();
    let domain_socket_path = args
        .domain_socket_path
        .to_str()
        .context("Invalid domain socket path")?;

    let mut sub_service = sg_ipc::SubService::new(domain_socket_path);

    sub_service.start(callback).await?;
    Ok(())
}
