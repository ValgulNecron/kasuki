use crate::event_handler::BotData;
use reqwest::Client;
use sea_orm::DatabaseConnection;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::cache::CacheInterface;
use shared::config::Config;
use shared::helper::get_guild_lang::get_guild_language;
use shared::image_saver::storage::ImageStore;
use std::str::FromStr;
use std::sync::Arc;
use unic_langid::LanguageIdentifier;

/// Discord-agnostic state needed by service functions.
///
/// Constructable in tests without any Discord infrastructure.
pub struct ServiceContext {
	pub db: Arc<DatabaseConnection>,
	pub anilist_cache: Arc<CacheInterface>,
	pub vndb_cache: Arc<CacheInterface>,
	pub http_client: Arc<Client>,
	pub config: Arc<Config>,
	pub image_store: Arc<dyn ImageStore>,
	/// Guild ID as a string, or `"0"` when used outside a guild.
	pub guild_id: String,
}

impl ServiceContext {
	/// Fetch the guild's preferred language as a [`LanguageIdentifier`].
	///
	/// This performs a database lookup; call it only once per invocation.
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

/// Full context including Discord types — used in command handlers.
///
/// Create with [`CommandContext::new`] at the start of a command handler.
/// Access service-layer fields directly through `Deref` (e.g., `cx.db`, `cx.guild_id`).
pub struct CommandContext {
	pub ctx: SerenityContext,
	pub bot_data: Arc<BotData>,
	pub command_interaction: CommandInteraction,
	pub service: ServiceContext,
}

impl CommandContext {
	/// Extract all common state from the Serenity context and command interaction.
	pub fn new(ctx: SerenityContext, command_interaction: CommandInteraction) -> Self {
		let bot_data = ctx.data::<BotData>().clone();
		let guild_id = command_interaction
			.guild_id
			.map(|id| id.to_string())
			.unwrap_or_else(|| String::from("0"));

		let service = ServiceContext {
			db: bot_data.db_connection.clone(),
			anilist_cache: bot_data.anilist_cache.clone(),
			vndb_cache: bot_data.vndb_cache.clone(),
			http_client: bot_data.http_client.clone(),
			config: bot_data.config.clone(),
			image_store: bot_data.image_store.clone(),
			guild_id,
		};

		Self {
			ctx,
			bot_data,
			command_interaction,
			service,
		}
	}
}

impl std::ops::Deref for CommandContext {
	type Target = ServiceContext;

	fn deref(&self) -> &ServiceContext {
		&self.service
	}
}
