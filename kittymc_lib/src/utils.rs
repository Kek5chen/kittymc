use crate::error::KittyMCError;
use crate::subtypes::components::TextComponent;
use uuid::{Builder, Uuid};

pub fn generate_cracked_uuid(name: &str) -> Result<Uuid, KittyMCError> {
    if name.len() > 16 {
        return Err(KittyMCError::TooMuchData(name.len(), 16));
    }

    let md5 = md5::compute(format!("OfflinePlayer:{name}"));

    Ok(Builder::from_md5_bytes(md5.0).into_uuid())
}

pub fn is_cool(name: &str) -> bool {
    const COOL_PEOPLE: [&'static str; 3] = ["will_owo", "IT0NA31", "OnlyAfro"];

    COOL_PEOPLE.contains(&name)
}

pub fn rainbowize_cool_people_textcomp(name: &str, bold: bool) -> Option<TextComponent> {
    if !is_cool(name) {
        return None;
    }

    Some(
        TextComponent::builder()
            .text(rainbowize_cool_people(name, bold))
            .build(),
    )
}

pub fn rainbowize_cool_people(name: &str, bold: bool) -> String {
    if is_cool(&name) {
        to_mc_rainbow(name, bold)
    } else {
        name.to_string()
    }
}

pub fn to_mc_rainbow(text: &str, bold: bool) -> String {
    let colors = ["§c", "§6", "§e", "§a", "§b", "§9", "§d"];

    let mut result = String::new();
    for (i, ch) in text.chars().enumerate() {
        let color_code = colors[i % colors.len()];
        result.push_str(color_code);
        if bold {
            result.push_str("§l");
        }
        result.push(ch);
    }

    result
}

#[test]
fn test_cracked_uuid() {
    use std::str::FromStr;

    assert_eq!(
        generate_cracked_uuid("will_owo").unwrap(),
        Uuid::from_str("0e22d127-3477-35f9-a65a-6fb3611c78fb").unwrap()
    );
    assert_eq!(
        generate_cracked_uuid("meow").unwrap(),
        Uuid::from_str("dadfb5ef-c239-3cb3-b316-aec3a76dbc71").unwrap()
    );
    assert_eq!(
        generate_cracked_uuid("IT0NA31").unwrap(),
        Uuid::from_str("fe86cee2-9d18-3100-bc41-6740712ec780").unwrap()
    );
}
