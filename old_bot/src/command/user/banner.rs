use crate::command::command::{Command, CommandRun};
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::command::user::avatar::{get_user_command, get_user_command_user};
use crate::event_handler::BotData;
use crate::impl_command;
use crate::structure::message::user::banner::load_localization_banner;
use anyhow::Result;
use sea_orm::DatabaseConnection;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use std::sync::Arc;

#[derive(Clone)]
pub struct BannerCommand {
    pub ctx: SerenityContext,
    pub command_interaction: CommandInteraction,
}

impl_command!(
	for BannerCommand,
	get_contents = |self_: BannerCommand| async move {
		self_.defer().await?;
		let ctx = self_.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self_.get_command_interaction();

		let user = match command_interaction.data.kind.0 {
			1 => get_user_command(ctx, command_interaction).await?,
			2 => get_user_command_user(ctx, command_interaction).await,
			_ => {
				return Err(anyhow::anyhow!("Invalid command type"));
			},
		};

		let db_connection = bot_data.db_connection.clone();

		let banner = match user.banner_url() {
			Some(url) => url,
			None => {
				let embed_content =
					no_banner(command_interaction, &user.name, db_connection).await?;
				let embed_contents = EmbedsContents::new(CommandType::Followup, embed_content);
				return Ok(embed_contents);
			},
		};

		let username = user.name.as_str();

		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		let banner_localised = load_localization_banner(guild_id, db_connection).await?;

		let embed_content = EmbedContent::new(banner_localised.title.replace("$user$", username))
			.images_url(banner);

		let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

		Ok(embed_contents)
	}
);

pub async fn no_banner(
    command_interaction: &CommandInteraction, username: &str,
    db_connection: Arc<DatabaseConnection>,
) -> Result<Vec<EmbedContent>> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let banner_localised = load_localization_banner(guild_id, db_connection).await?;

    let embed_content = EmbedContent::new(banner_localised.no_banner_title)
        .description(banner_localised.no_banner.replace("$user$", username));

    Ok(vec![embed_content])
}
