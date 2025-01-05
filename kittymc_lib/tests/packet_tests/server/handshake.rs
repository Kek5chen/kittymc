use crate::packet_tests::utils::check_serialized_packet;
use kittymc_lib::packets::packet_serialization::SerializablePacket;
use kittymc_lib::packets::server::handshake::HandshakePacket;
use kittymc_lib::packets::Packet;
use kittymc_lib::subtypes::state::State;

#[test]
fn test_00_handshake_serialize() {
    let handshake = HandshakePacket {
        protocol_version: 47,
        server_address: "meowmc.de".to_string(),
        server_port: 25565,
        next_state: State::Status,
    };

    let serialized = handshake.serialize();
    check_serialized_packet(&serialized, 15, 0, |data| {
        assert_eq!(data[0], 47); // Protocol Version
        assert_eq!(data[1] as usize, b"meowmc.de".len()); // Server Address Length
        assert_eq!(&data[2..11], b"meowmc.de"); // Server Address
    }).unwrap();

    let (len, deserialized_res) = HandshakePacket::deserialize(&serialized[2..]).unwrap();
    assert_eq!(len, serialized.len() - 2, "Length of deserialized size didn't match with serialized packet");
    assert_eq!(deserialized_res, Packet::Handshake(handshake));
}