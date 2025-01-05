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
    let _actual_real_raw_packet = b"0\x00\x02$0e22d127-3477-35f9-a65a-6fb3611c78fb\x08will_owo";
    let raw_packet = b"/\x02$0e22d127-3477-35f9-a65a-6fb3611c78fb\x08will_owo";
    // The difference here is that the actual packet serialized its varints badly. It adds an extra 0 after the packet length
    // The raw packet is a slightly modified version since kittymc does not have this encoding error
    // clients accept it fine without the extra 0

    let packet = LoginSuccessPacket::deserialize(&raw_packet[2..]).unwrap();
    assert_eq!(&packet.1.serialize(), raw_packet) // Here the implementation of integer-encodings varint encode is different from the minecraft one
}