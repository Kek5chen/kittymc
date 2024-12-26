mod server;
mod client;

use crate::server::KittyMCServer;

#[tokio::main]
async fn main() {
    let server = match KittyMCServer::new(25565).await {
        Ok(server) => server,
        Err(e) => {
            eprintln!("Error while trying to start the server: {e}");
            return;
        }
    };

    match server.run().await {
        Ok(server) => server,
        Err(e) => {
            eprintln!("Error occurred while server was running: {e}");
            return;
        }
    };
}
