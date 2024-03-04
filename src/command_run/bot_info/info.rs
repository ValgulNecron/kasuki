use serenity::all::{
    ButtonStyle, CommandInteraction, Context, CreateActionRow, CreateButton, CreateEmbed,
    CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseMessage, Timestamp,
};

use crate::constant::{APP_VERSION, COLOR};
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::lang_struct::general::info::load_localization_info;

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };
    let info_localised = load_localization_info(guild_id).await?;
    let shard_count = ctx.cache.shard_count();
    let shard = ctx.shard_id.to_string();
    let user_count = ctx.cache.user_count();
    let server_count = ctx.cache.guild_count();
    let bot = ctx.http.get_current_application_info().await
        .map_err( |e|
            AppError::new(
                format!("Error while getting the bot info {}", e),
                ErrorType::Option,
                ErrorResponseType::Message,
            )
        )?;
    let bot_name = bot.name;
    let bot_id = bot.id.to_string();
    let creation_date = bot.id.created_at().to_rfc3339().unwrap_or_default();
    let bot_icon = bot.icon.ok_or(
        AppError::new(
            String::from("The bot has no avatar"),
            ErrorType::Option,
            ErrorResponseType::Message,
        )
    )?;
    let avatar = if bot_icon.is_animated() {
        format!("https://cdn.discordapp.com/icons/{}/{}.gif?size=1024", bot_id, bot_icon)
    } else {
        format!("https://cdn.discordapp.com/icons/{}/{}.webp?size=1024", bot_id, bot_icon)
    };

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .description(
            info_localised
                .desc
        )
        .field(info_localised.bot_name, bot_name, true)
        .field(info_localised.bot_id, bot_id, true)
        .field(info_localised.version,APP_VERSION, true)
        .field(info_localised.shard_count, shard_count.to_string(), true)
        .field(info_localised.shard, shard,true)
        .field(info_localised.user_count, user_count.to_string(), true)
        .field(info_localised.server_count, server_count.to_string(), true)
        .field(info_localised.creation_date,creation_date, true)
        .thumbnail(avatar)

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
    let button = CreateButton::new_link("https://discord.com/api/oauth2/authorize?client_id=923286536445894697&permissions=395677134144&scope=bot")
        .style(ButtonStyle::Primary)
        .label(&info_localised.button_add_the_bot);
    buttons.push(button);
    let button = CreateButton::new_link("https://discord.com/api/oauth2/authorize?client_id=1122304053620260924&permissions=395677134144&scope=bot")
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
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {}", e),
                ErrorType::Command,
                ErrorResponseType::Message,
            )
        })
}
