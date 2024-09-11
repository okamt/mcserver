use connection::ConnectionManager;
use tokio::net::ToSocketAddrs;

pub mod connection;

pub struct MinecraftServer {
    connection_manager: ConnectionManager,
}

impl MinecraftServer {
    pub async fn new<A>(address: A) -> std::io::Result<Self>
    where
        A: ToSocketAddrs,
    {
        Ok(MinecraftServer {
            connection_manager: ConnectionManager::new(address).await?,
        })
    }

    pub async fn start(&self) -> ! {
        self.connection_manager.listen().await
    }
}
