use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp,
};

use crate::constant::{COLOR, COMMAND_SENDING_ERROR};
use crate::error_enum::AppError;
use crate::lang_struct::general::credit::load_localization_credit;

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };
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

    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())
}
