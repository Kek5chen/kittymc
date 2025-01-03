use kittymc_lib::subtypes::ChatBuilder;

#[test]
fn test_chat_serialization() {
    let chat = ChatBuilder::default()
        .text("Meow".to_string())
        .build()
        .unwrap();
    let json = serde_json::to_string(&chat).unwrap();

    assert_eq!(r#"{"text":"Meow"}"#, json);
}