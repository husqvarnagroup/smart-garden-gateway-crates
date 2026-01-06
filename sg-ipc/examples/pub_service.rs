use anyhow::Context;
use clap::Parser;
use sg_ipc::PubServiceBuilder;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(required = true)]
    domain_socket_path: PathBuf,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), anyhow::Error> {
    let args = Cli::parse();

    let (pub_service_builder, pub_service) = PubServiceBuilder::new();

    let server_task = tokio::task::spawn(async move {
        let domain_socket_path = args
            .domain_socket_path
            .to_str()
            .context("Invalid domain socket path")
            .unwrap();
        pub_service_builder
            .start(domain_socket_path)
            .expect("Failed to run pub service");
    });

    // Wait for clients to be ready
    tokio::time::sleep(Duration::from_secs(5)).await;

    for i in 0..10 {
        let msg = format!("Hello, World: {i}");
        println!("Publishing message: {msg}");
        pub_service
            .publish(&msg)
            .context("Failed to publish message")?;
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    pub_service.publish("").context("Failed to disconnect")?;

    server_task.await?;
    Ok(())
}
