use serenity::all::{
    ButtonStyle, CommandInteraction, Context, CreateActionRow, CreateButton, CreateEmbed,
    CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseMessage, Timestamp,
};

use crate::constant::{COLOR, COMMAND_SENDING_ERROR, VERSION};
use crate::error_enum::AppError;
use crate::lang_struct::general::info::load_localization_info;

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };
    let info_localised = load_localization_info(guild_id).await?;

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .description(
            info_localised
                .desc
                .replace("$number$", ctx.cache.guilds().len().to_string().as_str())
                .replace("$version$", VERSION),
        )
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
    components.push(CreateActionRow::Buttons(buttons.clone()));
    buttons.clear();
    let button = CreateButton::new_link("https://discord.com/api/oauth2/authorize?client_id=923286536445894697&permissions=395677117760&scope=bot")
        .style(ButtonStyle::Primary)
        .label(&info_localised.button_add_the_bot);
    buttons.push(button);
    let button = CreateButton::new_link("https://discord.com/api/oauth2/authorize?client_id=1122304053620260924&permissions=395677117760&scope=bot")
        .style(ButtonStyle::Primary)
        .label(&info_localised.button_add_the_beta_bot);
    buttons.push(button);
    components.push(CreateActionRow::Buttons(buttons));

    let builder_message = CreateInteractionResponseMessage::new()
        .embed(builder_embed)
        .components(components);

    let builder = CreateInteractionResponse::Message(builder_message);

    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())
}
