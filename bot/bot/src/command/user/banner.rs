use crate::command::command::{Command, CommandRun};
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::command::user::avatar::{get_user_command, get_user_command_user};
use crate::event_handler::BotData;
use crate::impl_command;
use fluent_templates::fluent_bundle::FluentValue;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{get_language_identifier, Loader, USABLE_LOCALES};
use std::borrow::Cow;
use std::collections::HashMap;

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

		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		let lang_id = get_language_identifier(guild_id, db_connection.clone()).await;

		let banner = match user.banner_url() {
			Some(url) => url,
			None => {
				let embed_content =
					no_banner(&user.name, &lang_id);
				let embed_contents = EmbedsContents::new(CommandType::Followup, embed_content);
				return Ok(embed_contents);
			},
		};

		let username = user.name.as_str();

		let mut args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
		args.insert(Cow::Borrowed("user"), FluentValue::from(username.to_string()));

		let embed_content = EmbedContent::new(USABLE_LOCALES.lookup_with_args(&lang_id, "user_banner-title", &args))
			.images_url(banner);

		let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

		Ok(embed_contents)
	}
);

pub fn no_banner(
	username: &str, lang_id: &unic_langid::LanguageIdentifier,
) -> Vec<EmbedContent> {
	let mut args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
	args.insert(Cow::Borrowed("user"), FluentValue::from(username.to_string()));

	let embed_content = EmbedContent::new(USABLE_LOCALES.lookup(lang_id, "user_banner-no_banner_title"))
		.description(USABLE_LOCALES.lookup_with_args(lang_id, "user_banner-no_banner", &args));

	vec![embed_content]
}
