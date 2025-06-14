use crate::command::command::Command;
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::helper::vndbapi::user::get_user;
use crate::structure::message::vn::user::UserLocalised;
use crate::structure::message::vn::user::load_localization_user;
use anyhow::anyhow;
use serenity::all::{CommandInteraction, Context as SerenityContext};

pub struct VnUserCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for VnUserCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	async fn get_contents<'a>(&'a self) -> anyhow::Result<EmbedsContents<'a>> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();
		let config = bot_data.config.clone();
		let vndb_cache = bot_data.vndb_cache.clone();

		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		let map = get_option_map_string_subcommand(command_interaction);

		let user = map
			.get(&String::from("username"))
			.ok_or(anyhow!("No username provided"))?;

		let path = format!("/user?q={}&fields=lengthvotes,lengthvotes_sum", user);

		let user = get_user(path, vndb_cache).await?;

		let user_localised: UserLocalised =
			load_localization_user(guild_id, config.db.clone()).await?;

		let fields = vec![
			(user_localised.id.clone(), user.id.clone(), true),
			(
				user_localised.playtime.clone(),
				user.lengthvotes.to_string(),
				true,
			),
			(
				user_localised.playtimesum.clone(),
				user.lengthvotes_sum.to_string(),
				true,
			),
			(user_localised.name.clone(), user.username.clone(), true),
		];

		let embed_content =
			EmbedContent::new(user_localised.title.replace("$user$", &user.username))
				.fields(fields);

		let embed_contents = EmbedsContents::new(CommandType::First, vec![embed_content]);

		Ok(embed_contents)
	}
}
