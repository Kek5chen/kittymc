use kittymc_lib::packets::packet_serialization::NamedPacket;
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::net::TcpListener;
use std::sync::RwLock;
use tracing::{info, instrument, warn};
use kittymc_lib::error::KittyMCError;
use kittymc_lib::packets::client::login::success_02::LoginSuccessPacket;
use kittymc_lib::packets::Packet;
use kittymc_lib::subtypes::state::State;
use log::debug;
use uuid::Uuid;
use kittymc_lib::packets::client::login::disconnect_login_00::DisconnectLoginPacket;
use kittymc_lib::packets::client::play::player_abilities_2c::PlayerAbilitiesPacket;
use kittymc_lib::packets::client::play::plugin_message_18::PluginMessagePacket;
use kittymc_lib::packets::client::play::server_difficulty_0d::ServerDifficultyPacket;
use kittymc_lib::packets::client::login::set_compression_03::SetCompressionPacket;
use kittymc_lib::packets::client::play::chunk_data_20::ChunkDataPacket;
use kittymc_lib::packets::client::play::entity_status_1b::EntityStatusPacket;
use kittymc_lib::packets::client::play::held_item_change_3a::HeldItemChangePacket;
use kittymc_lib::packets::client::play::spawn_position_46::SpawnPositionPacket;
use kittymc_lib::packets::client::play::join_game_23::JoinGamePacket;
use kittymc_lib::packets::client::play::player_list_item_2e::PlayerListItemPacket;
use kittymc_lib::packets::client::play::player_position_and_look_2f::PlayerPositionAndLookPacket;
use kittymc_lib::packets::client::play::time_update_47::TimeUpdatePacket;
use kittymc_lib::packets::client::play::unlock_recipes_31::UnlockRecipesPacket;
use kittymc_lib::packets::client::status::response_00::StatusResponsePacket;
use kittymc_lib::packets::packet_serialization::SerializablePacket;
use crate::client::{Client, ClientInfo};
use crate::player::Player;

#[derive(Debug)]
pub struct KittyMCServer {
    server: TcpListener,
    players: HashMap<Uuid, Player>,
    clients: RwLock<HashMap<Uuid, Client>>,
    registering_clients: VecDeque<Client>,
}

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

    fn handle_client(&self, client: &mut Client) -> Result<bool, KittyMCError> {
        if !client.do_heartbeat()? {
            debug!("[{}] Client didn't respond to heartbeats for too long", client.addr());
           return Ok(false);
        }

        loop {
            let Some(packet) = client.fetch_packet()? else {
                return Ok(true);
            };

            match &packet {
                Packet::KeepAlive(packet) => {
                    client.register_backbeat(packet.id);
                }
                Packet::PluginMessage(msg) if msg.channel == "MC|Brand" => {
                    client.set_brand(String::from_utf8_lossy(&msg.data).to_string())
                }
                _ => ()
            }
        }
    }

    fn get_name_from_uuid(&self, uuid: &Uuid) -> Option<&str> {
        self.players.get(uuid).map(|p| p.name())
    }

    fn send_to_all<P: SerializablePacket + Debug + NamedPacket>(&mut self, packet: &P) -> Result<(), KittyMCError> {
        for client in self.clients.write().unwrap().iter_mut() {
            client.1.send_packet(packet)?;
        }

        Ok(())
    }

    fn handle_client_pre_play(&mut self, client: &mut Client) -> Result<Option<Uuid>, KittyMCError> {
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
                    self.send_to_all(&ChatMessagePacket::new_join_message(self.get_name_from_uuid(&uuid).unwrap()))?;

                    client.send_packet(&PluginMessagePacket::default_brand())?;
                    client.send_packet(&ServerDifficultyPacket::default())?;
                    client.send_packet(&PlayerAbilitiesPacket::default())?;
                    client.send_packet(&HeldItemChangePacket::default())?;
                    client.send_packet(&EntityStatusPacket::default())?;
                    client.send_packet(&UnlockRecipesPacket::default())?;
                    client.send_packet(&PlayerListItemPacket::default())?;
                    // Another Player List Item
                    client.send_packet(&PlayerPositionAndLookPacket::default())?;
                    // World Border
                    client.send_packet(&TimeUpdatePacket::default())?;
                    client.send_packet(&SpawnPositionPacket::default())?;
                    // Player Digging ???
                    // Steer Vehicle ???

                    // after client answers send chunks
                    client.send_packet(&ChunkDataPacket::default())?;

                    return Ok(Some(uuid))
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
                    None => self.registering_clients.push_back(client)
                }
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
                },
                Err(e) => {
                    warn!("[{}] Disconnected client due to error: {e}", client.1.addr());
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