use poise::serenity_prelude as serenity;
use poise::serenity_prelude::CreateEmbed;
use poise::{CreateReply, ReplyHandle};
use std::process::Command;

pub struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;


/// Execute a command.
#[poise::command(slash_command, install_context = "Guild|User", interaction_context = "Guild|BotDm|PrivateChannel")]
async fn sudo(
    ctx: Context<'_>,
    #[description = "Run a command"] cmd: String,
) -> Result<(), Error> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(&cmd)
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;
    let response = format!("```sh\n{}\n```", stdout);
    ctx.say(response).await?;
    Ok(())
}

/// Mock ping command for your or another user's account.
#[poise::command(slash_command, install_context = "Guild|User", interaction_context = "Guild|BotDm|PrivateChannel")]
async fn ping(
    ctx: Context<'_>,
    #[description = "ping"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());

    let response = format!("PING {} 56(84) bytes of data.\n64 bytes from {}: icmp_seq=1 ttl=52 time=0ms\n\t\t\t--- {}'s ping statistics ---\n1 packets transmitted, 1 received 0% packet loss, time 0ms", u.name, u.name, u.name);
    let ping_embed = CreateEmbed::default()
        // tested sh, bash, shell, and powershell.. haskell had more code coverage on syntax
        // highlighting
        .description(format!("```haskell\n{}\n```", response));
    let ping_msg = ctx.send(CreateReply::default().embed(ping_embed)).await?;


    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    let embed = CreateEmbed::default()
        .title("Message will be automatically deleted in 30 seconds...");
    let msg_embed = ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;

    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    msg_embed.delete(ctx).await?; // same as method below
    ReplyHandle::delete(&ping_msg, ctx)
        .await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    // pass token via docker run -e TOKEN=as213 
    let token = dotenv::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    // after we save token to var in app, remove it from os env.
    unsafe {
        std::env::remove_var("DISCORD_TOKEN");
    }

    let intents = serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![ping(), sudo()],
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
