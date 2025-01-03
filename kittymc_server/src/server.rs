use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::net::TcpListener;
use std::sync::RwLock;
use tracing::{info, instrument, warn};
use kittymc_lib::error::KittyMCError;
use kittymc_lib::packets::client::login::success_02::LoginSuccessPacket;
use kittymc_lib::packets::client::status::response_00::StatusResponsePacket;
use kittymc_lib::packets::Packet;
use kittymc_lib::subtypes::state::State;
use log::debug;
use uuid::Uuid;
use kittymc_lib::packets::client::play::chat_message_02::ChatMessagePacket;
use kittymc_lib::packets::client::play::player_abilities_39::PlayerAbilitiesPacket;
use kittymc_lib::packets::client::play::plugin_message_3f::PluginMessagePacket;
use kittymc_lib::packets::client::play::server_difficulty_41::ServerDifficultyPacket;
use kittymc_lib::packets::client::login::set_compression_03::SetCompressionPacket;
use kittymc_lib::packets::client::play::spawn_position_05::SpawnPositionPacket;
use kittymc_lib::packets::client::play::join_game_01::JoinGamePacket;
use kittymc_lib::packets::client::play::player_position_and_look::PlayerPositionAndLookPacket;
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
            debug!("Client didn't respond to heartbeats for too long");
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

    fn send_to_all<P: SerializablePacket + Debug>(&mut self, packet: &P) -> Result<(), KittyMCError> {
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
                    if handshake.protocol_version != 47 {
                        warn!("[{}] Client tried to connect with protocol version {} != 47. Disconnecting.", client.addr(), handshake.protocol_version);
                        return Err(KittyMCError::VersionMissmatch);
                    }
                    client.set_state(handshake.next_state);
                }
                Packet::StatusRequest => {
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

                    client.send_packet(&SetCompressionPacket::default())?;
                    client.set_compression(true);

                    client.send_packet(&success)?;
                    client.set_state(State::Play);

                    client.send_packet(&JoinGamePacket::default())?;
                    self.send_to_all(&ChatMessagePacket::new_join_message(self.get_name_from_uuid(&uuid).unwrap()))?;

                    client.send_packet(&PluginMessagePacket::default_brand())?;
                    client.send_packet(&ServerDifficultyPacket::default())?;
                    client.send_packet(&SpawnPositionPacket::default())?;
                    client.send_packet(&PlayerAbilitiesPacket::default())?;
                    client.send_packet(&PlayerPositionAndLookPacket::default())?;

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
                    info!("Registering Client disconnected ({e})");
                    continue;
                }
                Ok(opt_uuid) => match opt_uuid {
                    Some(uuid) => {
                        self.clients.write().unwrap().insert(uuid, client);
                        debug!("Client successfully registered");
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
                        info!("Forced Client disconnect");
                        disconnect_uuids.push(client.0.clone());
                    }
                }
                Err(KittyMCError::Disconnected) => {
                    info!("Client disconnected");
                    disconnect_uuids.push(client.0.clone())
                },
                Err(e) => {
                    warn!("Disconnected client due to error: {e}");
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