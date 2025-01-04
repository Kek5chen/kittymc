use crate::error::KittyMCError;
use crate::packets::client::play::{Direction, Location2};
use crate::packets::packet_serialization::{read_bool, read_f32, read_f64, SerializablePacket};
use crate::packets::Packet;
use kittymc_macros::Packet;

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct ClientPlayerPositionAndLookPacket {
    location: Location2, // Feet
    direction: Direction,
    on_ground: bool,
}

impl SerializablePacket for ClientPlayerPositionAndLookPacket {
    fn deserialize(mut data: &[u8]) -> Result<(usize, Packet), KittyMCError> {
        let mut size = 0;

        let location_x = read_f64(&mut data, &mut size)?;
        let location_y = read_f64(&mut data, &mut size)?;
        let location_z = read_f64(&mut data, &mut size)?;
        let yaw = read_f32(&mut data, &mut size)?;
        let pitch = read_f32(&mut data, &mut size)?;
        let on_ground = read_bool(&mut data, &mut size)?;


        Ok((size, Packet::PlayerPositionAndLook(ClientPlayerPositionAndLookPacket {
            location: Location2::new(location_x, location_y, location_z),
            direction: Direction::new(yaw, pitch),
            on_ground
        })))
    }

    fn id() -> u32 {
        0x0E
    }
}
