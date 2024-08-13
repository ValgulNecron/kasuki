use std::error::Error;
use std::sync::Arc;

use crate::command::command_trait::{Command, SlashCommand, UserCommand};
use crate::command::user::avatar::{get_user_command, get_user_command_user};
use crate::config::{BotConfigDetails, Config};
use crate::constant::COLOR;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::ResponseError;
use crate::structure::message::user::banner::load_localization_banner;
use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp, User,
};

pub struct BannerCommand {
    pub ctx: Context,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
}

impl Command for BannerCommand {
    fn get_ctx(&self) -> &Context {
        &self.ctx
    }
    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for BannerCommand {
    async fn run_slash(&self) -> Result<(), Box<dyn Error>> {
        let user = get_user_command(&self.ctx, &self.command_interaction).await?;
        send_embed(&self.ctx, &self.command_interaction, user, &self.config).await
    }
}

impl UserCommand for BannerCommand {
    async fn run_user(&self) -> Result<(), Box<dyn Error>> {
        let user = get_user_command_user(&self.ctx, &self.command_interaction);
        send_embed(
            &self.ctx,
            &self.command_interaction,
            user.await,
            &self.config,
        )
        .await
    }
}

pub async fn no_banner(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    username: &str,
    db_type: String,
    db_config: BotConfigDetails,
) -> Result<(), Box<dyn Error>> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };
    let banner_localised = load_localization_banner(guild_id, db_type, db_config).await?;

    let builder_embed = get_default_embed(None)
        .description(banner_localised.no_banner.replace("$user$", username))
        .title(&banner_localised.no_banner_title);

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|e| ResponseError::Sending(format!("{:#?}", e)))?;
    Ok(())
}
pub async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    user: User,
    config: &Arc<Config>,
) -> Result<(), Box<dyn Error>> {
    let db_type = config.bot.config.db_type.clone();
    let db_config = config.bot.config.clone();
    let banner = match user.banner_url() {
        Some(url) => url,
        None => {
            no_banner(&ctx, &command_interaction, &user.name, db_type, db_config).await?;
            return Ok(());
        }
    };
    let username = user.name.as_str();
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };
    let banner_localised = load_localization_banner(guild_id, db_type, db_config).await?;

    let builder_embed = get_default_embed(None)
        .image(banner)
        .title(banner_localised.title.replace("$user$", username));

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|e| ResponseError::Sending(format!("{:#?}", e)))?;
    Ok(())
}
