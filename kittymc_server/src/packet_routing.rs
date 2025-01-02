use uuid::Uuid;
use kittymc_lib::packets::Packet;

pub struct PacketSendInfo {
    sender: Uuid,
    receiver: (Option<Uuid>, Option<String>), // Uuid or Name
    packet: Packet,
}