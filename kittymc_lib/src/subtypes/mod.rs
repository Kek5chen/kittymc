use derive_builder::Builder;
use serde::{Deserialize, Serialize};

pub mod state;

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum Color {
    Black,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    White,
}

impl Color {
    pub fn as_str(&self) -> &'static str {
        match self {
            Color::Black => "black",
            Color::DarkBlue => "dark_blue",
            Color::DarkGreen => "dark_green",
            Color::DarkAqua => "dark_aqua",
            Color::DarkRed => "dark_red",
            Color::DarkPurple => "dark_purple",
            Color::Gold => "gold",
            Color::Gray => "gray",
            Color::DarkGray => "dark_gray",
            Color::Blue => "blue",
            Color::Green => "green",
            Color::Aqua => "aqua",
            Color::Red => "red",
            Color::LightPurple => "light_purple",
            Color::Yellow => "yellow",
            Color::White => "white",
        }
    }

    pub fn as_color_code(&self) -> &'static str {
        match self {
            Color::Black => "§0",
            Color::DarkBlue => "§1",
            Color::DarkGreen => "§2",
            Color::DarkAqua => "§3",
            Color::DarkRed => "§4",
            Color::DarkPurple => "§5",
            Color::Gold => "§6",
            Color::Gray => "§7",
            Color::DarkGray => "§8",
            Color::Blue => "§9",
            Color::Green => "§a",
            Color::Aqua => "§b",
            Color::Red => "§c",
            Color::LightPurple => "§d",
            Color::Yellow => "§e",
            Color::White => "§f",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Builder)]
pub struct Chat {
    text: String,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    bold: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option), default)]
    italic: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option), default)]
    underlined: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option), default)]
    strikethrough: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option), default)]
    obfuscated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option), default)]
    color: Option<Color>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[builder(setter(custom), default)]
    extra: Vec<Chat>,
}

