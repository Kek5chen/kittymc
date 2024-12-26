use tokio::net::TcpListener;
use kittymc_lib::error::KittyMCError;
use crate::client::Client;

pub struct KittyMCServer {
    server: TcpListener,
}

impl KittyMCServer {
    pub async fn new(port: u16) -> Result<KittyMCServer, KittyMCError> {
        let server = TcpListener::bind(("0.0.0.0", port)).await?;

        Ok(KittyMCServer {
            server,
        })
    }

    pub async fn run(&self) -> Result<(), KittyMCError> {
        loop {
            let client = Client::accept(&self.server).await?;
            client.run().await;
        }
    }
}