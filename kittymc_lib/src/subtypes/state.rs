#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    Handshake = 0,
    Status = 1,
    Login = 2,
    Transfer = 3,
    Undefined = 255,
}

impl From<u8> for State {
    fn from(value: u8) -> Self {
        match value {
            1 => State::Status,
            2 => State::Login,
            3 => State::Transfer,
            _ => State::Undefined
        }
    }
}