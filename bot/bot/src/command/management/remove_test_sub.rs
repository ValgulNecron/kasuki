//! The `RemoveTestSubCommand` struct represents a command for removing a user's test subscription.
//!
//! This struct implements the `Command` trait, allowing it to integrate with the bot's command system.
//!
//! # Fields
//! - `ctx`
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::helper::get_option::command::get_option_map_user;
use kasuki_macros::slash_command;
use anyhow::anyhow;
use fluent_templates::fluent_bundle::FluentValue;
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
	CommandInteraction, Context as SerenityContext, CreateInteractionResponseMessage,
};
use shared::localization::{get_language_identifier, Loader, USABLE_LOCALES};
use small_fixed_array::FixedString;
use std::borrow::Cow;
use std::collections::HashMap;
use tracing::error;

#[slash_command(
	name = "remove_test_sub", desc = "Remove premium subscriptions from a user.",
	command_type = GuildChatInput { guild_id = 1117152661620408531 },
	permissions = [Administrator],
	args = [(name = "user", desc = "The user to remove the subscription from.", arg_type = User, required = true, autocomplete = false)],
)]
async fn remove_test_sub_command(self_: RemoveTestSubCommand) -> Result<EmbedsContents<'_>> {
		let ctx = self_.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self_.get_command_interaction();
		let _config = bot_data.config.clone();

		let map = get_option_map_user(command_interaction);

		let user = map.get(&FixedString::from_str_trunc("user"));

		let user = match user {
			Some(user) => user,
			None => {
				return Err(anyhow!("No user provided"));
			},
		};

		let entitlements = ctx
			.http
			.get_entitlements(Some(*user), None, None, None, None, None, None)
			.await?;
		let db_connection = bot_data.db_connection.clone();

		let guild_id = command_interaction.guild_id.unwrap().to_string();
		let lang_id = get_language_identifier(guild_id, db_connection).await;

		// defer the response
		let builder_message = Defer(CreateInteractionResponseMessage::new());

		command_interaction
			.create_response(&ctx.http, builder_message)
			.await?;

		for entitlement in entitlements {
			if let Err(e) = ctx.http.delete_test_entitlement(entitlement.id).await {
				error!("Error while deleting entitlement: {}", e);
			}
		}

		let mut args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
		args.insert(Cow::Borrowed("user"), FluentValue::from(user.to_string()));

		let success_msg = USABLE_LOCALES.lookup_with_args(&lang_id, "management_remove_test_sub-success", &args);

		let embed_content = EmbedContent::new(String::new()).description(success_msg);

		let embed_contents = EmbedsContents::new(CommandType::First, vec![embed_content]);

	Ok(embed_contents)
}
