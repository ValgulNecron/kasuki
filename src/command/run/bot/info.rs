use crate::config::Config;
use crate::constant::{APP_VERSION, LIBRARY};
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::structure::message::bot::info::load_localization_info;
use serenity::all::{
    ButtonStyle, CommandInteraction, Context, CreateActionRow, CreateButton, CreateEmbedFooter,
    CreateInteractionResponse, CreateInteractionResponseMessage,
};
use std::sync::Arc;

/// Executes the command to display the bot's information.
///
/// This function retrieves the localized information strings and formats them into a response to the command interaction.
/// The response includes the bot's name, ID, creation date, avatar, and other details, which are sent as an embed.
/// It also includes buttons for various actions such as visiting the bot's GitHub page, official website, and Discord server.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command executed successfully, or `Err` if an error occurred.
pub async fn run(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
) -> Result<(), AppError> {
    // Retrieve the guild ID from the command interaction
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    // Load the localized information strings
    let info_localised = load_localization_info(guild_id).await?;

    // Retrieve various details about the bot and the server
    let shard_count = ctx.cache.shard_count();
    let shard = ctx.shard_id.to_string();
    let user_count = ctx.cache.user_count();
    let server_count = ctx.cache.guild_count();
    let bot = ctx.http.get_current_application_info().await.map_err(|e| {
        AppError::new(
            format!("Error while getting the bot info {}", e),
            ErrorType::Option,
            ErrorResponseType::Message,
        )
    })?;
    let bot_name = bot.name;
    let bot_id = bot.id.to_string();
    let creation_date = format!("<t:{}:F>", bot.id.created_at().unix_timestamp());

    // Retrieve the bot's avatar
    let bot_icon = bot.icon.ok_or(AppError::new(
        String::from("The bot has no avatar"),
        ErrorType::Option,
        ErrorResponseType::Message,
    ))?;
    let avatar = if bot_icon.is_animated() {
        format!(
            "https://cdn.discordapp.com/icons/{}/{}.gif?size=1024",
            bot_id, bot_icon
        )
    } else {
        format!(
            "https://cdn.discordapp.com/icons/{}/{}.webp?size=1024",
            bot_id, bot_icon
        )
    };

    let lib = LIBRARY.to_string();

    // Construct the embed for the response
    let builder_embed = get_default_embed(None)
        .description(info_localised.desc)
        .field(info_localised.bot_name, bot_name, true)
        .field(info_localised.bot_id, bot_id, true)
        .field(info_localised.version, APP_VERSION, true)
        .field(info_localised.shard_count, shard_count.to_string(), true)
        .field(info_localised.shard, shard, true)
        .field(info_localised.user_count, user_count.to_string(), true)
        .field(info_localised.server_count, server_count.to_string(), true)
        .field(info_localised.creation_date, creation_date, true)
        .field(info_localised.library, lib, true)
        .thumbnail(avatar)
        .title(&info_localised.title)
        .footer(CreateEmbedFooter::new(&info_localised.footer));

    // Initialize the buttons and components for the response
    let mut buttons = Vec::new();
    let mut components = Vec::new();

    // Add buttons for various actions
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
    let button = CreateButton::new_link("https://discord.com/api/oauth2/authorize?client_id=923286536445894697&permissions=395677134144&scope=bot")
        .style(ButtonStyle::Primary)
        .label(&info_localised.button_add_the_bot);
    buttons.push(button);
    let button = CreateButton::new_link("https://discord.com/api/oauth2/authorize?client_id=1122304053620260924&permissions=395677134144&scope=bot")
        .style(ButtonStyle::Primary)
        .label(&info_localised.button_add_the_beta_bot);
    buttons.push(button);
    components.push(CreateActionRow::Buttons(buttons));

    // Construct the message for the response
    let builder_message = CreateInteractionResponseMessage::new()
        .embed(builder_embed)
        .components(components);

    // Construct the response
    let builder = CreateInteractionResponse::Message(builder_message);

    // Send the response to the command interaction
    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {}", e),
                ErrorType::Command,
                ErrorResponseType::Message,
            )
        })
}
