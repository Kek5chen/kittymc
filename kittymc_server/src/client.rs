use std::fmt::Debug;
use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::{Duration, Instant};
use uuid::Uuid;

use tracing::{debug, info, instrument, trace, warn};

use kittymc_lib::error::KittyMCError;
use kittymc_lib::packets::{Packet, packet_serialization::SerializablePacket};
use kittymc_lib::packets::client::play::keep_alive_00::KeepAlivePacket;
use kittymc_lib::packets::packet_serialization::compress_packet;
use kittymc_lib::subtypes::state::State;

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct ClientInfo {
    pub uuid: Uuid,
    pub username: String,
}

#[derive(Debug)]
pub struct Client {
    connected_at: Instant,
    socket: TcpStream,
    addr: SocketAddr,
    current_state: State,
    last_heartbeat: Instant,
    last_heartbeat_id: u32,
    last_backbeat: Instant,
    buffer: Vec<u8>,
    buffer_size: usize,
    compression: bool,
    brand: Option<String>,
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
                connected_at: Instant::now(),
                socket,
                addr,
                current_state: State::Handshake,
                last_heartbeat: Instant::now(),
                last_heartbeat_id: 0,
                last_backbeat: Instant::now(),
                buffer: vec![0; 2048],
                buffer_size: 0,
                compression: false,
                brand: None,
            },
        )
    }

    pub fn addr(&self) -> &SocketAddr {
        &self.addr
    }

    pub fn set_state(&mut self, state: State) {
        if state == State::Play {
            self.last_heartbeat = Instant::now();
            self.last_backbeat = Instant::now();
        }

        self.current_state = state;
    }

    pub fn set_compression(&mut self, compress: bool) {
        self.compression = compress;
    }

    pub fn set_brand(&mut self, brand: String) {
        self.brand = Some(brand);
    }

    #[instrument(skip(self, b_packet))]
    pub fn send_packet_raw(&mut self, b_packet: &[u8]) -> Result<(), KittyMCError> {
        trace!("================= SEND Packet Start ==================");
        if self.compression {
            let compressed = compress_packet(b_packet)?;
            self.socket.write_all(&compressed)?;
            trace!("[{}] Sent (C) : {compressed:?}", self.addr);
            trace!("[{}] Uncompressed : {b_packet:?}", self.addr);
        } else {
            self.socket.write_all(&b_packet)?;
            trace!("[{}] Sent (UC) : {b_packet:?}", self.addr);
        }
        trace!("================= SEND Packet End ==================");

        Ok(())
    }

    #[instrument(skip(self))]
    pub fn send_packet<P: SerializablePacket + Debug>(&mut self, packet: &P) -> Result<(), KittyMCError> {
        debug!("[{}] >>>", self.addr);
        self.send_packet_raw(&packet.serialize())?;
        Ok(())
    }

    pub fn do_heartbeat(&mut self) -> Result<bool, KittyMCError> {
        if self.current_state != State::Play {
            return Ok(true);
        }

        if self.last_heartbeat.elapsed() >= Duration::from_secs(5) {
            self.last_heartbeat_id = rand::random();
            self.send_packet(&KeepAlivePacket::new(self.last_heartbeat_id))?;
            self.last_heartbeat = Instant::now();
        }

        Ok(self.last_backbeat.elapsed() <= Duration::from_secs(30))
    }

    pub fn register_backbeat(&mut self, _id: u32) {
        // TODO: Should probably store four heartbeat ids and then see if any matches
        //if self.last_heartbeat_id == id {
            self.last_backbeat = Instant::now();
        //}
    }

    #[instrument(skip(self))]
    pub fn fetch_packet(&mut self) -> Result<Option<Packet>, KittyMCError> {
        let mut fetch_more = false;
        loop {
            let mut n = self.buffer_size;
            if n == 0 || fetch_more {
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
                trace!("================= RECV Packet Start ==================");

                trace!("[{}] Complete Received Packet : {:?}", self.addr, &self.buffer[..n]);
            } else {
                trace!("================= RECV Packet Start ==================");
            }

            let (packet_len, packet) =
                match Packet::deserialize_packet(self.current_state, &self.buffer[..n], self.compression) {
                    Ok(packet) => {
                        trace!("[{}] Parsed Range : {:?}", self.addr, &self.buffer[..packet.0]);
                        trace!("[{}] Parsed Value : {:?}", self.addr, packet.1);
                        debug!("[{}] <<< {:?}", self.addr, packet.1);
                        trace!("================= RECV Packet End ==================");
                        packet
                    },
                    Err(KittyMCError::NotEnoughData(_, _)) => {
                        trace!("[{}] Not enough data. Taking more...", self.addr);
                        fetch_more = true;
                        continue;
                    }
                    Err(e) => {
                        warn!("[{}] Error when deserializing packet: {}", self.addr, e);
                        warn!("[{}] Packet started with : {:?}", self.addr, & self.buffer[..n]);
                        trace!("================= RECV Packet End ==================");
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
