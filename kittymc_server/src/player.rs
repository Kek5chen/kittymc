use crate::client::ClientInfo;
use kittymc_lib::packets::client::play::GameMode;
use kittymc_lib::subtypes::{Direction, Location2};
use uuid::Uuid;
use kittymc_lib::subtypes::metadata::{EntityMetaState, PlayerMetadata};
use kittymc_lib::utils::is_cool;
use crate::inventory::Inventory;

#[derive(Debug)]
pub struct Player {
    uuid: Uuid,
    username: String,
    entity_id: i32,
    position: Location2,
    direction: Direction,
    last_position: Location2,
    last_direction: Direction,
    game_mode: GameMode,
    pub inventory: Inventory,
    current_slot: i16,
    state: PlayerMetadata,
}

#[allow(dead_code)]
impl Player {
    pub fn from_client_info(
        client_info: ClientInfo,
        id: i32,
        position: &Location2,
        direction: &Direction,
        game_mode: GameMode,
    ) -> Self {
        Self::new(
            client_info.uuid,
            client_info.username,
            id,
            position,
            direction,
            game_mode,
        )
    }

    pub fn new(
        uuid: Uuid,
        username: String,
        id: i32,
        position: &Location2,
        direction: &Direction,
        game_mode: GameMode,
    ) -> Self {
        Self {
            uuid,
            username,
            entity_id: id,
            position: *position,
            direction: *direction,
            last_position: *position,
            last_direction: *direction,
            game_mode,
            inventory: Inventory::new(),
            current_slot: 0,
            state: Default::default(),
        }
    }

    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    pub fn name(&self) -> &str {
        &self.username
    }

    pub fn id(&self) -> i32 {
        self.entity_id
    }

    pub fn position(&self) -> &Location2 {
        &self.position
    }

    pub fn set_position(&mut self, position: &Location2) {
        self.last_position = self.position;
        self.position = *position;
    }

    pub fn direction(&self) -> &Direction {
        &self.direction
    }

    pub fn set_direction(&mut self, direction: &Direction) {
        self.last_direction = self.direction;
        self.direction = *direction;
    }

    pub fn last_position(&self) -> &Location2 {
        &self.last_position
    }

    pub fn last_direction(&self) -> &Direction {
        &self.last_direction
    }

    pub fn game_mode(&self) -> GameMode {
        self.game_mode
    }

    pub fn set_game_mode(&mut self, game_mode: GameMode) {
        self.game_mode = game_mode;
    }

    pub fn set_current_slot(&mut self, slot: i16) {
        self.current_slot = slot;
    }

    pub fn current_slot(&self) -> i16 {
        self.current_slot
    }

    pub fn current_hotbar_slot(&self) -> i16 {
        self.current_slot + 36
    }

    pub fn is_cool(&self) -> bool {
        is_cool(&self.username)
    }

    pub fn get_state(&self) -> &PlayerMetadata {
        &self.state
    }

    pub fn set_crouching(&mut self, is_crouching: bool) {
        self.state.living.entity.meta_state.set(EntityMetaState::crouched, is_crouching);
    }

    pub fn set_sprinting(&mut self, is_sprinting: bool) {
        self.state.living.entity.meta_state.set(EntityMetaState::sprinting, is_sprinting);
    }
}
