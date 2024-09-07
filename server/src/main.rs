use std::error::Error;

use server::MinecraftServer;
use tokio::signal;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting server...");

    let minecraft_server = MinecraftServer::new("127.0.0.1:25565").await?;

    tokio::spawn(async move {
        minecraft_server.start().await;
    });

    match signal::ctrl_c().await {
        Ok(()) => {}
        Err(err) => {
            tracing::error!("Unable to listen for shutdown signal: {}.", err);
        }
    }

    Ok(())
}
