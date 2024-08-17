use poise::serenity_prelude as serenity;
use dotenv;
use std::process::Command;





struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}



/// Execute a command.
#[poise::command(slash_command, prefix_command)]
async fn sudo(
    ctx: Context<'_>,
    #[description = "Run a command"] cmd: String,
) -> Result<(), Error> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(&cmd)
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;
    let response = format!("stdout:\n`{}`", stdout);
    ctx.say(response).await?;
    Ok(())
}





/// Mock ping command for your or another user's account.
#[poise::command(slash_command, prefix_command)]
async fn ping(
    ctx: Context<'_>,
    #[description = "Ping"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());

    let response = format!("`PING {} 56(84) bytes of data.\n64 bytes from {}: icmp_seq=1 ttl=52 time=0ms\n--- {} ping statistics ---\n1 packets transmitted, 1 received 0% packet loss, time 0ms`", u.name, u.name, u.name);
    ctx.say(response).await?;
    Ok(())
}



#[tokio::main]
async fn main() {
    let token = dotenv::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![age(), ping(), sudo()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
