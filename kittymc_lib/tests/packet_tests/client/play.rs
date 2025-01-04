use kittymc_lib::packets::client::login::set_compression_03::SetCompressionPacket;
use kittymc_lib::packets::client::login::success_02::LoginSuccessPacket;
use kittymc_lib::packets::packet_serialization::SerializablePacket;

#[test]
fn test_03_set_compression() {
    let packet = SetCompressionPacket {
        threshold: 256,
    }.serialize();

    assert_eq!(&packet, &[03, 03, 0x80, 02]);
}

#[test]
fn test_02_login_success() {
    let raw_packet = b"0\x00\x02$0e22d127-3477-35f9-a65a-6fb3611c78fb\x08will_owo";
    let packet = LoginSuccessPacket::deserialize(&raw_packet[3..]).unwrap();
    assert_eq!(&packet.1.serialize(), raw_packet) // Here the implementation of integer-encodings varint encode is different than the minecraft one
}