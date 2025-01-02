use uuid::Uuid;
use crate::client::ClientInfo;

#[derive(Debug)]
pub struct Player {
    uuid: Uuid,
    username: String,
}

impl Player {
    pub fn from_client_info(client_info: ClientInfo) -> Self {
        Self::new(client_info.uuid, client_info.username)
    }

    pub fn new(uuid: Uuid, username: String) -> Self {
        Self {
            uuid,
            username,
        }
    }

    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    pub fn name(&self) -> &str {
        &self.username
    }
}