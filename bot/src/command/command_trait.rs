use crate::constant::{
	MAX_FREE_AI_IMAGES, MAX_FREE_AI_QUESTIONS, MAX_FREE_AI_TRANSCRIPTS, MAX_FREE_AI_TRANSLATIONS,
	PAID_IMAGE_MULTIPLIER, PAID_QUESTION_MULTIPLIER, PAID_TRANSCRIPT_MULTIPLIER,
	PAID_TRANSLATION_MULTIPLIER,
};
use crate::event_handler::BotData;
use crate::helper::create_default_embed::get_default_embed;
use anyhow::Result;
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{CommandInteraction, CreateInteractionResponseMessage, SkuFlags, SkuId};
use serenity::builder::{
	CreateAttachment, CreateButton, CreateInteractionResponse, CreateInteractionResponseFollowup,
};
use serenity::model::Colour;
use serenity::prelude::Context as SerenityContext;

pub trait Command {
	fn get_ctx(&self) -> &SerenityContext;

	fn get_command_interaction(&self) -> &CommandInteraction;
}

pub trait MessageCommand {
	async fn run_message(&self) -> Result<()>;
}

pub trait SlashCommand {
	async fn run_slash(&self) -> Result<()>;
}

pub trait UserCommand {
	async fn run_user(&self) -> Result<()>;
}

pub trait Embed {
	async fn send_embed(
		&self, fields: Vec<(String, String, bool)>, images: Option<Vec<String>>, title: String,
		description: String, thumbnail: Option<String>, url: Option<String>,
		command_type: EmbedType, colour: Option<Colour>, attachments: Vec<CreateAttachment>,
	) -> Result<()>;

	async fn defer(&self) -> Result<()>;
}

pub trait PremiumCommand {
	async fn check_hourly_limit(
		&self, command_name: impl Into<String> + Clone, bot_data: &BotData,
		command: PremiumCommandType,
	) -> Result<bool>;
}

impl<T: Command> Embed for T {
	async fn send_embed(
		&self, fields: Vec<(String, String, bool)>, images: Option<Vec<String>>, title: String,
		description: String, thumbnail: Option<String>, url: Option<String>,
		command_type: EmbedType, colour: Option<Colour>, attachments: Vec<CreateAttachment<'_>>,
	) -> Result<()> {
		let ctx = self.get_ctx();

		let command_interaction = self.get_command_interaction();

		let mut builder_embed = get_default_embed(colour);

		builder_embed = builder_embed.title(title);

		builder_embed = builder_embed.description(description);

		if let Some(thumbnail) = thumbnail {
			builder_embed = builder_embed.thumbnail(thumbnail);
		}

		if let Some(url) = url {
			builder_embed = builder_embed.url(url);
		}

		builder_embed = builder_embed.fields(fields);

		let mut builders_embeds = Vec::new();

		if let Some(images) = images {
			if images.len() <= 1 {
				builder_embed = builder_embed.image(images[0].clone());
				builders_embeds.push(builder_embed)
			} else {
				for image in images {
					let builder_embed = builder_embed.clone().image(image.clone());
					builders_embeds.push(builder_embed)
				}
			}
		}

		match command_type {
			EmbedType::First => {
				let builder = CreateInteractionResponseMessage::new()
					.embeds(builders_embeds)
					.files(attachments);

				let builder = CreateInteractionResponse::Message(builder);

				command_interaction
					.create_response(&ctx.http, builder)
					.await?;
			},
			EmbedType::Followup => {
				let builder = CreateInteractionResponseFollowup::new()
					.embeds(builders_embeds)
					.files(attachments);

				command_interaction
					.create_followup(&ctx.http, builder)
					.await?;
			},
		}

		Ok(())
	}

	async fn defer(&self) -> Result<()> {
		let ctx = self.get_ctx();

		let command_interaction = self.get_command_interaction();

		let builder_message = Defer(Default::default());

		command_interaction
			.create_response(&ctx.http, builder_message)
			.await?;

		Ok(())
	}
}

pub enum EmbedType {
	First,
	Followup,
}

impl<T: Command> PremiumCommand for T {
	async fn check_hourly_limit(
		&self, command_name: impl Into<String> + Clone, handler: &BotData,
		command: PremiumCommandType,
	) -> Result<bool> {
		let bot_data = self.get_ctx().data::<BotData>().clone();

		let ctx = self.get_ctx();

		let command_interaction = self.get_command_interaction();

		let free_limit = match command {
			PremiumCommandType::AIImage => MAX_FREE_AI_IMAGES,
			PremiumCommandType::AIQuestion => MAX_FREE_AI_QUESTIONS,
			PremiumCommandType::AITranscript => MAX_FREE_AI_TRANSCRIPTS,
			PremiumCommandType::AITranslation => MAX_FREE_AI_TRANSLATIONS,
		};

		let paid_multiplier = match command {
			PremiumCommandType::AIImage => PAID_IMAGE_MULTIPLIER,
			PremiumCommandType::AIQuestion => PAID_QUESTION_MULTIPLIER,
			PremiumCommandType::AITranscript => PAID_TRANSCRIPT_MULTIPLIER,
			PremiumCommandType::AITranslation => PAID_TRANSLATION_MULTIPLIER,
		};

		if !bot_data.config.bot.respect_premium {
			return Ok(false);
		}

		let usage = handler
			.get_hourly_usage(command_name.into(), command_interaction.user.id.to_string())
			.await;

		let user_skus: Vec<SkuId> = command_interaction
			.entitlements
			.iter()
			.map(|entitlement| entitlement.sku_id)
			.collect();

		let available_skus = ctx.http.get_skus().await?;

		let mut user_sub = None;

		let mut available_user_sku = None;

		for available_sku in available_skus {
			match available_sku.kind.0 {
				5 => {
					if available_sku.flags == SkuFlags::USER_SUBSCRIPTION {
						available_user_sku = Some(available_sku.id);

						if user_sub.is_none() && user_skus.contains(&available_sku.id) {
							user_sub = Some(available_sku.id);
						}
					}
				},
				6 => {},
				2 => {},
				3 => {},
				_ => {},
			};
		}

		if available_user_sku.is_none() {
			return Ok(false);
		}

		if usage <= free_limit as u128 && user_sub.is_none() {
			return Ok(false);
		}

		if usage <= (free_limit as f64 * paid_multiplier) as u128 && user_sub.is_some() {
			return Ok(false);
		}

		let premium_button = CreateButton::new_premium(available_user_sku.unwrap());

		let builder = CreateInteractionResponseMessage::new();

		let builder = builder.button(premium_button);

		let builder = CreateInteractionResponse::Message(builder);

		command_interaction
			.create_response(&ctx.http, builder)
			.await?;

		Ok(true)
	}
}

pub enum PremiumCommandType {
	AIImage,
	AIQuestion,
	AITranscript,
	AITranslation,
}
