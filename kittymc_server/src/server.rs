use crate::client::{Client, ClientInfo};
use crate::player::Player;
use kittymc_lib::error::KittyMCError;
use kittymc_lib::packets::client::login::*;
use kittymc_lib::packets::client::play::animation_06::{AnimationType, ServerAnimationPacket};
use kittymc_lib::packets::client::play::disconnect_1a::DisconnectPlayPacket;
use kittymc_lib::packets::client::play::entity_look_28::EntityLookPacket;
use kittymc_lib::packets::client::play::entity_relative_move_26::EntityRelativeMovePacket;
use kittymc_lib::packets::client::play::player_list_item_2e::PlayerListItemAction;
use kittymc_lib::packets::client::play::*;
use kittymc_lib::packets::client::status::*;
use kittymc_lib::packets::packet_serialization::NamedPacket;
use kittymc_lib::packets::packet_serialization::SerializablePacket;
use kittymc_lib::packets::server::login::LoginStartPacket;
use kittymc_lib::packets::server::play::client_settings_04::Hand;
use kittymc_lib::packets::server::play::player_digging_14::PlayerDiggingStatus;
use kittymc_lib::packets::Packet;
use kittymc_lib::subtypes::metadata::EntityMetadata;
use kittymc_lib::subtypes::state::State;
use kittymc_lib::subtypes::{Direction, Location, Location2};
use kittymc_lib::utils::rainbowize_cool_people_textcomp;
use log::{debug, error};
use rand::random;
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::io::ErrorKind;
use std::net::TcpListener;
use std::sync::{Arc, Mutex, RwLock};
use std::thread::sleep;
use std::time::Duration;
use tracing::{info, instrument, warn};
use uuid::Uuid;
use kittymc_lib::packets::client::play::chunk_data_20::BlockStateId;
use crate::chunking::chunk_manager::ChunkManager;
use crate::inventory::ItemStack;

#[derive(Debug)]
pub struct KittyMCServer {
    server: TcpListener,
    players: HashMap<Uuid, Player>,
    clients: RwLock<HashMap<Uuid, Client>>,
    registering_clients: VecDeque<Client>,
    chunk_manager: RwLock<ChunkManager>,
    next_entity_id: u32,
    shutdown_signal: Arc<Mutex<bool>>,
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
            shutdown_signal: Arc::new(Mutex::new(false)),
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

    fn send_initial_chunks(&mut self, client: &mut Client) -> Result<(), KittyMCError> {
        if client.load_initial_chunks {
            let mut chunk_manager = self.chunk_manager.write().unwrap();
            if client.update_chunks(&Location::new(0., 5., 0.), &mut chunk_manager)? {
                client.load_initial_chunks = false;
            }
        }
        Ok(())
    }

