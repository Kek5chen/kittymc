use std::collections::{HashMap, VecDeque};
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
use kittymc_lib::packets::client::login::set_compression_03::SetCompressionPacket;
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

        Ok(true)
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