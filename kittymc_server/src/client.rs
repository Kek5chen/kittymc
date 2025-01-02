use std::fmt::Debug;
use std::future::poll_fn;
use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::task::Poll;
use std::time::{Duration, Instant};
use uuid::Uuid;

use tracing::{debug, error, info, instrument, trace, warn};

use kittymc_lib::error::KittyMCError;
use kittymc_lib::packets::client::login::success_02::LoginSuccessPacket;
use kittymc_lib::packets::client::status::response_00::StatusResponsePacket;
use kittymc_lib::packets::{Packet, packet_serialization::SerializablePacket};
use kittymc_lib::packets::client::play::keep_alive_00::KeepAlivePacket;
use kittymc_lib::subtypes::state::State;
use crate::packet_routing::PacketSendInfo;
use crate::server::KittyMCServer;

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct ClientInfo {
    pub uuid: Uuid,
    pub username: String,
}

#[derive(Debug)]
pub struct Client {
    socket: TcpStream,
    addr: SocketAddr,
    current_state: State,
    last_heartbeat: Instant,
    last_heartbeat_id: u32,
    last_heartbeat_response: Instant,
    buffer: Vec<u8>,
    buffer_size: usize,
}

impl Client {
    #[instrument(skip(server))]
    pub fn accept(server: &TcpListener) -> Result<Option<Client>, KittyMCError> {
        let (socket, addr) = match server.accept() {
            Ok(socket) => socket,
            Err(e) if e.kind() == ErrorKind::WouldBlock => return Ok(None),
            Err(e) => Err(e)?,
        };

        socket.set_nonblocking(true).expect("Couldn't set socket to nonblocking");

        Client::new(socket, addr).map(|c| Some(c))
    }

    #[instrument(skip(socket, addr))]
    pub fn new(socket: TcpStream, addr: SocketAddr) -> Result<Client, KittyMCError> {
        info!("[{}] Client connected", addr);

        Ok(
            Client {
                socket,
                addr,
                current_state: State::Handshake,
                last_heartbeat: Instant::now(),
                last_heartbeat_id: 0,
                last_heartbeat_response: Instant::now(),
                buffer: vec![0; 2048],
                buffer_size: 0,
            },
        )
    }

    pub fn addr(&self) -> &SocketAddr {
        &self.addr
    }

    pub fn set_state(&mut self, state: State) {
        if state == State::Play {
            self.last_heartbeat = Instant::now();
            self.last_heartbeat_response = Instant::now();
        }

        self.current_state = state;
    }

    #[instrument(skip(self, b_packet))]
    pub fn send_packet_raw(&mut self, b_packet: &[u8]) -> Result<(), KittyMCError> {
        self.socket.write_all(b_packet)?;

        trace!("Sent : {b_packet:?}");

        Ok(())
    }

    #[instrument(skip(self))]
    pub fn send_packet<P: SerializablePacket + Debug>(&mut self, packet: &P) -> Result<(), KittyMCError> {
        self.send_packet_raw(&packet.serialize())?;
        Ok(())
    }

    pub fn do_heartbeat(&mut self) -> Result<bool, KittyMCError> {
        if self.current_state != State::Play {
            return Ok(true);
        }

        if self.last_heartbeat_response < self.last_heartbeat {
            return Ok(self.last_heartbeat.elapsed() <= Duration::from_secs(30));
        }

        if self.last_heartbeat.elapsed() >= Duration::from_secs(5) {
            self.last_heartbeat_id = rand::random();
            self.send_packet(&KeepAlivePacket::new(self.last_heartbeat_id))?;
            self.last_heartbeat = Instant::now();
        }

        Ok(true)
    }

    #[instrument(skip(self))]
    pub fn fetch_packet(&mut self) -> Result<Option<Packet>, KittyMCError> {
        let mut fetch_more = false;
        loop {
            let mut n = self.buffer_size;
            if n == 0 || fetch_more {
                fetch_more = false;
                if n == self.buffer.len() {
                    self.buffer.resize(n + 2048, 0);
                }
                let max_len = self.buffer.len();
                match self.socket.read(&mut self.buffer[n..max_len]) {
                    Ok(0) => {
                        // The other side closed the connection
                        return Err(KittyMCError::Disconnected);
                    }
                    Ok(new_n) => n += new_n,
                    Err(e) if e.kind() == ErrorKind::WouldBlock => return Ok(None),
                    Err(e) => return Err(e.into()),
                }

                trace!("[{}] Complete Received Packet : {:?}", self.addr, &self.buffer[..n]);
            }

            let (packet_len, packet) =
                match Packet::deserialize_packet(self.current_state, &self.buffer[..n]) {
                    Ok(packet) => {
                        trace!("[{}] Received : {:?}", self.addr, &self.buffer[..packet.0]);
                        trace!("[{}] Parsed : {:?}", self.addr, packet.1);
                        packet
                    },
                    Err(KittyMCError::NotEnoughData(_, _)) => {
                        fetch_more = true;
                        continue;
                    }
                    Err(e) => {
                        error ! ("[{}] Error when deserializing packet: {}", self.addr, e);
                        error ! ("[{}] Packet started with : {:?}", self.addr, & self.buffer[..n]);
                        return Err(KittyMCError::DeserializationError);
                    }
                };

            self.buffer_size = n - packet_len;
            self.buffer.drain(0..packet_len);

            if self.buffer_size < 2048 {
                self.buffer.resize(2048, 0); // shouldn't be able to become smaller than 2048
            }


            return Ok(Some(packet));
        }
    }
}
