use anyhow::format_err;
use integer_encoding::VarInt;

pub fn check_serialized_packet(
    actual_data: &[u8],
    len: usize,
    packet_id: u32,
    other: fn(data: &[u8]),
) -> anyhow::Result<()> {
    let mut data = actual_data;
    let (actual_len, size): (usize, usize) =
        VarInt::decode_var(data).ok_or_else(|| format_err!("Packet Length not valid"))?;
    assert_eq!(
        actual_len, len,
        "Serialized Packet length doesn't match hardcoded Packet Length"
    );
    assert_eq!(
        actual_data.len(),
        len + 1,
        "Actual Packet length doesn't match hardcoded Packet Length"
    );
    assert_eq!(
        actual_len,
        actual_data.len() - 1,
        "Serialized Packet length doesn't match actual Packet length"
    );
    data = &data[size..];

    let (actual_packet_id, size): (u32, usize) =
        VarInt::decode_var(data).ok_or_else(|| format_err!("Packet ID not valid"))?;
    assert_eq!(actual_packet_id, packet_id, "Packet ID doesn't match");
    data = &data[size..];

    other(data);

    Ok(())
}
