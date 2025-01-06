mod chunk_manager;
mod client;
mod player;
mod server;

use crate::server::KittyMCServer;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(LevelFilter::INFO.into()))
        .pretty()
        .compact()
        .with_target(false)
        .with_line_number(false)
        .with_file(false)
        .without_time()
        .init();

    let mut server = match KittyMCServer::new(25565) {
        Ok(server) => server,
        Err(e) => {
            eprintln!("Error while trying to start the server: {e}");
            return;
        }
    };

    match server.run() {
        Err(e) => {
            eprintln!("Error occurred while server was running: {e}");
            return;
        }
        _ => (),
    };
}
