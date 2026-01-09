use discord_rs::{Client, Intents, MessageBuilder, EmbedBuilder};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Get token from environment
    let token = std::env::var("DISCORD_TOKEN")
        .expect("Expected DISCORD_TOKEN in environment");

    // Create client with intents
    let mut client = Client::new(token)
        .intents(Intents::GUILDS | Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT);

    // Register event handlers
    client.on_ready(|_ctx, ready| async move {
        info!("Logged in as {}#{}", ready.user.username, ready.user.discriminator);
        Ok(())
    });

    client.on_message_create(|ctx, msg| async move {
        if msg.content == "!ping" {
            info!("Received ping from {}", msg.author.username);
            
            let builder = MessageBuilder::new()
                .content("Pong!")
                .add_embed(EmbedBuilder::new()
                    .title("Ping-Pong Stats")
                    .description("Latencies are looking good.")
                    .color(0x00FF00)
                    .build());

            // Use the ergonomic send method from builders trait
            use discord_rs::builders::MessageSend;
            msg.channel_id.send(&ctx, builder).await?;
        }
        Ok(())
    });

    // Start the bot
    client.login().await?;

    Ok(())
}
