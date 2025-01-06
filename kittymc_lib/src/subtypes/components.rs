use crate::packets::packet_serialization::write_length_prefixed_string;
use crate::subtypes::Color;
use crate::utils::rainbowize_cool_people;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, TypedBuilder)]
pub struct ClickEvent {
    #[serde(skip_serializing_if = "String::is_empty", default)]
    #[builder(default, setter(into))]
    pub open_url: String,

    #[serde(skip_serializing_if = "String::is_empty", default)]
    #[builder(default, setter(into))]
    pub run_command: String,

    #[serde(skip_serializing_if = "String::is_empty", default)]
    #[builder(default, setter(into))]
    pub suggest_command: String,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[builder(setter(strip_option), default)]
    pub change_page: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, TypedBuilder)]
pub struct HoverEvent {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[builder(setter(into, strip_option), default)]
    pub show_text: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[builder(setter(strip_option), default)]
    pub show_item: Option<()>, // TODO: NBT

    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[builder(setter(strip_option), default)]
    pub show_entity: Option<()>, // TODO: NBT
}

fn is_false(b: &bool) -> bool {
    !*b
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, TypedBuilder, Default)]
pub struct BaseComponent {
    #[serde(skip_serializing_if = "is_false", default)]
    #[builder(default)]
    pub bold: bool,

    #[serde(skip_serializing_if = "is_false", default)]
    #[builder(default)]
    pub italic: bool,

    #[serde(skip_serializing_if = "is_false", default)]
    #[builder(default)]
    pub underlined: bool,

    #[serde(skip_serializing_if = "is_false", default)]
    #[builder(default)]
    pub strikethrough: bool,

    #[serde(skip_serializing_if = "is_false", default)]
    #[builder(default)]
    pub obfuscated: bool,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[builder(setter(strip_option), default)]
    pub color: Option<Color>,

    #[serde(skip_serializing_if = "String::is_empty", default)]
    #[builder(default)]
    pub insertion: String,

    #[serde(skip_serializing_if = "Option::is_none", flatten, default)]
    #[builder(setter(strip_option), default)]
    pub click_event: Option<ClickEvent>,

    #[serde(skip_serializing_if = "Option::is_none", flatten, default)]
    #[builder(setter(strip_option), default)]
    pub hover_event: Option<HoverEvent>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    #[builder(default)]
    pub extra: Vec<Component>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, TypedBuilder)]
pub struct TextComponent {
    #[builder(setter(into), default)]
    pub text: String,
    #[serde(flatten, default)]
    #[builder(default)]
    pub options: BaseComponent,
}

impl TextComponent {
    pub fn write(&self, buffer: &mut Vec<u8>) {
        write_length_prefixed_string(
            buffer,
            &serde_json::to_string(self).unwrap_or_else(|_| "INVALID".to_string()),
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, TypedBuilder)]
pub struct TranslationComponent {
    pub translate: String,
    pub with: Vec<Component>,
}

impl TranslationComponent {
    pub fn write(&self, buffer: &mut Vec<u8>) {
        write_length_prefixed_string(
            buffer,
            &serde_json::to_string(self).unwrap_or_else(|_| "INVALID".to_string()),
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum Component {
    Text(TextComponent),
    Translation(TranslationComponent),
    KeyBind,  // TODO
    Score,    // TODO
    Selector, // TODO
}

const CHAT_TRANSLATION_TAG: &'static str = "chat.type.text";

impl Component {
    pub fn write(&self, buffer: &mut Vec<u8>) {
        write_length_prefixed_string(
            buffer,
            &serde_json::to_string(&self).unwrap_or_else(|_| "INVALID".to_string()),
        );
    }

    pub fn default_join(player: &str) -> Self {
        Self::default_state_message(player, "joined")
    }

    pub fn default_quit(player: &str) -> Self {
        Self::default_state_message(player, "quit")
    }

    pub fn default_state_message(player_name: &str, verb: &str) -> Self {
        let name = rainbowize_cool_people(player_name, true);
        Component::Text(
            TextComponent::builder()
                .text(name)
                .options(
                    BaseComponent::builder()
                        .bold(true)
                        .italic(true)
                        .color(Color::DarkPurple)
                        .extra(vec![Component::Text(
                            TextComponent::builder()
                                .text(format!(" {verb} the game"))
                                .options(BaseComponent::builder().color(Color::Gray).build())
                                .build(),
                        )])
                        .build(),
                )
                .build(),
        )
    }

    pub fn default_chat(player: &str, message: &str) -> Self {
        let name = rainbowize_cool_people(player, true);
        Component::Translation(
            TranslationComponent::builder()
                .translate(CHAT_TRANSLATION_TAG.to_string())
                .with(vec![
                    Component::Text(
                        TextComponent::builder()
                            .text(name)
                            .options(
                                BaseComponent::builder()
                                    .bold(true)
                                    .italic(true)
                                    .color(Color::DarkPurple)
                                    .build(),
                            )
                            .build(),
                    ),
                    Component::Text(
                        TextComponent::builder()
                            .text(message)
                            .options(BaseComponent::builder().color(Color::Gray).build())
                            .build(),
                    ),
                ])
                .build(),
        )
    }
}