    fn handle_client(&mut self, uuid: &Uuid, client: &mut Client) -> Result<bool, KittyMCError> {
        if !client.do_heartbeat()? {
            debug!(
                "[{}] Client didn't respond to heartbeats for too long",
                client.addr()
            );
            return Ok(false);
        }

        self.send_initial_chunks(client)?;

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
                    {
                        let player = self.players.get_mut(&uuid).unwrap();
                        player.set_position(&packet.location);
                        player.set_direction(&packet.direction);
                    }
                    self.update_global_position(uuid)?;
                    self.update_global_rotation(uuid)?;
                    let mut chunk_manager = self.chunk_manager.write().unwrap();
                    client.update_chunks(&location, &mut chunk_manager)?;
                }
                Packet::PlayerPosition(packet) => {
                    let location = Location::new(
                        packet.location.x as f32,
                        packet.location.y as f32,
                        packet.location.z as f32,
                    );
                    {
                        let player = self.players.get_mut(&uuid).unwrap();
                        player.set_position(&packet.location);
                    }
                    self.update_global_position(uuid)?;
                    let mut chunk_manager = self.chunk_manager.write().unwrap();
                    client.update_chunks(&location, &mut chunk_manager)?;
                }
                Packet::PlayerLook(packet) => {
                    {
                        let player = self.players.get_mut(&uuid).unwrap();
                        player.set_direction(&packet.direction);
                    }
                    self.update_global_rotation(uuid)?;
                }
                Packet::ChatMessage(chat) => {
                    let name = self.get_name_from_uuid(uuid).unwrap_or_else(|| "UNNAMED");
                    let broadcast = ClientChatMessagePacket::new_chat_message(name, &chat.message);
                    info!("<{}> {}", name, chat.message);
                    self.send_to_all(Some(client), &broadcast)?;
                }
                Packet::ClientAnimation(animation) => {
                    let entity_id = self.players.get(uuid).unwrap().id();
                    match animation.hand {
                        Hand::Left => self.send_to_all(
                            None,
                            &ServerAnimationPacket {
                                entity_id,
                                animation: AnimationType::SwingMainArm,
                            },
                        )?,
                        Hand::Right => {}
                        _ => (),
                    }
                }
                Packet::PlayerDigging(digging) => {
                    let game_mode;
                    let _position;
                    let is_cool;
                    {
                        let player = self.players.get(uuid).unwrap();
                        is_cool = player.is_cool();
                        game_mode = player.game_mode();
                        _position = player.position();
                    }

                    // TODO: Range Check

                    if (digging.status == PlayerDiggingStatus::StartedDigging
                        && game_mode == GameMode::Creative
                        || (digging.status == PlayerDiggingStatus::StartedDigging
                            && game_mode != GameMode::Adventure))
                        && is_cool
                    {
                        let loc = digging.location.clone();
                        self.set_block(&loc, 0)?;

                        self.send_to_all(
                            None,
                            &BlockChangePacket::new_empty(loc.clone()),
                        )?;
                        self.send_to_all(
                            None,
                            &BlockBreakAnimationPacket::new(random(), loc, 0x7F),
                        )?;
                    }
                }
                Packet::CreativeInventoryAction(action) => {
                    let player = self.players.get_mut(uuid)
                        .ok_or(KittyMCError::PlayerNotFound)?;

                    let item = if action.clicked_item.id == u16::MAX {
                        None
                    } else {
                        Some(ItemStack {
                            item_id: action.clicked_item.id,
                            damage: action.clicked_item.item_damage,
                            count: action.clicked_item.item_count,
                        })
                    };

                    player.inventory.set_slot(action.slot, item);
                }
                Packet::ClientHeldItemChange(change) => {
                    let player = self.players.get_mut(uuid)
                    .ok_or(KittyMCError::PlayerNotFound)?;

                    player.set_current_slot(change.slot);
                }
                Packet::PlayerBlockPlacement(place) => {
                    let game_mode;
                    let _position;
                    let block;
                    {
                        let player = self.players.get(uuid).unwrap();
                        game_mode = player.game_mode();
                        block = player.inventory.get_slot(player.current_slot() + 36)
                            .ok_or(KittyMCError::InventoryError)?;
                        _position = player.position();
                    }

                    if game_mode == GameMode::Creative {
                        let block_state = ((block.item_id << 4) | (block.damage & 15)) as BlockStateId;
                        let loc = place.location - place.face.as_offset();
                        self.set_block(&loc, block_state)?;

                        self.send_to_all(
                            None,
                            &BlockChangePacket::new(loc, block_state),
                        )?;
                    }
                }
                _ => (),
            }
        }
    }

    pub fn set_block(&mut self, location: &Location, block_state: BlockStateId) -> Result<(), KittyMCError> {
        let mut chunk_manager = self.chunk_manager
            .write()
            .map_err(|_| KittyMCError::LockPoisonError)?;
        chunk_manager.set_block(location, block_state)
    }

    pub fn update_global_position(&mut self, uuid: &Uuid) -> Result<(), KittyMCError> {
        let current_pos;
        let last_pos;
        let entity_id;
        {
            let Some(player) = self.players.get(uuid) else {
                return Ok(());
            };

            current_pos = player.position();
            last_pos = player.last_position();
            entity_id = player.id();
        };
        let relative = current_pos - last_pos;

        if relative.magnitude() > 4.0 {
            // self.send_to_all(
            //     None,
            //     &EntityTeleportPacket {
            //         entity_id,
            //         location: current_pos,
            //         on_ground: false,
            //     },
            // )
            Ok(())
        } else {
            let delta_x = ((current_pos.x * 32. - last_pos.x * 32.) * 128.0).round() as i16;
            let delta_y = ((current_pos.y * 32. - last_pos.y * 32.) * 128.0).round() as i16;
            let delta_z = ((current_pos.z * 32. - last_pos.z * 32.) * 128.0).round() as i16;

            self.send_to_all(
                None,
                &EntityRelativeMovePacket {
                    entity_id,
                    delta_x,
                    delta_y,
                    delta_z,
                    on_ground: false,
                },
            )
        }
    }

    pub fn update_global_rotation(&mut self, uuid: &Uuid) -> Result<(), KittyMCError> {
        let current_rot;
        let last_rot;
        let entity_id;
        {
            let Some(player) = self.players.get(uuid) else {
                return Ok(());
            };

            current_rot = player.direction().clone();
            last_rot = player.last_direction().clone();
            entity_id = player.id();
        };

        if (current_rot.x - last_rot.x).abs() > f32::EPSILON {
            let _ = self.send_to_all(
                None,
                &EntityHeadLookPacket {
                    entity_id,
                    yaw: current_rot.x,
                },
            );
        }

        self.send_to_all(
            None,
            &EntityLookPacket {
                entity_id,
                direction: current_rot,
                on_ground: false,
            },
        )
    }

    fn add_player_to_all_player_lists(
        &mut self,
        client: &mut Client,
        player: &Player,
    ) -> Result<(), KittyMCError> {
        let display_name = rainbowize_cool_people_textcomp(player.name(), true);
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
                        display_name,
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
                metadata: EntityMetadata::default(),
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
            GameMode::Creative,
        );
        let uuid = player.uuid().clone();

        client.set_uuid(uuid.clone());

        let compression = SetCompressionPacket::default();
        client.send_packet(&compression)?;
        client.set_compression(true, compression.threshold);

        client.send_packet(&success)?;
        client.set_state(State::Play);

        client.send_packet(&JoinGamePacket::new(player.id()))?;
        let _ = self.add_player_to_all_player_lists(client, &player);
        self.spawn_player_to_all(&player)?;
        self.players.insert(uuid.clone(), player);

        client.send_packet(&ServerPluginMessagePacket::default_brand())?;
        client.send_packet(&ServerDifficultyPacket::default())?;
        client.send_packet(&PlayerAbilitiesPacket::default())?;
        client.send_packet(&ServerHeldItemChangePacket::default())?;
        client.send_packet(&EntityStatusPacket::default())?;
        client.send_packet(&UnlockRecipesPacket::default())?;
        for player in &self.players {
            client.add_player_to_player_list(player.1)?; // TODO: Add all players in one packet
            client.spawn_player(player.1)?;
        }

        // Another Player List Item
        client.send_packet(&ServerPlayerPositionAndLookPacket::default())?;
        // World Border
        client.send_packet(&TimeUpdatePacket::default())?;
        client.send_packet(&SpawnPositionPacket::default())?;
        // Player Digging ???
        // Steer Vehicle ???

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

                        let name = self.get_name_from_uuid(&uuid).unwrap();
                        info!("{name} joined the game");
                        self.send_to_all(
                            Some(&mut client),
                            &ClientChatMessagePacket::new_join_message(name),
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
                        client.send_packet(&DisconnectPlayPacket::default_error(&e))?;
                        warn!("[{}] Disconnected client due to error: {e}", client.addr());
                    }
                };
                let Some(player) = self.players.remove(&uuid) else {
                    error!("Player shouldn't have been removed yet!");
                    continue;
                };
                info!("{} left the game", player.name());

                let _ = self.remove_player_from_all_player_lists(&mut client, &player);
                let _ = self.send_to_all(
                    Some(&mut client),
                    &ClientChatMessagePacket::new_quit_message(player.name()),
                );
            }
        }

        Ok(())
    }

    pub fn setup_shutdown_signal_handler(&self) {
        let signal_sender = self.shutdown_signal.clone();
        ctrlc::set_handler(move || {
            if *signal_sender.lock().unwrap() {
                std::process::exit(1);
            }
            warn!("Received shutdown signal. Shutting down...");
            *signal_sender.lock().unwrap() = true;
        })
        .expect("Error setting Ctrl-C shutdown handler");
    }

    #[instrument(skip(self))]
    pub fn shutdown(&mut self) {
        for (_, client) in self.clients.write().unwrap().iter_mut() {
            client
                .send_packet(&DisconnectPlayPacket::default_restart())
                .unwrap()
        }
    }

    #[instrument(skip(self))]
    pub fn run(&mut self) -> Result<(), KittyMCError> {
        self.setup_shutdown_signal_handler();
        loop {
            if *self.shutdown_signal.lock().unwrap() {
                info!("Acknowledged shutdown signal. Initiating shut down...");

                self.shutdown();

                info!("Shut down complete. Bye bye!");
                return Ok(());
            }
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
