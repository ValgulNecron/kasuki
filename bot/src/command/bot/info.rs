use crate::command::command_trait::{Command, SlashCommand};
use crate::config::Config;
use crate::constant::{APP_VERSION, LIBRARY};
use crate::get_url;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_dispatch;
use crate::structure::database::prelude::UserColor;
use crate::structure::message::bot::info::load_localization_info;
use sea_orm::EntityTrait;
use serenity::all::{
    ButtonStyle, CommandInteraction, Context, CreateActionRow, CreateButton, CreateEmbedFooter,
    CreateInteractionResponse, CreateInteractionResponseMessage,
};
use std::error::Error;
use std::sync::Arc;

pub struct InfoCommand {
    pub ctx: Context,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
}

impl Command for InfoCommand {
    fn get_ctx(&self) -> &Context {

        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {

        &self.command_interaction
    }
}

impl SlashCommand for InfoCommand {
    async fn run_slash(&self) -> Result<(), Box<dyn Error>> {

        send_embed(&self.ctx, &self.command_interaction, self.config.clone()).await
    }
}

async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
) -> Result<(), Box<dyn Error>> {

    // Retrieve the guild ID from the command interaction
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    // Load the localized information strings
    let info_localised = load_localization_info(guild_id, config.db.clone()).await?;

    // Retrieve various details about the bot and the server
    let shard_count = ctx.cache.shard_count();

    let shard = ctx.shard_id.to_string();

    let connection = sea_orm::Database::connect(get_url(config.db.clone())).await?;

    let user_count = UserColor::find().all(&connection).await?.len();

    let bot = ctx.http.get_current_application_info().await?;

    let bot_name = bot.name;

    let bot_id = bot.id.to_string();

    let creation_date = format!("<t:{}:F>", bot.id.created_at().unix_timestamp());

    let server_count = ctx.cache.guild_count();

    let app_guild_count = bot.approximate_guild_count.unwrap_or_default() as usize;

    let guild_count = if server_count > app_guild_count {

        server_count
    } else {

        app_guild_count
    };

    let app_installation_count = bot.approximate_user_install_count.unwrap_or_default() as usize;

    // Retrieve the bot's avatar
    let bot_icon = bot
        .icon
        .ok_or(error_dispatch::Error::WebRequest("No bot icon".to_string()))?;

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
        .field(info_localised.server_count, guild_count.to_string(), true)
        .field(info_localised.creation_date, creation_date, true)
        .field(info_localised.library, lib, true)
        .field(
            info_localised.app_installation_count,
            app_installation_count.to_string(),
            true,
        )
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
        .await?;

    Ok(())
}
