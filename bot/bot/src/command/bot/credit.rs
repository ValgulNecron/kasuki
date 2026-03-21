use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use kasuki_macros::slash_command;
use serde::Deserialize;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{Loader, USABLE_LOCALES};
use tracing::error;

#[derive(Deserialize)]
struct GitHubContributor {
	login: String,
	html_url: String,
	contributions: u32,
}

#[slash_command(
	name = "credit", desc = "Get the credit of the app.",
	command_type = SubCommand(parent = "bot"),
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
)]
async fn credit_command(self_: CreditCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);

	let lang_id = cx.lang_id().await;

	let title = USABLE_LOCALES.lookup(&lang_id, "bot_credit-title");
	let desc = USABLE_LOCALES.lookup(&lang_id, "bot_credit-desc");

	let mut fields = Vec::new();

	let contributors = fetch_contributors(&cx.http_client).await;
	if !contributors.is_empty() {
		let contributor_list: String = contributors
			.iter()
			.map(|c| format!("[{}]({}) ({} contributions)", c.login, c.html_url, c.contributions))
			.collect::<Vec<_>>()
			.join("\n");

		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "bot_credit-contributors"),
			contributor_list,
			false,
		));
	}

	let embed_content = EmbedContent::new(title).description(desc).fields(fields);

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}

async fn fetch_contributors(client: &reqwest::Client) -> Vec<GitHubContributor> {
	let url = "https://api.github.com/repos/ValgulNecron/kasuki/contributors";

	match client
		.get(url)
		.header("Accept", "application/vnd.github.v3+json")
		.header("User-Agent", "kasuki-bot")
		.send()
		.await
	{
		Ok(response) => match response.json::<Vec<GitHubContributor>>().await {
			Ok(contributors) => contributors,
			Err(e) => {
				error!("Failed to parse GitHub contributors: {}", e);
				Vec::new()
			},
		},
		Err(e) => {
			error!("Failed to fetch GitHub contributors: {}", e);
			Vec::new()
		},
	}
}
