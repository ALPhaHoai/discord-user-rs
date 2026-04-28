use discord_user::DiscordUser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use discord_user::operations::RelationshipOps;

    // Initialize tracing subscriber to see logs
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    let token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN env var not set");
    println!("Initializing DiscordUser...");

    let client = DiscordUser::new(&token);

    // Fetch user data first to verify token and identification
    match client.fetch_user_data().await {
        Ok(user) => println!("Fetched user: {} ({})", user.username, user.id),
        Err(e) => eprintln!("Failed to fetch user data: {}", e),
    }

    // Fetch and print relationships via HTTP
    println!("Fetching relationships via HTTP...");
    match client.get_my_relationship().await {
        Ok(relationships) => {
            println!("Successfully fetched {} relationships:", relationships.len());

            for rel in relationships {
                let username = rel.user.as_ref().map(|u| u.username.as_str()).unwrap_or_else(|| rel.id.as_str());
                println!("- {} (Type: {:?})", username, rel.relationship_type);
            }
        }
        Err(e) => eprintln!("Failed to fetch relationships: {}", e),
    }

    Ok(())
}
