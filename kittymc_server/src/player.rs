use crate::client::ClientInfo;
use uuid::Uuid;

#[derive(Debug)]
pub struct Player {
    uuid: Uuid,
    username: String,
    entity_id: u32,
}

impl Player {
    pub fn from_client_info(client_info: ClientInfo, id: u32) -> Self {
        Self::new(client_info.uuid, client_info.username, id)
    }

    pub fn new(uuid: Uuid, username: String, id: u32) -> Self {
        Self {
            uuid,
            username,
            entity_id: id,
        }
    }

    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    pub fn name(&self) -> &str {
        &self.username
    }
    pub fn id(&self) -> u32 {
        self.entity_id
    }
}
