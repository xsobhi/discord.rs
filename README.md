# discord.rs

🚧 **WORK IN PROGRESS** 🚧

> **WARNING**: This library is currently in active development and is **NOT** ready for production use. APIs are subject to change without notice.

A Rust library for interacting with the Discord API.

## Status

This project is currently under heavy construction. Features may be incomplete or missing. Use at your own risk.

## Usage

Add this to your `Cargo.toml`:

```toml
[package]
name = "my_bot"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
tracing-subscriber = "0.3"
discord_rs = { path = "../discord.rs", features = ["gateway_zlib"] } 
```

And this to your `main.rs`:

```rust
use discord_rs::{Client, Intents, MessageBuilder};
use discord_rs::builders::MessageSend;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let token = std::env::var("DISCORD_TOKEN")
        .expect("Missing DISCORD_TOKEN env var");

    // Required for reading msg.content in most guilds
    let intents = Intents::GUILDS | Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT;

    let mut client = Client::new(token).intents(intents);

    client.on_ready(|_ctx, ready| async move {
        println!("Ready as {}", ready.user.username);
        Ok(())
    });

    client.on_message_create(|ctx, msg| async move {
        if msg.content == "!ping" {
            msg.channel_id
                .send(&ctx, MessageBuilder::new().content("Pong!"))
                .await?;
        }
        Ok(())
    });

    client.login().await?;
    Ok(())
}
```
