use std::io::ErrorKind;
use std::net::SocketAddr;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use kittymc_lib::error::KittyMCError;
use kittymc_lib::packets::Packet;
use kittymc_lib::subtypes::state::State;

pub struct Client {
    socket: TcpStream,
    addr: SocketAddr,
    current_state: State,
}

impl Client {
    pub async fn accept(server: &TcpListener) -> Result<Client, KittyMCError> {
        let (socket, addr) = server.accept().await?;

        println!("Client {addr} connected");

        Ok(Client {
            socket,
            addr,
            current_state: State::Handshake,
        })
    }

    pub async fn run(mut self) {
        tokio::spawn(async move {
            match self.client_loop().await {
                Err(e) => eprintln!("Fatal error in client {}: {e}", self.addr),
                Ok(()) => println!("Client {} disconnected.", self.addr),
            }
        });
    }

    async fn client_loop(&mut self) -> Result<(), KittyMCError> {
        loop {
            match self.socket.readable().await {
                Ok(_) => {}
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => continue,
                Err(e) => Err(e)?,
            }
            let mut buffer = vec![0u8; 2048];
            let mut n = match self.socket.read(&mut buffer).await {
                Ok(0) => {
                    // The other side closed the connection
                    return Ok(());
                }
                Ok(n) => n,
                Err(e) => Err(e)?,
            };
            let packet = Packet::deserialize_packet(self.current_state, &buffer);
            match &packet {
                Ok((n, packet)) => match packet {
                    Packet::Handshake(handshake) => {
                        if handshake.protocol_version != 47 {
                            return Ok(());
                        }
                        self.current_state = handshake.next_state;
                    }
                    Packet::LoginStart(_) => {}
                }
                Err(e) => continue,
            }
            println!("buffer: {:?}", packet);
        }
    }
}