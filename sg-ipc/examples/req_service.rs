use anyhow::Context;
use clap::Parser;
use rand::Rng;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(required = true)]
    domain_socket_path: PathBuf,

    #[arg()]
    messages: Vec<String>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), anyhow::Error> {
    let args = Cli::parse();
    let domain_socket_path = args
        .domain_socket_path
        .to_str()
        .context("Invalid domain socket path")?;

    let mut req_service = sg_ipc::ReqService::new(domain_socket_path).await?;

    let mut rng = rand::rng();

    for message in args.messages {
        sleep(Duration::from_millis(rng.random_range(1..500))).await;
        let rep = req_service.send(message).await?;
        println!("Received reply: {rep}");
    }

    Ok(())
}
