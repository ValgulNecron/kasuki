use crate::constant::{COLOR, COMMAND_SENDING_ERROR};
use crate::error_enum::AppError;
use crate::error_enum::AppError::LangageGuildIdError;
use crate::structure::general::info::load_localization_info;
use serenity::all::{
    ActionRowComponent, Button, ButtonStyle, CommandInteraction, Context, CreateActionRow,
    CreateButton, CreateEmbed, CreateEmbedFooter, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp,
};

pub async fn run(ctx: &Context, command: &CommandInteraction) -> Result<(), AppError> {
    let guild_id = command
        .guild_id
        .ok_or(LangageGuildIdError(String::from(
            "Guild id for langage not found.",
        )))?
        .to_string();
    let info_localised = load_localization_info(guild_id).await?;

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .description(&info_localised.desc)
        .title(&info_localised.title)
        .footer(CreateEmbedFooter::new(&info_localised.footer));
    let mut components = Vec::new();
    let button = CreateButton::new_link("github")
        .style(ButtonStyle::Primary)
        .label(&info_localised.button_see_on_github);
    components.push(CreateActionRow::from(button));

    let builder_message = CreateInteractionResponseMessage::new()
        .embed(builder_embed)
        .components(components);

    let builder = CreateInteractionResponse::Message(builder_message);

    command
        .create_response(&ctx.http, builder)
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())
}
