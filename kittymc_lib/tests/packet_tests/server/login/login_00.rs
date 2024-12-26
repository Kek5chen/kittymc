use kittymc_lib::packets::Packet;
use kittymc_lib::packets::packet_serialization::SerializablePacket;
use kittymc_lib::packets::server::login::login_00::LoginStartPacket;
use crate::packet_tests::utils::check_serialized_packet;

#[test]
fn test_00_login_serialize() {
    let packet = LoginStartPacket {
        name: "MeowHD".to_string()
    };

    let serialized = packet.serialize();

    check_serialized_packet(&serialized, 8, 0, |data| {
        assert_eq!(data[0] as usize, b"MeowHD".len()); // Length
        assert_eq!(&data[1..7], b"MeowHD"); // Name
    }).unwrap();

    let (len, deserialized_res) = LoginStartPacket::deserialize(&serialized[2..]).unwrap();
    assert_eq!(len, serialized.len() - 2, "Length of deserialized size didn't match with serialized packet");
    assert_eq!(deserialized_res, Packet::LoginStart(packet));
}
