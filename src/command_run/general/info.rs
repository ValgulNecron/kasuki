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
    let mut buttons = Vec::new();
    let mut components = Vec::new();

    let button = CreateButton::new_link("https://github.com/ValgulNecron/kasuki")
        .style(ButtonStyle::Primary)
        .label(&info_localised.button_see_on_github);
    buttons.push(button);
    let button = CreateButton::new_link("https://kasuki.valgul.moe/")
        .style(ButtonStyle::Primary)
        .label(&info_localised.button_official_website);
    buttons.push(button);
    let button = CreateButton::new_link("https://discord.gg/h4hYxMURQx")
        .style(ButtonStyle::Primary)
        .label(&info_localised.button_official_discord);
    buttons.push(button);
    let button = CreateButton::new_link("https://discord.com/api/oauth2/authorize?client_id=923286536445894697&permissions=533113194560&scope=bot")
        .style(ButtonStyle::Primary)
        .label(&info_localised.button_add_the_bot);
    buttons.push(button);
    components.push(CreateActionRow::Buttons(buttons));

    let builder_message = CreateInteractionResponseMessage::new()
        .embed(builder_embed)
        .components(components);

    let builder = CreateInteractionResponse::Message(builder_message);

    command
        .create_response(&ctx.http, builder)
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())
}
