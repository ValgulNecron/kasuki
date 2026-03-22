use anyhow::anyhow;

use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::helper::get_option::command::{get_option_map_string, get_option_map_user};
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext, EntitlementOwner};
use shared::fluent_args;
use shared::localization::{Loader, USABLE_LOCALES};
use small_fixed_array::FixedString;

#[slash_command(
	name = "give_premium_sub", desc = "Give a premium subscription to a user.",
	command_type = GuildChatInput { guild_id = 1117152661620408531 },
	permissions = [Administrator],
	args = [
		(name = "user", desc = "The user to give the subscription to.", arg_type = User, required = true, autocomplete = false),
		(name = "subscription", desc = "The subscription to give.", arg_type = String, required = true, autocomplete = true)
	],
)]
async fn give_premium_sub_command(self_: GivePremiumSubCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);

	let map = get_option_map_user(&cx.command_interaction);

	let user = *map
		.get(&FixedString::from_str_trunc("user"))
		.ok_or(anyhow!("No option for user"))?;

	let map = get_option_map_string(&cx.command_interaction);

	let subscription = map
		.get(&FixedString::from_str_trunc("subscription"))
		.ok_or(anyhow!("No option for subscription"))?
		.clone();

	let skus = cx.ctx.http.get_skus().await?;

	let skus_id: Vec<String> = skus.iter().map(|sku| sku.id.to_string()).collect();

	if !skus_id.contains(&subscription) {
		Err(anyhow!("Invalid sub id"))?
	}

	let mut sku_id = Default::default();

	for sku in skus {
		if sku.id.to_string() == subscription {
			sku_id = sku.id;
		}
	}

	let _ = cx
		.ctx
		.http
		.create_test_entitlement(sku_id, EntitlementOwner::User(user))
		.await?;

	let lang_id = cx.lang_id().await;

	let args = fluent_args!("user" => user.to_string(), "subscription" => subscription.clone());

	let success_msg =
		USABLE_LOCALES.lookup_with_args(&lang_id, "management_give_premium_sub-success", &args);

	let embed_content = EmbedContent::new(String::default()).description(success_msg);

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}
