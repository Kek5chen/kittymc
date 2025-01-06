use crate::chunk_manager::ChunkManager;
use crate::client::{Client, ClientInfo};
use crate::player::Player;
use kittymc_lib::error::KittyMCError;
use kittymc_lib::packets::client::login::*;
use kittymc_lib::packets::client::play::player_list_item_2e::PlayerListItemAction;
use kittymc_lib::packets::client::play::*;
use kittymc_lib::packets::client::status::*;
use kittymc_lib::packets::packet_serialization::NamedPacket;
use kittymc_lib::packets::packet_serialization::SerializablePacket;
use kittymc_lib::packets::server::login::LoginStartPacket;
use kittymc_lib::packets::Packet;
use kittymc_lib::subtypes::metadata::PlayerMetadata;
use kittymc_lib::subtypes::state::State;
use kittymc_lib::subtypes::{Direction, Location, Location2};
use log::{debug, error};
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::io::ErrorKind;
use std::net::TcpListener;
use std::sync::RwLock;
use std::thread::sleep;
use std::time::Duration;
use tracing::{info, instrument, warn};
use uuid::Uuid;

#[derive(Debug)]
pub struct KittyMCServer {
    server: TcpListener,
    players: HashMap<Uuid, Player>,
    clients: RwLock<HashMap<Uuid, Client>>,
    registering_clients: VecDeque<Client>,
    chunk_manager: RwLock<ChunkManager>,
    next_entity_id: u32,
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
            chunk_manager: RwLock::new(ChunkManager::new()),
            next_entity_id: 0,
        })
    }

    fn get_name_from_uuid(&self, uuid: &Uuid) -> Option<&str> {
        self.players.get(uuid).map(|p| p.name())
    }

    fn send_to_all<P: SerializablePacket + Debug + NamedPacket>(
        &mut self,
        sender: Option<&mut Client>,
        packet: &P,
    ) -> Result<(), KittyMCError> {
        let mut error = Ok(());

        if let Err(e) = match sender {
            Some(sender) => sender.send_packet(packet),
            None => Ok(()),
        } {
            error = Err(e);
        }
        for client in self.clients.write().unwrap().iter_mut() {
            if let Err(e) = client.1.send_packet(packet) {
                error = Err(e);
            }
        }

        error
    }

    fn handle_client(&mut self, uuid: &Uuid, client: &mut Client) -> Result<bool, KittyMCError> {
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
                Packet::ClientSettings(settings) => {
                    client.set_view_distance(settings.view_distance as u32);
                }
                Packet::PlayerPositionAndLook(packet) => {
                    let location = Location::new(
                        packet.location.x as f32,
                        packet.location.y as f32,
                        packet.location.z as f32,
                    );
                    let mut chunk_manager = self.chunk_manager.write().unwrap();
                    client.update_chunks(&location, &mut chunk_manager)?;
                }
                Packet::PlayerPosition(packet) => {
                    let location = Location::new(
                        packet.location.x as f32,
                        packet.location.y as f32,
                        packet.location.z as f32,
                    );
                    let mut chunk_manager = self.chunk_manager.write().unwrap();
                    client.update_chunks(&location, &mut chunk_manager)?;
                }
                Packet::ChatMessage(chat) => {
                    let name = self.get_name_from_uuid(uuid).unwrap_or_else(|| "UNNAMED");
                    let broadcast = ClientChatMessagePacket::new_chat_message(name, &chat.message);
                    self.send_to_all(Some(client), &broadcast)?;
                }
                _ => (),
            }
        }
    }

    fn add_player_to_all_player_lists(
        &mut self,
        client: &mut Client,
        player: &Player,
    ) -> Result<(), KittyMCError> {
        self.send_to_all(
            Some(client),
            &PlayerListItemPacket {
                actions: vec![(
                    player.uuid().clone(),
                    PlayerListItemAction::AddPlayer {
                        name: player.name().to_string(),
                        properties: vec![],
                        game_mode: GameMode::Survival,
                        ping: 0, // fix ping
                        display_name: None,
                    },
                )],
            },
        )
    }

    fn remove_player_from_all_player_lists(
        &mut self,
        client: &mut Client,
        player: &Player,
    ) -> Result<(), KittyMCError> {
        self.send_to_all(
            Some(client),
            &PlayerListItemPacket {
                actions: vec![(player.uuid().clone(), PlayerListItemAction::RemovePlayer)],
            },
        )
    }

    fn spawn_player_to_all(&mut self, player: &Player) -> Result<(), KittyMCError> {
        self.send_to_all(
            None,
            &SpawnPlayerPacket {
                entity_id: player.id(),
                player_uuid: player.uuid().clone(),
                location: *player.position(),
                direction: *player.direction(),
                metadata: PlayerMetadata::default(),
            },
        )
    }

    fn login_player(
        &mut self,
        client: &mut Client,
        login: &LoginStartPacket,
    ) -> Result<Uuid, KittyMCError> {
        let success = LoginSuccessPacket::from_name_cracked(&login.name)?;
        let client_info = ClientInfo {
            username: success.username.clone(),
            uuid: success.uuid.clone(),
        };

        let player = Player::from_client_info(
            client_info,
            self.get_next_entity_id(),
            &Location2::new(0., 5., 0.),
            &Direction::zeros(),
        );
        let uuid = player.uuid().clone();

        let compression = SetCompressionPacket::default();
        client.send_packet(&compression)?;
        client.set_compression(true, compression.threshold);

        client.send_packet(&success)?;
        client.set_state(State::Play);

        client.send_packet(&JoinGamePacket::new(player.id()))?;
        //self.spawn_player_to_all(&player)?;
        let _ = self.add_player_to_all_player_lists(client, &player);
        self.players.insert(uuid.clone(), player);

        client.send_packet(&ServerPluginMessagePacket::default_brand())?;
        client.send_packet(&ServerDifficultyPacket::default())?;
        client.send_packet(&PlayerAbilitiesPacket::default())?;
        client.send_packet(&ServerHeldItemChangePacket::default())?;
        client.send_packet(&EntityStatusPacket::default())?;
        client.send_packet(&UnlockRecipesPacket::default())?;
        for player in &self.players {
            client.add_player_to_player_list(player.1)?; // TODO: Add all players in one packet
        }

        // Another Player List Item
        client.send_packet(&ServerPlayerPositionAndLookPacket::default())?;
        // World Border
        client.send_packet(&TimeUpdatePacket::default())?;
        client.send_packet(&SpawnPositionPacket::default())?;
        // Player Digging ???
        // Steer Vehicle ???

        // after client answers send chunks

        // FIXME: This is blocking right now. It's GOING TO stall the whole server if chunks
        //   get harder to generate. This should definitely become an asynchronous stated thing.
        let mut chunk_manager = self.chunk_manager.write().unwrap();
        while !client.update_chunks(&Location::new(0., 5., 0.), &mut chunk_manager)? {
            sleep(Duration::from_millis(5))
        }

        Ok(uuid)
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
                    return Ok(Some(self.login_player(client, login)?));
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

                        self.send_to_all(
                            Some(&mut client),
                            &ClientChatMessagePacket::new_join_message(
                                self.get_name_from_uuid(&uuid).unwrap(),
                            ),
                        )?;

                        self.clients.write().unwrap().insert(uuid.clone(), client);
                    }
                    None => self.registering_clients.push_back(client),
                },
            }
        }

        {
            let uuids: Vec<_> = self
                .clients
                .write()
                .unwrap()
                .iter()
                .map(|c| c.0.clone())
                .collect();
            for uuid in uuids {
                let Some(mut client) = self.clients.write().unwrap().remove(&uuid) else {
                    continue;
                };
                match self.handle_client(&uuid, &mut client) {
                    Ok(keep_alive) => {
                        if keep_alive {
                            self.clients.write().unwrap().insert(uuid, client);
                            continue;
                        }
                        info!("[{}] Forced Client disconnect", client.addr());
                    }
                    Err(KittyMCError::Disconnected) => {
                        info!("[{}] Client disconnected", client.addr());
                    }
                    Err(KittyMCError::IoError(e)) if e.kind() == ErrorKind::BrokenPipe => {
                        info!("[{}] Client disconnected", client.addr());
                    }
                    Err(e) => {
                        warn!("[{}] Disconnected client due to error: {e}", client.addr());
                    }
                };
                let Some(player) = self.players.remove(&uuid) else {
                    error!("Player shouldn't have been removed yet!");
                    continue;
                };

                let _ = self.remove_player_from_all_player_lists(&mut client, &player);
                let _ = self.send_to_all(
                    Some(&mut client),
                    &ClientChatMessagePacket::new_quit_message(player.name()),
                );
            }
        }

        Ok(())
    }

    #[instrument(skip(self))]
    pub fn run(&mut self) -> Result<(), KittyMCError> {
        loop {
            // TODO: Monitor if this runs fine
            sleep(Duration::from_millis(1));
            if let Err(e) = self.handle_clients() {
                error!("Client Loop exited early with error: {e}");
            }
        }
    }

    fn get_next_entity_id(&mut self) -> u32 {
        let id = self.next_entity_id;
        self.next_entity_id = self.next_entity_id.wrapping_add(1);
        id
    }
}
