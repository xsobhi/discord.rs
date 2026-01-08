use discord_rs_model::{User, Snowflake};
use serde_json::json;

#[test]
fn test_user_deserialization() {
    let json = json!({
        "id": "80351110224678912",
        "username": "Nelly",
        "discriminator": "1337",
        "avatar": "8342729096ea3675442027381ff50dfe",
        "verified": true,
        "email": "nelly@discord.com",
        "flags": 64,
        "premium_type": 1,
        "public_flags": 64
    });

    let user: User = serde_json::from_value(json).unwrap();
    
    assert_eq!(user.id, Snowflake(80351110224678912));
    assert_eq!(user.username, "Nelly");
    assert_eq!(user.discriminator, "1337");
    assert_eq!(user.bot, false); // default
}

#[test]
fn test_snowflake_serialization() {
    let id = Snowflake(123456789);
    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(json, "\"123456789\"");
}
