use crate::event_handler::BotData;
use sea_orm::DatabaseConnection;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::cache::CacheInterface;
use shared::helper::get_guild_lang::get_guild_language;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use unic_langid::LanguageIdentifier;

/// A convenience wrapper that pre-extracts the most common state accessed by every command.
///
/// Create with [`CommandContext::new`] at the start of a command handler.
/// All fields are public so callers can use them directly.
pub struct CommandContext {
	pub ctx: SerenityContext,
	pub bot_data: Arc<BotData>,
	pub command_interaction: CommandInteraction,
	pub db: Arc<DatabaseConnection>,
	pub anilist_cache: Arc<RwLock<CacheInterface>>,
	pub vndb_cache: Arc<RwLock<CacheInterface>>,
	/// Guild ID as a string, or `"0"` when the command was used in a DM.
	pub guild_id: String,
}

impl CommandContext {
	/// Extract all common state from the Serenity context and command interaction.
	pub fn new(ctx: SerenityContext, command_interaction: CommandInteraction) -> Self {
		let bot_data = ctx.data::<BotData>().clone();
		let db = bot_data.db_connection.clone();
		let anilist_cache = bot_data.anilist_cache.clone();
		let vndb_cache = bot_data.vndb_cache.clone();
		let guild_id = command_interaction
			.guild_id
			.map(|id| id.to_string())
			.unwrap_or_else(|| String::from("0"));

		Self {
			ctx,
			bot_data,
			command_interaction,
			db,
			anilist_cache,
			vndb_cache,
			guild_id,
		}
	}

	/// Fetch the guild's preferred language as a [`LanguageIdentifier`].
	///
	/// This performs a database lookup; call it only once per command invocation.
	pub async fn lang_id(&self) -> LanguageIdentifier {
		let lang = get_guild_language(self.guild_id.clone(), self.db.clone()).await;
		let lang_code = match lang.as_str() {
			"jp" => "ja",
			"en" => "en-US",
			other => other,
		};
		LanguageIdentifier::from_str(lang_code)
			.unwrap_or_else(|_| LanguageIdentifier::from_str("en-US").unwrap())
	}
}
