use kittymc_lib::subtypes::{
    BaseComponent, ClickEvent, Color, Component, HoverEvent, TextComponent, TranslationComponent,
};
use serde_json::Value;

#[test]
fn test_click_event_serialize_minimal() {
    // Create a minimal ClickEvent with just open_url
    let click_evt = ClickEvent::builder()
        .open_url("https://example.com")
        .build();

    // Convert to JSON
    let json_str = serde_json::to_string(&click_evt).unwrap();
    // Quick JSON check
    let as_json = serde_json::from_str::<serde_json::Value>(&json_str).unwrap();

    // Ensure field is present
    assert_eq!(
        as_json["open_url"],
        serde_json::Value::String("https://example.com".into())
    );
    // Ensure empty fields are omitted
    assert!(as_json.get("run_command").is_none());
    assert!(as_json.get("suggest_command").is_none());
    assert!(as_json.get("change_page").is_none());
}

#[test]
fn test_click_event_deserialize() {
    let incoming = r#"
    {
        "open_url": "https://example.com",
        "run_command": "/hello"
    }
    "#;
    let evt: ClickEvent = serde_json::from_str(incoming).unwrap();
    assert_eq!(evt.open_url, "https://example.com");
    assert_eq!(evt.run_command, "/hello");
    // `change_page` was not provided => should be `None`
    assert!(evt.change_page.is_none());
}

#[test]
fn test_hover_event_round_trip() {
    let hover_evt = HoverEvent::builder().show_text("Hover here").build();
    let serialized = serde_json::to_string(&hover_evt).unwrap();
    let deserialized: HoverEvent = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.show_text, Some("Hover here".to_string()));
}

#[test]
fn test_chat_style_default() {
    // By default, booleans should be false, and Option fields should be None
    let style = BaseComponent::default();
    let serialized = serde_json::to_string(&style).unwrap();
    // Should be an empty object because all fields are default/empty
    assert_eq!(serialized, "{}");
}

#[test]
fn test_chat_style_with_options() {
    let style = BaseComponent::builder()
        .bold(true)
        .italic(true)
        .color(Color::DarkPurple)
        .insertion("InsertMe".to_string())
        .build();

    let serialized = serde_json::to_string(&style).unwrap();
    let as_json = serde_json::from_str::<serde_json::Value>(&serialized).unwrap();

    // Check presence of fields
    assert_eq!(as_json["bold"], true);
    assert_eq!(as_json["italic"], true);
    assert_eq!(as_json["color"], Value::String("dark_purple".to_string()));
    assert_eq!(as_json["insertion"], "InsertMe");
}

#[test]
fn test_text_component_serialize() {
    let text_comp = TextComponent::builder()
        .text("Hello, world!")
        .options(
            BaseComponent::builder()
                .bold(true)
                .color(Color::Gray)
                .build(),
        )
        .build();

    let serialized = serde_json::to_string(&text_comp).unwrap();
    let as_json = serde_json::from_str::<serde_json::Value>(&serialized).unwrap();

    // "text" should be present
    assert_eq!(as_json["text"], "Hello, world!");
    // Style fields should appear at the same level if flattened
    assert_eq!(as_json["bold"], true);
    assert_eq!(as_json["color"], Value::String("gray".to_string()));
}

#[test]
fn test_translation_component_round_trip() {
    let trans_comp = TranslationComponent {
        translate: "chat.type.text".into(),
        with: vec![
            Component::Text(TextComponent::builder().text("Player1").build()),
            Component::Text(TextComponent::builder().text("Hello!").build()),
        ],
    };

    let serialized = serde_json::to_string(&trans_comp).unwrap();
    let deserialized: TranslationComponent = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.translate, "chat.type.text");
    assert_eq!(deserialized.with.len(), 2);
}

#[test]
fn test_component_enum_serialization() {
    let comp_text = Component::Text(TextComponent::builder().text("Just Testing").build());
    let json_text = serde_json::to_string(&comp_text).unwrap();
    // Because of untagged enum, it should look like { "text": "...", "bold": false, ...}
    let as_json = serde_json::from_str::<serde_json::Value>(&json_text).unwrap();
    assert_eq!(as_json["text"], "Just Testing");
}

#[test]
fn test_component_write_length_prefixed_text() {
    let comp_text = Component::Text(TextComponent::builder().text("Prefixed?").build());
    let mut buffer = Vec::new();
    comp_text.write(&mut buffer);

    // The first byte(s) encode the varint length of the JSON,
    // then the JSON data in UTF-8. We won't parse the entire buffer,
    // but let's at least check it's non-empty and the final part contains "Prefixed?".
    assert!(!buffer.is_empty());

    // A quick approach is to skip the varint (since we know its length is small)
    // and check the trailing bytes. The varint might be 1 or 2 bytes depending
    // on the JSON length. Let's just do a rough search:
    let full_str = String::from_utf8_lossy(&buffer);
    assert!(full_str.contains("Prefixed?"));
}

#[test]
fn test_component_default_join() {
    let join_comp = Component::default_join("PlayerXYZ");

    // Should produce a Text component with extra sub-text
    let json_str = serde_json::to_string(&join_comp).unwrap();
    let json_val: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // Because it's untagged, we expect top-level fields like "text", "bold", "italic", etc.
    // Something like: {"text":"PlayerXYZ","bold":true,"italic":true,"color":"DarkPurple","extra":[...]}
    assert_eq!(json_val["text"], "PlayerXYZ");
    assert_eq!(json_val["bold"], true);
    assert_eq!(json_val["italic"], true);
    assert_eq!(json_val["color"], Value::String("dark_purple".to_string()));

    // Check that extra has at least 1 item
    let extra_array = json_val["extra"].as_array().unwrap();
    assert_eq!(extra_array.len(), 1);

    // That item should be a text component with " joined the game"
    let joined_text = &extra_array[0];
    assert_eq!(joined_text["text"], " joined the game");
    assert_eq!(joined_text["color"], Value::String("gray".to_string()));
}
