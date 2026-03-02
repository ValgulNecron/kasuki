//! WaifuCommand Struct
//!
//! A command used to fetch details about a character from AniList and return it as embed content.
//!
//! ## Fields
//!
//! - `ctx`: The Serenity `Context` allowing access to bot and framework state.
//! - `command_interaction`: The `CommandInteraction` containing data related to the Discord command input.
//!
//! ## Example Usage
//! ```no_run
//! use serenity::all::CommandInteraction;
//! use serenity::all::Context as SerenityContext;
//!
//! let command = WaifuCommand {
//!     ctx: serenity_context,
//!     command_interaction: command_interaction
//! };
//!
//! command.get_contents().await?;
//! ```
//!

use serenity::all::{CommandInteraction, Context as SerenityContext};

use crate::command::anilist_user::character::get_character_by_id;
use crate::command::context::CommandContext;
use crate::structure::run::anilist::character::character_content;
use kasuki_macros::slash_command;

#[slash_command(
	name = "waifu", desc = "Get a random waifu.", command_type = ChatInput,
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
)]
async fn waifu_command(self_: WaifuCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);
	let anilist_cache = cx.anilist_cache.clone();

	// Execute the corresponding search function based on the specified type
	// Fetch the data of the character with ID 156323 from AniList
	let value = 156323;

	let data = get_character_by_id(value, anilist_cache).await?;

	let embed_content = character_content(cx.command_interaction, data, cx.db).await?;

	Ok(embed_content)
}
