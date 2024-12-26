use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use kittymc_lib::error::KittyMCError;

pub struct Client {
    socket: TcpStream,
    addr: SocketAddr,
}

impl Client {
    pub async fn accept(server: &TcpListener) -> Result<Client, KittyMCError> {
        let (socket, addr) = server.accept().await?;

        println!("Client {addr} connected");

        Ok(Client {
            socket,
            addr
        })
    }

    pub async fn run(self) {
        tokio::spawn(async move {
            match self.client_loop().await {
                Err(e) => eprintln!("Fatal error in client {}: {e}", self.addr),
                Ok(()) => println!("Client {} disconnected.", self.addr),
            }
        });
    }

    async fn client_loop(&self) -> Result<(), KittyMCError> {
        loop {
        }
    }
}