use kittymc_lib::packets::packet_serialization::NamedPacket;
use std::collections::HashSet;
use std::fmt::Debug;
use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::{Duration, Instant};
use uuid::Uuid;

use tracing::{debug, info, instrument, trace, warn};

use crate::chunk_manager::ChunkManager;
use kittymc_lib::error::KittyMCError;
use kittymc_lib::packets::client::play::keep_alive_1f::ServerKeepAlivePacket;
use kittymc_lib::packets::client::play::{ChunkDataPacket, UnloadChunkPacket};
use kittymc_lib::packets::packet_serialization::compress_packet;
use kittymc_lib::packets::{packet_serialization::SerializablePacket, CompressionInfo, Packet};
use kittymc_lib::subtypes::state::State;
use kittymc_lib::subtypes::{ChunkPosition, Location};

const CHUNK_LOAD_RADIUS: f32 = 5.;

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct ClientInfo {
    pub uuid: Uuid,
    pub username: String,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Client {
    connected_at: Instant,
    socket: TcpStream,
    addr: SocketAddr,
    current_state: State,
    last_heartbeat: Instant,
    last_heartbeat_id: u64,
    last_backbeat: Instant,
    buffer: Vec<u8>,
    buffer_size: usize,
    fragmented: bool,
    compression: CompressionInfo,
    brand: Option<String>,
    loaded_chunks: HashSet<ChunkPosition>,
}

#[allow(dead_code)]
impl Client {
    #[instrument(skip(server))]
    pub fn accept(server: &TcpListener) -> Result<Option<Client>, KittyMCError> {
        let (socket, addr) = match server.accept() {
            Ok(socket) => socket,
            Err(e) if e.kind() == ErrorKind::WouldBlock => return Ok(None),
            Err(e) => Err(e)?,
        };

        socket
            .set_nonblocking(true)
            .expect("Couldn't set socket to nonblocking");

        Client::new(socket, addr).map(|c| Some(c))
    }

    #[instrument(skip(socket, addr))]
    pub fn new(socket: TcpStream, addr: SocketAddr) -> Result<Client, KittyMCError> {
        info!("[{}] Client connected", addr);

        Ok(Client {
            connected_at: Instant::now(),
            socket,
            addr,
            current_state: State::Handshake,
            last_heartbeat: Instant::now(),
            last_heartbeat_id: 0,
            last_backbeat: Instant::now(),
            buffer: vec![0; 2048],
            buffer_size: 0,
            fragmented: false,
            compression: CompressionInfo::default(),
            brand: None,
            loaded_chunks: HashSet::new(),
        })
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

    pub fn set_compression(&mut self, compress: bool, threshold: u32) {
        self.compression.enabled = compress;
        self.compression.compression_threshold = threshold;
    }

    pub fn set_brand(&mut self, brand: String) {
        self.brand = Some(brand);
    }

    // TODO: Something is broken with compression
    #[instrument(skip(self, b_packet))]
    pub fn send_packet_raw(&mut self, b_packet: &[u8]) -> Result<(), KittyMCError> {
        if self.compression.enabled {
            let compressed = compress_packet(b_packet, self.compression.compression_threshold)?;
            self.socket.write_all(&compressed)?;
        } else {
            self.socket.write_all(&b_packet)?;
        }

        Ok(())
    }

    #[instrument(skip(self, packet))]
    pub fn send_packet<P: SerializablePacket + Debug + NamedPacket>(
        &mut self,
        packet: &P,
    ) -> Result<(), KittyMCError> {
        debug!(
            "[{}] OUT >>> {}(0x{:x?})({})",
            self.addr,
            P::name(),
            P::id(),
            P::id()
        );
        self.send_packet_raw(&packet.serialize())?;
        Ok(())
    }

    pub fn do_heartbeat(&mut self) -> Result<bool, KittyMCError> {
        if self.current_state != State::Play {
            return Ok(true);
        }

        if self.last_heartbeat.elapsed() >= Duration::from_secs(5) {
            self.last_heartbeat_id = rand::random();
            self.send_packet(&ServerKeepAlivePacket::new(self.last_heartbeat_id))?;
            self.last_heartbeat = Instant::now();
        }

        Ok(self.last_backbeat.elapsed() <= Duration::from_secs(30))
    }

    pub fn register_backbeat(&mut self, _id: u64) {
        // TODO: Should probably store four heartbeat ids and then see if any matches
        //if self.last_heartbeat_id == id {
        self.last_backbeat = Instant::now();
        //}
    }

    #[instrument(skip(self))]
    pub fn fetch_packet(&mut self) -> Result<Option<Packet>, KittyMCError> {
        let mut n = self.buffer_size;
        if n == self.buffer.len() {
            // buffer has not enough space to fit packet. so extend it.
            self.buffer.resize(n + 2048, 0);
            trace!("[{}] Increased buffer size to fit bigger packet", self.addr);
        }
        let max_len = self.buffer.len();
        match self.socket.read(&mut self.buffer[n..max_len]) {
            Ok(0) => {
                // The other side closed the connection
                return Err(KittyMCError::Disconnected);
            }
            Ok(new_n) => {
                self.fragmented = false;
                n += new_n;
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                if n == 0 || self.fragmented {
                    return Ok(None);
                }
            }
            Err(e) => return Err(e.into()),
        }
        self.buffer_size = n;

        trace!(
            "[{}] Complete Received Data : {:?}",
            self.addr,
            &self.buffer[..n]
        );

        let (packet_len, packet) =
            match Packet::deserialize(self.current_state, &self.buffer[..n], &self.compression) {
                Ok(packet) => {
                    debug!(
                        "[{}] IN <<< {}(0x{:x?})({})",
                        self.addr,
                        packet.1.name(),
                        packet.1.id(),
                        packet.1.id()
                    );
                    (packet.0, Some(packet.1))
                }
                Err(KittyMCError::NotEnoughData(_, _)) => {
                    trace!("[{}] Not enough data. Waiting for more", self.addr);
                    self.fragmented = true;
                    return Ok(None);
                }
                Err(KittyMCError::NotImplemented(packet_id, packet_len)) => {
                    warn!(
                        "[{}] IN UNIMPLEMENTED <<< UNKNOWN(0x{:x?})({}) (len: {})",
                        self.addr, packet_id, packet_id, packet_len
                    );
                    (packet_len, None)
                }
                Err(e) => {
                    warn!("[{}] Error when deserializing packet: {}", self.addr, e);
                    warn!(
                        "[{}] Packet started with : {:?}",
                        self.addr,
                        &self.buffer[..n]
                    );
                    return Err(e);
                }
            };

        self.buffer_size = n - packet_len;
        self.buffer.drain(0..packet_len);

        if self.buffer_size < 2048 {
            self.buffer.resize(2048, 0); // shouldn't be able to become smaller than 2048
        }

        Ok(packet)
    }

    pub fn load_chunks<'a, I>(
        &mut self,
        positions: I,
        chunk_manager: &mut ChunkManager,
    ) -> Result<(), KittyMCError>
    where
        I: Iterator<Item = &'a ChunkPosition>,
    {
        for pos in positions {
            if self.loaded_chunks.contains(pos) {
                continue;
            }
            let Some(chunk) = chunk_manager.request_chunk(pos) else {
                continue;
            };

            trace!("[{}] Loading new Chunk {:?}", self.addr, pos);

            {
                let chunk = chunk.read().unwrap();
                let packet =
                    ChunkDataPacket::new(chunk.as_ref(), pos.x() as i32 / 16, pos.z() as i32 / 16);
                self.send_packet(&packet)?;
            }
            self.loaded_chunks.insert(pos.clone());
        }

        Ok(())
    }

    pub fn unload_chunks<'a, I>(&mut self, positions: I)
    where
        I: Iterator<Item = &'a ChunkPosition>,
    {
        for pos in positions {
            let packet = UnloadChunkPacket::new(pos);
            trace!("[{}] Unloading Chunk {:?}", self.addr, pos);
            match self.send_packet(&packet) {
                Ok(_) => {
                    self.loaded_chunks.remove(pos);
                }
                _ => (),
            }
        }
    }

    #[instrument(skip(self, pos, chunk_manager))]
    pub fn update_chunks(
        &mut self,
        pos: &Location,
        chunk_manager: &mut ChunkManager,
    ) -> Result<(), KittyMCError> {
        let new: Vec<_> =
            ChunkPosition::iter_xz_circle_in_range(pos, CHUNK_LOAD_RADIUS * 16.).collect();
        let unloadable = self
            .loaded_chunks
            .iter()
            .filter(|c| !new.contains(c))
            .cloned()
            .collect::<Vec<_>>();

        self.load_chunks(new.iter(), chunk_manager)?;
        self.unload_chunks(unloadable.iter());

        Ok(())
    }
}
