use regex::Regex;
use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage, Timestamp,
};
use tracing::error;

use crate::constant::COLOR;
use crate::event_handler::{BotData, Handler};

const ERROR_MESSAGE: &str = "**There was an error while processing the command**\
    \n**This error is most likely an input error** \
    **like searching for non existent anime, requesting nsfw image to the ai, etc.**\n \
    **but in some case it's a server or programming error**.\
    **in this case report it to me and I will try to fix it the fastest I can**";

pub async fn command_dispatching(
    message: String,
    command_interaction: &CommandInteraction,
    ctx: &Context,
) {
    error!("{}", message.replace("\\n", "\n"));

    match send_error(message.clone(), command_interaction, ctx).await {
        Ok(_) => {}
        Err(_) => match send_differed_error(message, command_interaction, ctx).await {
            Ok(_) => {}
            Err(e) => {
                error!("{}", e);
            }
        },
    }
}

async fn send_error(
    e: String,
    command_interaction: &CommandInteraction,
    ctx: &Context,
) -> Result<(), String> {
    let error_message = format!("{}\n{}", ERROR_MESSAGE, e);

    // censor url and token in the error message
    let error_message = censor_url_and_token(error_message, ctx);

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .description(error_message)
        .title("There was an error while processing the command");

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|e| format!("{:#?}", e))?;

    Ok(())
}

async fn send_differed_error(
    e: String,
    command_interaction: &CommandInteraction,
    ctx: &Context,
) -> Result<(), String> {
    let error_message = format!("{}\n{}", ERROR_MESSAGE, e);

    // censor url and token in the error message
    let error_message = censor_url_and_token(error_message, ctx);

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .description(error_message)
        .title("There was an error while processing the command");

    let builder = CreateInteractionResponseFollowup::new().embed(builder_embed);

    let _ = command_interaction
        .create_followup(&ctx.http, builder)
        .await
        .map_err(|e| format!("{:#?}", e))?;

    Ok(())
}

fn censor_url_and_token(error_message: String, ctx: &Context) -> String {
    let config = ctx.data::<BotData>().config.clone();

    let mut error_message = error_message;

    let discord_token = config.bot.discord_token.clone();

    let db_user = config.db.user.clone().unwrap_or_default();

    let db_pass = config.db.password.clone().unwrap_or_default();

    let db_port = config.db.port.unwrap_or_default().to_string();

    let db_host = config.db.host.clone().unwrap_or_default();

    let image_token = config.ai.image.ai_image_token.clone().unwrap_or_default();

    let transcript_token = config
        .ai
        .transcription
        .ai_transcription_token
        .clone()
        .unwrap_or_default();

    let chat_token = config
        .ai
        .question
        .ai_question_token
        .clone()
        .unwrap_or_default();

    error_message = error_message
        .replace(&discord_token, "[REDACTED]")
        .replace(&image_token, "[REDACTED]")
        .replace(&transcript_token, "[REDACTED]")
        .replace(&chat_token, "[REDACTED]")
        .replace(&db_user, "[REDACTED]")
        .replace(&db_pass, "[REDACTED]")
        .replace(&db_port, "[REDACTED]")
        .replace(&db_host, "[REDACTED]");

    // replace url with [REDACTED]
    let url_regex = Regex::new(r"https?://(www\.)?[-a-zA-Z0-9@:%._+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_+.~#?&/=]*)").unwrap();

    error_message = url_regex
        .replace_all(&error_message, "[REDACTED]")
        .to_string();

    error_message
}
