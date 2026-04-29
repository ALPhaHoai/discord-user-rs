//! Smoke test that PATCHes Discord's settings-proto endpoint with a real token
//! and verifies the response round-trips through the proto decoder.
//!
//! Usage:
//!   DISCORD_TOKEN="..." cargo run --example set_custom_status_smoke
//!
//! WARNING: This will visibly change your Discord custom status.

use discord_user::operations::StatusOps;
use discord_user::DiscordUser;
use discord_user::UserStatus;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN env var not set");
    let text = std::env::var("STATUS_TEXT").unwrap_or_else(|_| "discord-user-rs smoke test".to_string());

    let client = DiscordUser::new(&token);

    // Auto-clear the custom status 60s from now so it doesn't linger.
    let now_ms =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64;
    let expires_ms = now_ms + 60_000;

    println!("PATCH /users/@me/settings-proto/1");
    println!("  text:       {text:?}");
    println!("  expires_at: {expires_ms} (now + 60s)");

    let resp = client.set_custom_status(UserStatus::Online, Some(&text), Some(expires_ms)).await?;

    println!("\nResponse:");
    println!("  out_of_date:        {}", resp.out_of_date);
    println!("  raw_settings (b64): {} bytes", resp.raw_settings_b64.len());

    let status = resp.settings.status.as_ref().expect("status present");
    println!("  status:             {:?}", status.status);
    println!("  show_current_game:  {:?}", status.show_current_game);
    println!("  status_expires_at:  {:?}", status.status_expires_at_ms);

    if let Some(custom) = status.custom_status.as_ref() {
        println!("  custom_status:");
        println!("    text:        {:?}", custom.text);
        println!("    expires_at:  {:?}", custom.expires_at_ms);
        println!("    created_at:  {:?}", custom.created_at_ms);
        println!("    emoji_id:    {:?}", custom.emoji_id);
        println!("    emoji_name:  {:?}", custom.emoji_name);

        assert_eq!(custom.text, text, "server should echo the text we sent");
        assert_eq!(
            custom.expires_at_ms,
            Some(expires_ms),
            "server should echo the expiry we sent"
        );
        println!("\nOK — server echoed the values we sent, decoder parsed them correctly.");
    } else {
        eprintln!("WARN: server response had no custom_status");
    }

    Ok(())
}
