use anyhow::Context;
use crate::command::command::CommandRun;
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::helper::vndbapi::character::get_character;
use kasuki_macros::slash_command;
use markdown_converter::vndb::convert_vndb_markdown;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{get_language_identifier, Loader, USABLE_LOCALES};
use tracing::{debug, info, warn};

#[slash_command(
	name = "character", desc = "Get info of a VN character.",
	command_type = SubCommand(parent = "vn"),
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [(name = "name", desc = "Name of the character.", arg_type = String, required = true, autocomplete = true)],
)]
async fn vn_character_command(self_: VnCharacterCommand) -> Result<EmbedsContents<'_>> {
	info!("Processing VN character command");
	self_.defer().await?;
	let ctx = self_.get_ctx();
	let bot_data = ctx.data::<BotData>().clone();
	let command_interaction = self_.get_command_interaction();
	let vndb_cache = bot_data.vndb_cache.clone();

	let guild_id = match command_interaction.guild_id {
		Some(id) => {
			debug!("Command executed in guild: {}", id);
			id.to_string()
		},
		None => {
			debug!("Command executed in DM");
			String::from("0")
		},
	};

	let map = get_option_map_string_subcommand(command_interaction);

	debug!("Command options: {:?}", map);

	// Extract the character name from the command options
	// If no name is provided, default to an empty string
	let character = map
		.get(&String::from("name"))
		.cloned()
		.unwrap_or(String::new());
	let db_connection = bot_data.db_connection.clone();

	debug!("Loading character localization for guild: {}", guild_id);
	let lang_id = get_language_identifier(guild_id.clone(), db_connection).await;
	debug!("Character localization loaded successfully");

	info!("Fetching character information for: {}", character);
	let character = get_character(character.clone(), vndb_cache)
		.await
		.context(format!(
			"Failed to get character information for: {}",
			character
		))?;

	debug!("Found {} character results", character.results.len());
	if character.results.is_empty() {
		warn!("No character results found for the query");
	}

	// Get the first character from the results
	// This is safe because we've already checked if the results array is empty
	let character = character
		.results
		.get(0)
		.context(format!(
			"No character results found for: {}",
			character.clone().results.len()
		))?
		.clone();
	info!("Processing character: {}", character.name);

	// Initialize an empty vector to store the embed fields
	// Each field will be a tuple of (name, value, inline)
	let mut fields = vec![];

	if let Some(blood_type) = character.blood_type {
		fields.push((USABLE_LOCALES.lookup(&lang_id, "vn_character-blood_type"), blood_type, true));
	}

	if let Some(height) = character.height {
		let cm = format!("{}cm", height);

		fields.push((USABLE_LOCALES.lookup(&lang_id, "vn_character-height"), cm, true));
	}

	if let Some(weight) = character.weight {
		let weight = format!("{}kg", weight);

		fields.push((USABLE_LOCALES.lookup(&lang_id, "vn_character-weight"), weight, true));
	}

	if let Some(age) = character.age {
		fields.push((USABLE_LOCALES.lookup(&lang_id, "vn_character-age"), age.to_string(), true));
	}

	if let Some(bust) = character.bust {
		let bust = format!("{}cm", bust);

		fields.push((USABLE_LOCALES.lookup(&lang_id, "vn_character-bust"), bust, true));
	}

	if let Some(waist) = character.waist {
		let waist = format!("{}cm", waist);

		fields.push((USABLE_LOCALES.lookup(&lang_id, "vn_character-waist"), waist, true));
	}

	if let Some(hips) = character.hips {
		let hips = format!("{}cm", hips);

		fields.push((USABLE_LOCALES.lookup(&lang_id, "vn_character-hip"), hips, true));
	}

	if let Some(cup) = character.cup {
		fields.push((USABLE_LOCALES.lookup(&lang_id, "vn_character-cup"), cup, true));
	}

	let sex = format!("{}, ||{}||", character.sex[0], character.sex[1]);

	fields.push((USABLE_LOCALES.lookup(&lang_id, "vn_character-sex"), sex, true));

	if let Some(birthday) = character.birthday {
		let birthday = format!("{:02}/{:02}", birthday[0], birthday[1]);

		fields.push((USABLE_LOCALES.lookup(&lang_id, "vn_character-birthday"), birthday, true));
	}

	let vns = character
		.vns
		.iter()
		.map(|vn| vn.title.clone())
		.take(10)
		.collect::<Vec<String>>()
		.join(", ");

	fields.push((USABLE_LOCALES.lookup(&lang_id, "vn_character-vns"), vns, true));

	let traits = character
		.traits
		.iter()
		.map(|traits| traits.name.clone())
		.take(10)
		.collect::<Vec<String>>()
		.join(", ");

	fields.push((USABLE_LOCALES.lookup(&lang_id, "vn_character-traits"), traits, true));
	let char_desc = character.description.clone().unwrap_or_default();

	// Extract the sexual content rating from the character image
	// If no image is available, default to 2.0 (unsafe)
	let sexual = match character.image.clone() {
		Some(image) => image.sexual,
		None => 2.0,
	};

	// Extract the violence content rating from the character image
	// If no image is available, default to 2.0 (unsafe)
	let violence = match character.image.clone() {
		Some(image) => image.violence,
		None => 2.0,
	};

	// Extract the image URL from the character image
	// If no image is available, set to None
	let url: Option<String> = match character.image {
		Some(image) => Some(image.url.clone()),
		None => None,
	};

	// Create the embed content with the character information
	// The description is converted from VNDB markdown format to Discord markdown
	debug!("Building embed content for character: {}", character.name);
	let mut embed_content = EmbedContent::new(character.name.clone())
		.description(String::from(convert_vndb_markdown(&char_desc)))
		.fields(fields)
		.url(format!("https://vndb.org/{}", character.id));

	// Check if the character image is safe to display
	// Images are considered safe if:
	// - Sexual content rating is <= 1.5 (low to moderate)
	// - Violence rating is <= 1.0 (low)
	if (sexual <= 1.5) && (violence <= 1.0) {
		debug!(
			"Character image is safe to display (sexual: {}, violence: {})",
			sexual, violence
		);
		if let Some(url) = url.clone() {
			debug!("Adding image URL to embed: {}", url);
			embed_content = embed_content.images_url(url);
		} else {
			debug!("No image URL available for character");
		}
	} else {
		// Skip adding the image if it's not safe to display
		warn!(
			"Character image not displayed due to content rating (sexual: {}, violence: {})",
			sexual, violence
		);
	}

	// Create the final embed contents with the CommandType::Followup flag
	// This indicates that this is a followup message after the defer
	debug!("Creating final embed contents");
	let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

	// Return the embed contents wrapped in Ok
	// This indicates that the command was processed successfully
	info!("VN character command processed successfully");
	Ok(embed_contents)
}
