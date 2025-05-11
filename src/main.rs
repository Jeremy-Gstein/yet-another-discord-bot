use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use poise::{serenity_prelude::CreateEmbed, CreateReply, ReplyHandle,};
use poise::serenity_prelude::{self as serenity, CreateAttachment,};
use dotenv::dotenv;
use uuid::Uuid;

pub struct Data {} 
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Execute a command and stream the output while editing an embed message.
#[poise::command(slash_command, install_context = "Guild|User", interaction_context = "Guild|BotDm|PrivateChannel")]
async fn sudo(
    ctx: Context<'_>,
    #[description = "Run a command"] cmd: String,
) -> Result<(), Error> {
    // Use shell to process pipes/redirection (SECURITY NOTE: This executes arbitrary shell commands! See: README.md)
    let mut child = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(&["/C", &cmd])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?
    } else {
        Command::new("su")
            .arg("bday")
            .arg("sh")
            .arg("-c")
            .arg(&cmd)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?
    };

    // Create initial embed
    let initial_embed = CreateEmbed::default()
        .title(format!("Running Command: {}", cmd))
        .description("```sh\n[Command output will appear here]\n```");
    let msg = ctx.send(CreateReply::default().embed(initial_embed)).await?;

    // Read both stdout and stderr
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let mut combined_output = String::new();

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    // Process output streams concurrently
    loop {
        tokio::select! {
            Ok(Some(line)) = stdout_reader.next_line() => {
                handle_line(&mut combined_output, line, &msg, ctx, &cmd).await?;
            }
            Ok(Some(line)) = stderr_reader.next_line() => {
                handle_line(&mut combined_output, format!("[stderr] {}", line), &msg, ctx, &cmd).await?;
            }
            else => break,
        }
    }

    // Handle command completion
    let status = child.wait().await?;
    let final_status = format!(
        "{} (exit code: {})", 
        if status.success() { "✅ Success" } else { "❌ Failed" },
        status.code().unwrap_or(-1)
    );

    let truncated_output = truncate_output(combined_output);
    let final_embed = CreateEmbed::default()
        .title(format!("Finished: {}", cmd))
        .description(format!("```sh\n{}\n```", truncated_output))
        .field("Status", final_status, false);

    msg.edit(ctx, CreateReply::default().embed(final_embed)).await?;
    Ok(())
}

// Helper functions
async fn handle_line(
    output: &mut String,
    line: String,
    msg: &ReplyHandle<'_>,
    ctx: Context<'_>,
    cmd: &str,
) -> Result<(), Error> {
    const MAX_CONTENT_LEN: usize = 4000; // discord max is 4096

    // check if line is already truncated, if it is stop loop
    if output.contains("\n[...truncated...]") {
        return Ok(());
    }

    // check if adding this line would go over MAX_CONTENT_LEN
    if output.len() + line.len() + 1 > MAX_CONTENT_LEN {
        output.push_str("\n[...truncated...]");
        let final_output = truncate_output(output.clone());
        let embed = CreateEmbed::default()
            .title(format!("Running: {}", cmd))
            .description(format!("```sh\n{}\n```", final_output));
        msg.edit(ctx, CreateReply::default().embed(embed)).await?;
        return Ok(());
    }
    
    output.push_str(&line);
    output.push('\n');
    
    let truncated = truncate_output(output.clone());
    let embed = CreateEmbed::default()
        .title(format!("Running: {}", cmd))
        .description(format!("```sh\n{}\n```", truncated));
    
    msg.edit(ctx, CreateReply::default().embed(embed)).await?;
    Ok(())
}

