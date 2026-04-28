# discord-user

A Discord self-bot client library for Rust. Connects to Discord as a **user account** (not a bot token) via the WebSocket gateway and REST API.

> **Warning:** Self-botting violates [Discord's Terms of Service](https://discord.com/terms). Use at your own risk.

## Features

- WebSocket gateway connection with automatic reconnection
- Rate-limit-aware HTTP client
- Typed event system with RAII subscription guards
- Message, relationship, channel, guild, and status operations
- Fluent message/embed builder API
- Custom status persistence via Discord's protobuf settings API

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
discord-user-rs = "0.1"
tokio = { version = "1", features = ["full"] }
```

### Connect and listen for messages

```rust
use discord_user::{DiscordUser, UserStatus};

#[tokio::main]
async fn main() -> discord_user::Result<()> {
    let mut client = DiscordUser::new("your-user-token")
        .with_status(UserStatus::Online);

    client.init().await?;

    // Listen for incoming messages
    let _sub = client.on_message_create(|event| {
        println!("[{}] {}: {}", event.channel_id, event.author.username, event.content);
    }).await;

    // Keep alive
    tokio::signal::ctrl_c().await.unwrap();
    client.disconnect().await;
    Ok(())
}
```

### Send a message

```rust
use discord_user::{DiscordUser, UserStatus};
use discord_user::operations::MessageOps;

let mut client = DiscordUser::new(token).with_status(UserStatus::Online);
client.init().await?;

// Simple send
client.send_message(&channel_id, "Hello!", None).await?;

// Fluent builder with embed
client.message()
    .channel("123456789")
    .content("Check this out")
    .embed(|e| e
        .title("My Embed")
        .description("Some description")
        .color(0x00FF00))
    .send()
    .await?;
```

### Manage relationships

```rust
use discord_user::operations::RelationshipOps;

// Get friend list
let relationships = client.get_my_relationship().await?;

// Send friend request
client.send_friend_request_by_username("username", None).await?;

// Accept a request
client.accept_friend_request(&user_id).await?;
```

### Set status

```rust
use discord_user::{UserStatus, operations::StatusOps};

// Online/Idle/DnD/Invisible
client.set_status(UserStatus::DoNotDisturb).await?;

// Custom status text (persisted via protobuf settings API)
client.set_custom_status(UserStatus::Online, Some("Working"), None).await?;

// Clear custom status
client.clear_custom_status().await?;
```

### Builder pattern

```rust
use discord_user::{DiscordUserBuilder, UserStatus};

let mut client = DiscordUserBuilder::new()
    .token("your-token")
    .status(UserStatus::Invisible)
    .max_reconnect_attempts(10)
    .event_buffer_size(512)
    .build()?;

client.init().await?;
```

## Event System

Events are dispatched from the gateway and consumed via typed handlers. Each `on_*` call returns an `EventSubscription` that auto-unsubscribes on drop.

| Method | Event |
|---|---|
| `on_message_create` | `MESSAGE_CREATE` |
| `on_message_update` | `MESSAGE_UPDATE` |
| `on_message_delete` | `MESSAGE_DELETE` |
| `on_typing_start` | `TYPING_START` |
| `on_presence_update` | `PRESENCE_UPDATE` |
| `on_relationship_add` | `RELATIONSHIP_ADD` |
| `on_reaction_add` | `MESSAGE_REACTION_ADD` |
| `on_voice_state_update` | `VOICE_STATE_UPDATE` |
| `on_channel_create/update/delete` | `CHANNEL_*` |
| `on_guild_create/update/delete` | `GUILD_*` |
| `on_guild_member_add` | `GUILD_MEMBER_ADD` |
| `on_user_update` | `USER_UPDATE` |
| `on_interaction_create` | `INTERACTION_CREATE` |
| `on_typed_event` | All events via `TypedEvent` enum |

## Operations

All operation traits are automatically implemented for any type implementing `DiscordContext`.

| Trait | Import |
|---|---|
| `MessageOps` | `discord_user::operations::MessageOps` |
| `RelationshipOps` | `discord_user::operations::RelationshipOps` |
| `ChannelOps` | `discord_user::operations::ChannelOps` |
| `GuildOps` | `discord_user::operations::GuildOps` |
| `StatusOps` | `discord_user::operations::StatusOps` |

## Notes

- **Gateway:** Discord API v9, gateway `wss://gateway-us-east1-c.discord.gg`. The hardcoded client build number must be updated periodically when Discord pushes new client versions.
- **HTTP:** Parses `X-RateLimit-*` headers and retries up to 5 times with backoff. User-Agent mimics Chrome.
- **Logging:** Uses the `tracing` crate. Initialize a subscriber (e.g. `tracing-subscriber`) to see logs.
