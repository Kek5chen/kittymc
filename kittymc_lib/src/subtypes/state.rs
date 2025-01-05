#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    Handshake = 0,
    Status = 1,
    Login = 2,
    Transfer = 3,
    Play = 4,
    Undefined = 255,
}

impl From<u32> for State {
    fn from(value: u32) -> Self {
        match value {
            1 => State::Status,
            2 => State::Login,
            3 => State::Transfer,
            4 => State::Play,
            _ => State::Undefined,
        }
    }
}
