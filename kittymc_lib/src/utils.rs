use crate::error::KittyMCError;
use uuid::{Builder, Uuid};

pub fn generate_cracked_uuid(name: &str) -> Result<Uuid, KittyMCError> {
    if name.len() > 16 {
        return Err(KittyMCError::TooMuchData(name.len(), 16));
    }

    let md5 = md5::compute(format!("OfflinePlayer:{name}"));

    Ok(Builder::from_md5_bytes(md5.0).into_uuid())
}

#[test]
fn test_cracked_uuid() {
    use std::str::FromStr;

    assert_eq!(generate_cracked_uuid("will_owo").unwrap(), Uuid::from_str("0e22d127-3477-35f9-a65a-6fb3611c78fb").unwrap());
    assert_eq!(generate_cracked_uuid("meow").unwrap(), Uuid::from_str("dadfb5ef-c239-3cb3-b316-aec3a76dbc71").unwrap());
    assert_eq!(generate_cracked_uuid("IT0NA31").unwrap(), Uuid::from_str("fe86cee2-9d18-3100-bc41-6740712ec780").unwrap());
}

