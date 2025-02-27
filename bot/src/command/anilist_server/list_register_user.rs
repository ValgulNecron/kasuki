use std::borrow::Cow;

use crate::command::command_trait::{Command, Embed, EmbedContent, EmbedType, SlashCommand};
use crate::config::DbConfig;
use crate::constant::{MEMBER_LIST_LIMIT, PASS_LIMIT};
use crate::database::prelude::RegisteredUser;
use crate::database::registered_user::{Column, Model};
use crate::event_handler::BotData;
use crate::get_url;
use crate::structure::message::anilist_server::list_register_user::load_localization_list_user;
use anyhow::{anyhow, Result};
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serenity::all::{
	CommandInteraction, Context as SerenityContext, CreateActionRow, CreateButton, PartialGuild,
	User, UserId,
};
use serenity::nonmax::NonMaxU16;

pub struct ListRegisterUser {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for ListRegisterUser {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for ListRegisterUser {
	async fn run_slash(&self) -> Result<()> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();
		let config = bot_data.config.clone();

		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		// Load the localized text for the list user command
		let list_user_localised = load_localization_list_user(guild_id, config.db.clone()).await?;

		// Retrieve the guild from the guild ID
		let guild_id = match command_interaction.guild_id {
			Some(id) => id,
			None => return Err(anyhow!("Failed to get the id of the guild")),
		};

		let guild = guild_id.to_partial_guild_with_counts(&ctx.http).await?;

		self.defer().await?;

		let (desc, len, last_id): (String, usize, Option<UserId>) =
			get_the_list(guild, ctx, None, config.db.clone()).await?;

		let mut content = EmbedContent {
			title: list_user_localised.title,
			description: desc,
			thumbnail: None,
			url: None,
			command_type: EmbedType::Followup,
			colour: None,
			fields: vec![],
			images: None,
			action_row: None,
			images_url: None,
		};

		if len >= MEMBER_LIST_LIMIT as usize {
			content.action_row = Some(CreateActionRow::Buttons(Cow::from(vec![
				CreateButton::new(format!("user_{}_0", last_id.unwrap()))
					.label(&list_user_localised.next),
			])));
		}
		self.send_embed(content).await
	}
}

struct Data {
	pub user: User,
	pub anilist: String,
}

pub async fn get_the_list(
	guild: PartialGuild, ctx: &SerenityContext, last_id: Option<UserId>, db_config: DbConfig,
) -> Result<(String, usize, Option<UserId>)> {
	let mut anilist_user = Vec::new();

	let mut last_id: Option<UserId> = last_id;

	let mut pass = 0;

	while anilist_user.len() < MEMBER_LIST_LIMIT as usize && pass < PASS_LIMIT {
		pass += 1;

		let members = guild
			.id
			.members(
				&ctx.http,
				Some(NonMaxU16::new(MEMBER_LIST_LIMIT).unwrap_or_default()),
				last_id,
			)
			.await?;

		if members.is_empty() {
			break;
		}

		for member in members {
			last_id = Some(member.user.id);

			let user_id = member.user.id.to_string();

			let connection = sea_orm::Database::connect(get_url(db_config.clone())).await?;

			let row = RegisteredUser::find()
				.filter(Column::UserId.eq(user_id.clone()))
				.one(&connection)
				.await?
				.unwrap_or(Model {
					user_id: user_id.clone(),
					anilist_id: 2134,
					registered_at: Default::default(),
				});

			let user_data = row;

			let data = Data {
				user: member.user,
				anilist: user_data.anilist_id.to_string(),
			};

			anilist_user.push(data)
		}
	}

	let user_links: Vec<String> = anilist_user
		.iter()
		.map(|data| {
			format!(
				"[{}](<https://anilist_user.co/user/{}>)",
				data.user.name, data.anilist
			)
		})
		.collect();

	let joined_string = user_links.join("\n\n");

	Ok((joined_string, anilist_user.len(), last_id))
}
