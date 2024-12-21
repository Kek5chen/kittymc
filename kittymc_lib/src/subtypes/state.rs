#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    Status = 1,
    Login = 2,
    Undefined = 255,
}

impl From<u8> for State {
    fn from(value: u8) -> Self {
        match value {
            1 => State::Status,
            2 => State::Login,
            _ => State::Undefined
        }
    }
}