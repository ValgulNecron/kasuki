use crate::constant::{COLOR, COMMAND_SENDING_ERROR};
use crate::error_enum::AppError;
use crate::error_enum::AppError::LangageGuildIdError;
use crate::lang_struct::general::credit::load_localization_credit;
use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp,
};

pub async fn run(ctx: &Context, command: &CommandInteraction) -> Result<(), AppError> {
    let guild_id = command
        .guild_id
        .ok_or(LangageGuildIdError(String::from(
            "Guild id for langage not found.",
        )))?
        .to_string();
    let credit_localised = load_localization_credit(guild_id).await?;
    let mut desc: String = "".to_string();
    for x in credit_localised.credits {
        desc += x.desc.as_str()
    }

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .description(desc)
        .title(&credit_localised.title);

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command
        .create_response(&ctx.http, builder)
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())
}
