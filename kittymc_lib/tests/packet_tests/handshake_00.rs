use kittymc_lib::packets::handshake_00::HandshakePacket;
use kittymc_lib::packets::Packet;
use kittymc_lib::packets::packet_serialization::SerializablePacket;
use kittymc_lib::subtypes::state::State;
use crate::packet_tests::utils::check_serialized_packet;

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
        assert_eq!(data[1] as usize, b"meowmc.de".len()); // Protocol Version
        assert_eq!(&data[2..11], b"meowmc.de");
    }).unwrap();

    let (len, deserialized_res) = HandshakePacket::deserialize(&serialized[2..]).unwrap();
    assert_eq!(len, serialized.len() - 2, "Length of deserialized size didn't match with serialized packet");
    assert_eq!(deserialized_res, Packet::Handshake(handshake));

}