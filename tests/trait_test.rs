use discord_user::DiscordUser;

#[tokio::test]
async fn test_trait_api() {
    // This test mainly verifies that the code compiles with the new trait design
    let _user = DiscordUser::new("compilation_test_token");

    // Check if we can access http via context
    // This would require importing DiscordContext too if we called it directly,
    // but MessageOps methods use it internally.

    // We can't easily run this without a real token/network, but compilation is
    // key.
}
