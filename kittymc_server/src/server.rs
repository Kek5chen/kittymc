use crate::client::{Client, ClientInfo};
use crate::player::Player;
use kittymc_lib::error::KittyMCError;
use kittymc_lib::packets::client::login::*;
use kittymc_lib::packets::client::play::*;
use kittymc_lib::packets::client::status::*;
use kittymc_lib::packets::packet_serialization::NamedPacket;
use kittymc_lib::packets::packet_serialization::SerializablePacket;
use kittymc_lib::packets::Packet;
use kittymc_lib::subtypes::state::State;
use log::debug;
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::net::TcpListener;
use std::sync::RwLock;
use tracing::{info, instrument, warn};
use uuid::Uuid;

#[derive(Debug)]
pub struct KittyMCServer {
    server: TcpListener,
    players: HashMap<Uuid, Player>,
    clients: RwLock<HashMap<Uuid, Client>>,
    registering_clients: VecDeque<Client>,
}

#[allow(dead_code)]
impl KittyMCServer {
    #[instrument(skip(port))]
    pub fn new(port: u16) -> Result<KittyMCServer, KittyMCError> {
        let server = TcpListener::bind(("0.0.0.0", port))?;

        server.set_nonblocking(true)?;

        info!("Starting server on port {port}");

        Ok(KittyMCServer {
            server,
            players: HashMap::new(),
            clients: RwLock::new(HashMap::new()),
            registering_clients: VecDeque::new(),
        })
    }

    fn get_name_from_uuid(&self, uuid: &Uuid) -> Option<&str> {
        self.players.get(uuid).map(|p| p.name())
    }

    fn send_to_all<P: SerializablePacket + Debug + NamedPacket>(
        &mut self,
        packet: &P,
    ) -> Result<(), KittyMCError> {
        for client in self.clients.write().unwrap().iter_mut() {
            client.1.send_packet(packet)?;
        }

        Ok(())
    }

    fn handle_client(&self, client: &mut Client) -> Result<bool, KittyMCError> {
        if !client.do_heartbeat()? {
            debug!(
                "[{}] Client didn't respond to heartbeats for too long",
                client.addr()
            );
            return Ok(false);
        }

        loop {
            let Some(packet) = client.fetch_packet()? else {
                return Ok(true);
            };

            // debug!("New packet :3 {:?}", packet);

            match &packet {
                Packet::KeepAlive(packet) => {
                    client.register_backbeat(packet.id);
                }
                Packet::PluginMessage(msg) if msg.channel == "MC|Brand" => {
                    client.set_brand(String::from_utf8_lossy(&msg.data).to_string())
                }
                _ => (),
            }
        }
    }

    fn handle_client_pre_play(
        &mut self,
        client: &mut Client,
    ) -> Result<Option<Uuid>, KittyMCError> {
        loop {
            let Some(packet) = client.fetch_packet()? else {
                return Ok(None);
            };

            match &packet {
                Packet::Handshake(handshake) => {
                    if handshake.protocol_version != 340 && handshake.next_state != State::Status {
                        info!("[{}] Client tried to connect with protocol version {} != 340. Disconnecting.", client.addr(), handshake.protocol_version);
                        client.send_packet(&DisconnectLoginPacket::wrong_version())?;
                        return Err(KittyMCError::VersionMissmatch);
                    }
                    client.set_state(handshake.next_state);
                }
                Packet::StatusRequest(_) => {
                    client.send_packet(&StatusResponsePacket::default())?;
                }
                Packet::StatusPing(ping) => {
                    client.send_packet(ping)?;
                }
                Packet::LoginStart(login) => {
                    let success = LoginSuccessPacket::from_name_cracked(&login.name)?;
                    let client_info = ClientInfo {
                        username: success.username.clone(),
                        uuid: success.uuid.clone(),
                    };

                    let player = Player::from_client_info(client_info);
                    let uuid = player.uuid().clone();
                    self.players.insert(uuid.clone(), player);

                    let compression = SetCompressionPacket::default();
                    client.send_packet(&compression)?;
                    client.set_compression(true, compression.threshold);

                    client.send_packet(&success)?;
                    client.set_state(State::Play);

                    client.send_packet(&JoinGamePacket::default())?;
                    client.send_packet(&ServerPluginMessagePacket::default_brand())?;
                    client.send_packet(&ServerDifficultyPacket::default())?;
                    client.send_packet(&PlayerAbilitiesPacket::default())?;
                    client.send_packet(&ServerHeldItemChangePacket::default())?;
                    client.send_packet(&EntityStatusPacket::default())?;
                    client.send_packet(&UnlockRecipesPacket::default())?;
                    client.send_packet(&PlayerListItemPacket::default())?;
                    // Another Player List Item
                    client.send_packet(&ServerPlayerPositionAndLookPacket::default())?;
                    // World Border
                    client.send_packet(&TimeUpdatePacket::default())?;
                    client.send_packet(&SpawnPositionPacket::default())?;
                    // Player Digging ???
                    // Steer Vehicle ???

                    // after client answers send chunks
                    for x in -4..4 {
                        for z in -4..4 {
                            client.send_packet(&ChunkDataPacket::default_at(x, z))?;
                        }
                    }

                    return Ok(Some(uuid));
                }
                _ => {}
            }
        }
    }

    fn handle_clients(&mut self) -> Result<(), KittyMCError> {
        let new_client = Client::accept(&self.server)?;

        if let Some(new_client) = new_client {
            self.registering_clients.push_back(new_client);
        }

        for _ in 0..self.registering_clients.len() {
            let mut client = self.registering_clients.pop_front().unwrap();

            match self.handle_client_pre_play(&mut client) {
                Err(e) => {
                    info!("[{}] Registering Client disconnected ({e})", client.addr());
                    continue;
                }
                Ok(opt_uuid) => match opt_uuid {
                    Some(uuid) => {
                        debug!("[{}] Client successfully registered", client.addr());
                        self.clients.write().unwrap().insert(uuid, client);
                    }
                    None => self.registering_clients.push_back(client),
                },
            }
        }

        let mut disconnect_uuids = vec![];

        let mut clients = self.clients.write().unwrap();
        for client in clients.iter_mut() {
            match self.handle_client(client.1) {
                Ok(keep_alive) => {
                    if !keep_alive {
                        info!("[{}] Forced Client disconnect", client.1.addr());
                        disconnect_uuids.push(client.0.clone());
                    }
                }
                Err(KittyMCError::Disconnected) => {
                    info!("[{}] Client disconnected", client.1.addr());
                    disconnect_uuids.push(client.0.clone())
                }
                Err(e) => {
                    warn!(
                        "[{}] Disconnected client due to error: {e}",
                        client.1.addr()
                    );
                    disconnect_uuids.push(client.0.clone())
                }
            }
        }

        for uuid in disconnect_uuids {
            clients.remove(&uuid);
        }

        Ok(())
    }

    #[instrument(skip(self))]
    pub fn run(&mut self) -> Result<(), KittyMCError> {
        loop {
            self.handle_clients()?;
        }
    }
}
