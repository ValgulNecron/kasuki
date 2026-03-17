use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::command::prenium_command::{PremiumCommand, PremiumCommandType};
use crate::helper::get_option::subcommand::{
	get_option_map_attachment_subcommand, get_option_map_string_subcommand,
};
use anyhow::anyhow;
use fluent_templates::Loader;
use kasuki_macros::slash_command;
use reqwest::Url;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::USABLE_LOCALES;
use shared::service::ai;
use uuid::Uuid;

#[slash_command(
	name = "transcript", desc = "Generate a transcript from a video.",
	command_type = SubCommand(parent = "ai"),
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	extra_fields = [command_name: String = full_command_name.to_string()],
	args = [
		(name = "video", desc = "Upload video file (max. 25MB).", arg_type = Attachment, required = true, autocomplete = false),
		(name = "prompt", desc = "A guide text for audio style. Must match the audio language.", arg_type = String, required = false, autocomplete = false),
		(name = "lang", desc = "Select input language (ISO-639-1)", arg_type = String, required = false, autocomplete = false)
	],
)]
async fn transcript_command(self_: TranscriptCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);
	let config = &cx.config;

	if self_
		.check_hourly_limit(
			self_.command_name.clone(),
			&cx.bot_data,
			PremiumCommandType::AITranscript,
		)
		.await?
	{
		return Err(anyhow!(
			"You have reached your hourly limit. Please try again later.",
		));
	}

	let map = get_option_map_string_subcommand(&cx.command_interaction);
	let attachment_map = get_option_map_attachment_subcommand(&cx.command_interaction);
	let prompt = map
		.get("lang")
		.cloned()
		.unwrap_or_default();
	let lang = map
		.get("prompt")
		.cloned()
		.unwrap_or_default();
	let attachment = attachment_map
		.get("video")
		.ok_or(anyhow!("No option for video"))?;

	let content_type = attachment
		.content_type
		.clone()
		.ok_or(anyhow!("Failed to get the content type"))?;
	let content = attachment.proxy_url.clone();
	if !content_type.starts_with("audio/") && !content_type.starts_with("video/") {
		return Err(anyhow!("Unsupported file type"));
	}
	let allowed_extensions = ["mp3", "mp4", "mpeg", "mpga", "m4a", "wav", "webm", "ogg"];

	let parsed_url = Url::parse(content.as_str())?;
	let mut path_segments = parsed_url
		.path_segments()
		.ok_or(anyhow!("Failed to get the path segments"))?;
	let last_segment = path_segments.next_back().unwrap_or_default();
	let file_extension = last_segment
		.rsplit('.')
		.next()
		.ok_or(anyhow!("Failed to get the file extension"))?
		.to_lowercase();
	if !allowed_extensions.contains(&&*file_extension) {
		return Err(anyhow!("Unsupported file extension"));
	}

	let client = &cx.http_client;

	let response = client.get(content.to_string()).send().await?;
	let buffer = response.bytes().await?;
	let uuid_name = Uuid::new_v4().to_string();

	let token = config
		.ai
		.transcription
		.ai_transcription_token
		.clone()
		.unwrap_or_default();
	let model = config
		.ai
		.transcription
		.ai_transcription_model
		.clone()
		.unwrap_or_default();
	let api_base_url = config
		.ai
		.transcription
		.ai_transcription_base_url
		.clone()
		.unwrap_or_default();

	let text = ai::transcribe(
		buffer.to_vec(),
		&content_type,
		&uuid_name,
		&prompt,
		&lang,
		&token,
		&model,
		&api_base_url,
		&client,
	)
	.await?;

	let lang_id = cx.lang_id().await;
	let title = USABLE_LOCALES.lookup(&lang_id, "ai_transcript-title");

	let embed_content = EmbedContent::new(title).description(text);

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}
