use poise::{
    serenity_prelude::{self as serenity, Attachment},
    PrefixFrameworkOptions,
};

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
    ctx.reply(response).await?;
    Ok(())
}

/// Command to import a JS/PY file and add it to the bot
#[poise::command(
    slash_command,
    prefix_command,
    subcommands("file", "text"),
    subcommand_required
)]
async fn import(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Subcommand to import a file
#[poise::command(slash_command, prefix_command)]
async fn file(
    ctx: Context<'_>,
    #[description = "File to import"] file: Attachment,
    #[description = "Name of the command"] name: Option<String>,
) -> Result<(), Error> {
    ctx.defer().await?;
    // Check if it is a JS or PY file, if not, return an error
    let file_name = &file.filename;
    let file_extension = file_name.split('.').last().unwrap_or("");
    if file_extension != "js" && file_extension != "py" {
        ctx.reply("Only JS and PY files are supported").await?;
        return Ok(());
    }

    // Check if command name is already in use, aka if a js/py file in the root directory has the same name
    let mut command_name = name
        .clone()
        .unwrap_or_else(|| file_name.to_string())
        .to_lowercase();
    if !command_name.ends_with(".js") && !command_name.ends_with(".py") {
        command_name = format!("{}.{}", command_name, file_extension).to_lowercase();
    }
    if std::fs::metadata(format!("./{}", command_name)).is_ok() {
        ctx.reply(format!("Command {} already exists", command_name))
            .await?;
        return Ok(());
    }

    // Save the file in the root directory
    let file_content = file.download().await?;
    let file_path = format!("./{}", command_name);
    std::fs::write(&file_path, file_content)?;

    // Attempt to run it and get the output, ctx.reply the error and delete the file if it fails
    let output = match std::process::Command::new(if file_extension == "js" {
        "node"
    } else {
        "python3"
    })
    .arg(&file_path)
    .output()
    {
        Ok(output) => {
            if !output.status.success() {
                std::fs::remove_file(&file_path)?;
                ctx.reply(format!(
                    "Error running the file: {}",
                    String::from_utf8_lossy(&output.stderr)
                ))
                .await?;
                return Ok(());
            }
            output
        }
        Err(e) => {
            std::fs::remove_file(&file_path)?;
            ctx.reply(format!("Error running the file: {}", e)).await?;
            return Ok(());
        }
    };

    // If the command was successful, ctx.reply the output
    let output = String::from_utf8_lossy(&output.stdout);
    ctx.reply(format!(
        "Imported file {} as command: `{}`\nOutput: {}",
        file_name, command_name, output
    ))
    .await?;

    Ok(())
}

/// Subcommand to import text encapsulated in triple backticks
#[poise::command(prefix_command)]
async fn text(
    ctx: Context<'_>,
    #[description = "Name of the command"] name: String,
    #[description = "Text to import"] text: poise::CodeBlock,
) -> Result<(), Error> {
    // Check if text.language is js or py
    let file_extension = match text.language.as_deref() {
        Some("js") => "js",
        Some("javascript") => "js",
        Some("py") => "py",
        Some("python") => "py",
        _ => {
            ctx.reply("Only JS and PY files are supported").await?;
            return Ok(());
        }
    };

    // Check if command name is already in use, aka if a js/py file in the root directory has the same name
    let mut command_name = name.clone().to_lowercase();
    if !command_name.ends_with(".js") && !command_name.ends_with(".py") {
        command_name = format!("{}.{}", command_name, file_extension).to_lowercase();
    }
    if std::fs::metadata(format!("./{}", command_name)).is_ok() {
        ctx.reply(format!("Command {} already exists", name))
            .await?;
        return Ok(());
    }

    // Save the file in the root directory
    let file_path = format!("./{}", command_name);
    std::fs::write(&file_path, text.code)?;

    // Attempt to run it and get the output, ctx.reply the error and delete the file if it fails
    let output = match std::process::Command::new(if file_extension == "js" {
        "node"
    } else {
        "python3"
    })
    .arg(&file_path)
    .output()
    {
        Ok(output) => {
            if !output.status.success() {
                std::fs::remove_file(&file_path)?;
                ctx.reply(format!(
                    "Error running the file: {}",
                    String::from_utf8_lossy(&output.stderr)
                ))
                .await?;
                return Ok(());
            }
            output
        }
        Err(e) => {
            std::fs::remove_file(&file_path)?;
            ctx.reply(format!("Error running the file: {}", e)).await?;
            return Ok(());
        }
    };

    // If the command was successful, ctx.reply the output
    let output = String::from_utf8_lossy(&output.stdout);
    ctx.reply(format!(
        "Imported text as command: `{}`\nOutput: {}",
        name, output
    ))
    .await?;

    Ok(())
}

/// Runs local file
#[poise::command(slash_command, prefix_command)]
async fn run(
    ctx: Context<'_>,
    #[description = "Name of the command"] name: String,
) -> Result<(), Error> {
    if !std::fs::metadata(format!("./{}.js", name)).is_ok()
        && !std::fs::metadata(format!("./{}.py", name)).is_ok()
    {
        ctx.reply(format!("Command {} does not exist", name))
            .await?;
        return Ok(());
    }

    let message = ctx.reply(format!("Running command: {}", name)).await?;

    let file_name = if std::fs::metadata(format!("./{}.js", name)).is_ok() {
        format!("./{}.js", name).to_lowercase()
    } else {
        format!("./{}.py", name).to_lowercase()
    };
    let file_extension = file_name.split('.').last().unwrap_or("");

    let output = match std::process::Command::new(if file_extension == "js" {
        "node"
    } else {
        "python3"
    })
    .arg(&file_name)
    .output()
    {
        Ok(output) => {
            if !output.status.success() {
                ctx.reply(format!(
                    "Error running the file: {}",
                    String::from_utf8_lossy(&output.stderr)
                ))
                .await?;
                return Ok(());
            }
            output
        }
        Err(e) => {
            ctx.reply(format!("Error running the file: {}", e)).await?;
            return Ok(());
        }
    };

    message.delete(ctx).await?;
    let output = String::from_utf8_lossy(&output.stdout);
    ctx.reply(format!("```\n{}\n```", output)).await?;

    Ok(())
}

/// Exports file
#[poise::command(slash_command, prefix_command)]
async fn export(
    ctx: Context<'_>,
    #[description = "Name of the command"] name: String,
) -> Result<(), Error> {
    if !std::fs::metadata(format!("./{}.js", name)).is_ok()
        && !std::fs::metadata(format!("./{}.py", name)).is_ok()
    {
        ctx.reply(format!("Command {} does not exist", name))
            .await?;
        return Ok(());
    }

    let file_name = if std::fs::metadata(format!("./{}.js", name)).is_ok() {
        format!("./{}.js", name).to_lowercase()
    } else {
        format!("./{}.py", name).to_lowercase()
    };

    // Send the file as an attachment
    ctx.send(
        poise::CreateReply::default()
            .content(format!("Exporting command: {}", name))
            .attachment(serenity::CreateAttachment::path(&file_name).await?),
    )
    .await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("Failed to load .env file");
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::all();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![age(), import(), export(), run()],
            prefix_options: PrefixFrameworkOptions {
                prefix: Some("!".into()),
                case_insensitive_commands: true,
                edit_tracker: Some(
                    poise::EditTracker::for_timespan(std::time::Duration::from_secs(120)).into(),
                ),
                ..Default::default()
            },
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

    println!("Bot started");
}
