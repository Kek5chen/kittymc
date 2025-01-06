use crate::client::ClientInfo;
use kittymc_lib::subtypes::{Direction, Location2};
use uuid::Uuid;

#[derive(Debug)]
pub struct Player {
    uuid: Uuid,
    username: String,
    entity_id: u32,
    position: Location2,
    direction: Direction,
}

impl Player {
    pub fn from_client_info(
        client_info: ClientInfo,
        id: u32,
        position: &Location2,
        direction: &Direction,
    ) -> Self {
        Self::new(
            client_info.uuid,
            client_info.username,
            id,
            position,
            direction,
        )
    }

    pub fn new(
        uuid: Uuid,
        username: String,
        id: u32,
        position: &Location2,
        direction: &Direction,
    ) -> Self {
        Self {
            uuid,
            username,
            entity_id: id,
            position: *position,
            direction: *direction,
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

    pub fn position(&self) -> &Location2 {
        &self.position
    }

    pub fn direction(&self) -> &Direction {
        &self.direction
    }
}
