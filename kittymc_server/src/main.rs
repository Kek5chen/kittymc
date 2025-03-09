mod client;
mod player;
mod server;
mod inventory;
mod chunking;

use log::error;
use crate::server::KittyMCServer;
use tracing::metadata::LevelFilter;
use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .pretty()
        .compact()
        .with_target(false)
        .with_line_number(false)
        .with_file(false)
        .init();

    let mut server = match KittyMCServer::new(25565) {
        Ok(server) => server,
        Err(e) => {
            error!("Error while trying to start the server: {e}");
            return;
        }
    };

    if let Err(e) = server.run() {
        error!("Error occurred while server was running: {e}");
    };
} 