fn truncate_output(mut output: String) -> String {
    const MAX_LEN: usize = 4000; // Reserve space for code block
    if output.len() > MAX_LEN {
        output.truncate(MAX_LEN);
        if !output.eq("\n[...truncated...]") {
            output.push_str("\n[...truncated...]");
        }
    }
    // if we end with a newline, pop it to safe space/formatting
    while output.ends_with('\n') {
        output.pop();
    }
    output
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

/// Download YouTube video as MP3 with preview attachment
#[poise::command(slash_command, install_context = "Guild|User", interaction_context = "Guild|BotDm|PrivateChannel")]
async fn mp3(
    ctx: Context<'_>,
    #[description = "YouTube URL"] url: String,
) -> Result<(), Error> {
    let msg_handle = ctx.send(CreateReply::default()
        .content("⏳ Processing your request...")
        .ephemeral(false))
        .await?;

    let temp_id = Uuid::new_v4();
    let temp_file = format!("/tmp/{}.mp3", temp_id);

    // Download and convert
    let yt_dlp_status = Command::new("yt-dlp")
        .args(&["-x", "--audio-format", "mp3", &url, "-o", &temp_file])
        .status()
        .await?;

    if !yt_dlp_status.success() {
        let _ = tokio::fs::remove_file(&temp_file).await;
        msg_handle.edit(ctx, CreateReply::default()
            .content("❌ Error downloading the video"))
            .await?;
        return Ok(());
    }

    // Upload to R2
    let create_r2_output = Command::new("create-r2.py")
        .arg(&temp_file)
        .output()
        .await?;

    //let _ = tokio::fs::remove_file(&temp_file).await;

    let r2_url = if create_r2_output.status.success() {
        String::from_utf8(create_r2_output.stdout)?.trim().to_string()
    } else {
        msg_handle.edit(ctx, CreateReply::default()
            .content("❌ Error uploading to cloud storage"))
            .await?;
        return Ok(());
    };

    // Prepare preview attachment
    let file_bytes = match tokio::fs::read(&temp_file).await {
        Ok(bytes) => bytes,
        Err(e) => {
            msg_handle.edit(ctx, CreateReply::default()
                .content(format!("🎵 Download ready: {}\n⚠️ Couldn't attach preview: {}", r2_url, e)))
                .await?;
            return Ok(());
        }
    };

    // Send final response with attachment
    const MAX_SIZE: usize = 20 * 1024 * 1024;
    if file_bytes.len() > MAX_SIZE {
        msg_handle.edit(ctx, CreateReply::default()
            .content(format!("🎵 Download ready: {}\n⚠️ File too large for preview attachment", r2_url)))
            .await?;
        } else {
            let attachment = CreateAttachment::bytes(file_bytes, "preview.mp3");
            msg_handle.edit(ctx, CreateReply::default()
                .content(format!("🎵 Download ready: {}", r2_url))
                .attachment(attachment))
                .await?;
    }

    Ok(())
}

#[poise::command(slash_command, install_context = "Guild|User", interaction_context = "Guild|BotDm|PrivateChannel")]
async fn gif(
    ctx: Context<'_>,
    #[description = "YouTube URL (with ?t= timestamp)"] url: String,
    #[description = "Duration in seconds (max 60)"] seconds: u64,
) -> Result<(), Error> {
    let msg_handle = ctx.send(CreateReply::default()
        .content("⏳ Processing your GIF...")
        .ephemeral(false))
    .await?;

    // Extract timestamp from URL and validate duration
    let (clean_url, start_time) = extract_timestamp_from_url(&url);
    let duration = seconds.min(60);
    let end_time = start_time + duration;

    let temp_id = Uuid::new_v4();
    let input_file = format!("/tmp/{}.webm", temp_id);
    let output_file = format!("/tmp/{}.gif", temp_id);

    // Download video segment with yt-dlp
    let yt_dlp_status = Command::new("yt-dlp")
        .args(&[
            "--download-sections",
            &format!("*{}-{}", 
                format_timestamp(start_time),
                format_timestamp(end_time)
            ),
            "--force-keyframes-at-cuts",
            "--output", &input_file,
            "--no-simulate",
            &clean_url,
        ])
        .status()
        .await?;

    if !yt_dlp_status.success() {
        cleanup_files(&[&input_file, &output_file]).await;
        msg_handle.edit(ctx, CreateReply::default()
            .content("❌ Error downloading video segment"))
        .await?;
        return Ok(());
    }

    // Convert to GIF with ffmpeg
    let ffmpeg_status = Command::new("ffmpeg")
        .args(&[
            "-y",
            "-t", &duration.to_string(),
            "-i", &input_file,
            "-vf", "fps=10,scale=640:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse",
            "-loop", "0",
            &output_file
        ])
        .status()
        .await?;

    cleanup_files(&[&input_file]).await;

    if !ffmpeg_status.success() {
        cleanup_files(&[&output_file]).await;
        msg_handle.edit(ctx, CreateReply::default()
            .content("❌ Error converting to GIF"))
        .await?;
        return Ok(());
    }

    // Handle GIF attachment
    match handle_gif_attachment(&output_file).await {
        Ok(attachment) => {
            msg_handle.edit(ctx, CreateReply::default()
                .content("🎥 Here's your GIF!")
                .attachment(attachment))
            .await?
        }
        Err(e) => {
            msg_handle.edit(ctx, CreateReply::default()
                .content(format!("⚠️ {}", e)))
            .await?
        }
    }

    cleanup_files(&[&output_file]).await;
    Ok(())
}

// Helper functions

async fn handle_gif_attachment(path: &str) -> Result<CreateAttachment, String> {
    let gif_bytes = tokio::fs::read(path)
        .await
        .map_err(|e| format!("Error reading GIF: {}", e))?;

    if gif_bytes.len() > 25 * 1024 * 1024 {
        return Err("GIF too large for attachment (max 25MB)".to_string());
    }

    Ok(CreateAttachment::bytes(gif_bytes, "preview.gif"))
}

fn extract_timestamp_from_url(url: &str) -> (String, u64) {
    let mut clean_url = url.to_string();
    let mut timestamp = 0;

    if let Some(pos) = url.find('?') {
        let query = &url[pos+1..];
        for param in query.split('&') {
            if let Some((key, value)) = param.split_once('=') {
                match key {
                    "t" | "start" => {
                        timestamp = parse_youtube_timestamp(value);
                        break;
                    }
                    _ => {}
                }
            }
        }
        clean_url = url[..pos].to_string();
    }

    (clean_url, timestamp)
}

fn parse_youtube_timestamp(input: &str) -> u64 {
    let mut total = 0;
    let mut current = 0;
    
    for c in input.chars() {
        if c.is_ascii_digit() {
            current = current * 10 + c.to_digit(10).unwrap() as u64;
        } else {
            match c {
                'h' => { total += current * 3600; current = 0; }
                'm' => { total += current * 60; current = 0; }
                's' => { total += current; current = 0; }
                _ => {}
            }
        }
    }
    
    total + current
}

fn format_timestamp(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;
    format!("{:01}:{:02}:{:02}", hours, minutes, seconds)
}

async fn cleanup_files(files: &[&str]) {
    for file in files {
        let _ = tokio::fs::remove_file(file).await;
    }
}


#[tokio::main]
async fn main() {
    // pass token via docker run -e TOKEN=as213 
    dotenv().ok();
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    // after we save token to var in app, remove it from os env.
    unsafe {
        std::env::remove_var("DISCORD_TOKEN");
        std::env::remove_var("ACCESS_KEY");
        std::env::remove_var("SECRET_ACCESS_KEY");
        std::env::remove_var("ENDPOINT_URL");
        std::env::remove_var("PUBLIC_DOMAIN");
        std::env::remove_var("BUCKET_NAME");
    }

    let intents = serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![ping(), sudo(), gif(), mp3(),],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data{})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
