mod server;
mod client;
mod player;

use tracing_subscriber::EnvFilter;
use crate::server::KittyMCServer;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
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
