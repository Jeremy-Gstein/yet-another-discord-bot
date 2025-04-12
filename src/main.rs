use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use poise::{serenity_prelude::CreateEmbed, CreateReply, ReplyHandle};
use poise::serenity_prelude as serenity;

pub struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;


/// Execute a command and stream the output while editing an embed message.
#[poise::command(slash_command, install_context = "Guild|User", interaction_context = "Guild|BotDm|PrivateChannel")]
async fn sudo(
    ctx: Context<'_>,
    #[description = "Run a command"] cmd: String,
) -> Result<(), Error> {
    let mut parts = cmd.split_whitespace();
    let program = parts.next().ok_or("No command provided?")?;
    let args: Vec<&str> = parts.collect();
    // Start the command and capture stdout
    let mut child = Command::new(program)
        .args(&args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();

    // Create the initial embed message
    let initial_embed = CreateEmbed::default()
        .title(format!("Running Command: {}", cmd))
        .description("```sh\n[Command output will appear here]\n```"); // Placeholder text
    let msg = ctx
        .send(CreateReply::default().embed(initial_embed).ephemeral(false)) // Initial message
        .await?;

    let mut output = String::new();
    const MAX_EMBED_LENGTH: usize = 4096 - 10; // Allow space for "```sh\n" and "\n```"

    // Stream and update the embed with output
    while let Some(line) = lines.next_line().await? {
        // Append the new line to the output buffer
        if output.len() + line.len() + 1 > MAX_EMBED_LENGTH {
            // If the embed exceeds the maximum length, truncate it
            output.push_str("\n[...Output truncated...]");
            break;
        }
        output.push_str(&line);
        output.push('\n');

        // Update the embed with the latest output
        let updated_embed = CreateEmbed::default()
            .title(format!("Running Command: {}", cmd))
            .description(format!("```sh\n{}\n```", output));
        msg.edit(ctx, CreateReply::default().embed(updated_embed))
            .await?;
    }

    // Wait for the command to finish and update the embed with the final status
    let status = child.wait().await?;
    let final_status = if status.success() {
        format!("✅ Success (exit code: {})", status.code().unwrap_or(0))
    } else {
        format!("❌ Failed (exit code: {})", status.code().unwrap_or(-1))
    };

    let final_embed = CreateEmbed::default()
        .title(format!("Finished Command: {}", cmd))
        .description(format!("```sh\n{}\n```", output))
        .field("Status:", format!("{}", final_status), true);
    msg.edit(ctx, CreateReply::default().embed(final_embed)).await?;

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
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
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
