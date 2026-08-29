use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::command::user::avatar::{get_user_command, get_user_command_user};
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::fluent_args;
use shared::localization::{Loader, USABLE_LOCALES};

#[slash_command(
	name = "banner", desc = "Get the banner.",
	command_type = SubCommand(parent = "user"),
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [(name = "username", desc = "Username of the user you want the profile of.", arg_type = User, required = false, autocomplete = false)],
)]
async fn banner_command(self_: BannerCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);

	let user = match cx.command_interaction.data.kind.0 {
		1 => get_user_command(&cx.ctx, &cx.command_interaction).await?,
		2 => get_user_command_user(&cx.ctx, &cx.command_interaction).await,
		_ => {
			return Err(anyhow::anyhow!("Invalid command type"));
		},
	};

	let lang_id = cx.lang_id().await;

	let banner = match user.banner_url() {
		Some(url) => url,
		None => {
			let embed_content = no_banner(&user.name, &lang_id);
			let embed_contents = EmbedsContents::new(embed_content);
			return Ok(embed_contents);
		},
	};

	let username = user.name.as_str();

	let args = fluent_args!("user" => username.to_string());

	let embed_content =
		EmbedContent::new(USABLE_LOCALES.lookup_with_args(&lang_id, "user_banner-title", &args))
			.images_url(banner);

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}

#[slash_command(
	name = "banner",
	command_type = User,
	struct_name = BannerCommand,
	install_contexts = [Guild],
)]
async fn banner_user_command(self_: BannerCommand) -> Result<EmbedsContents<'_>> {
	unreachable!()
}

pub fn no_banner(username: &str, lang_id: &unic_langid::LanguageIdentifier) -> Vec<EmbedContent> {
	let args = fluent_args!("user" => username.to_string());

	let embed_content =
		EmbedContent::new(USABLE_LOCALES.lookup(lang_id, "user_banner-no_banner_title"))
			.description(USABLE_LOCALES.lookup_with_args(lang_id, "user_banner-no_banner", &args));

	vec![embed_content]
}
