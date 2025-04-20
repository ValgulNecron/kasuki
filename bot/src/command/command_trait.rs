use crate::constant::{
	MAX_FREE_AI_IMAGES, MAX_FREE_AI_QUESTIONS, MAX_FREE_AI_TRANSCRIPTS, MAX_FREE_AI_TRANSLATIONS,
	PAID_IMAGE_MULTIPLIER, PAID_QUESTION_MULTIPLIER, PAID_TRANSCRIPT_MULTIPLIER,
	PAID_TRANSLATION_MULTIPLIER,
};
use crate::event_handler::BotData;
use crate::helper::create_default_embed::get_default_embed;
use anyhow::{Result, anyhow};
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{CommandInteraction, CreateInteractionResponseMessage, SkuFlags, SkuId};
use serenity::builder::{
	CreateActionRow, CreateAttachment, CreateButton, CreateEmbedFooter, CreateInteractionResponse,
	CreateInteractionResponseFollowup,
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
	async fn send_embed(&self, content: Vec<EmbedContent>) -> Result<()>;

	async fn defer(&self) -> Result<()>;
}

pub trait PremiumCommand {
	async fn check_hourly_limit(
		&self, command_name: impl Into<String> + Clone, bot_data: &BotData,
		command: PremiumCommandType,
	) -> Result<bool>;
}

#[derive(Clone)]
pub struct EmbedContent<'a, 'b> {
	pub title: String,
	pub description: String,
	pub thumbnail: Option<String>,
	pub url: Option<String>,
	pub command_type: EmbedType,
	pub colour: Option<Colour>,
	pub fields: Vec<(String, String, bool)>,
	pub images: Option<Vec<EmbedImage<'a>>>,
	pub action_row: Vec<CreateActionRow<'b>>,
	pub images_url: Option<String>,
	pub footer: Option<String>,
}

impl<'a, 'b> EmbedContent<'a, 'b> {
	pub fn new(title: String) -> EmbedContent<'static, 'static> {
		EmbedContent {
			title,
			description: "".to_string(),
			thumbnail: None,
			url: None,
			command_type: EmbedType::First,
			colour: None,
			fields: vec![],
			images: None,
			action_row: vec![],
			images_url: None,
			footer: None,
		}
	}

	pub fn title(mut self, title: String) -> EmbedContent<'a, 'b> {
		self.title = title;
		self
	}
	pub fn description(mut self, description: String) -> EmbedContent<'a, 'b> {
		self.description = description;
		self
	}
	pub fn thumbnail(mut self, thumbnail: Option<String>) -> EmbedContent<'a, 'b> {
		self.thumbnail = thumbnail;
		self
	}

	pub fn url(mut self, url: Option<String>) -> EmbedContent<'a, 'b> {
		self.url = url;
		self
	}

	pub fn command_type(mut self, command_type: EmbedType) -> EmbedContent<'a, 'b> {
		self.command_type = command_type;
		self
	}

	pub fn colour(mut self, colour: Option<Colour>) -> EmbedContent<'a, 'b> {
		self.colour = colour;
		self
	}

	pub fn fields(mut self, fields: Vec<(String, String, bool)>) -> EmbedContent<'a, 'b> {
		self.fields = fields;
		self
	}

	pub fn images(mut self, images: Option<Vec<EmbedImage<'a>>>) -> EmbedContent<'a, 'b> {
		self.images = images;
		self
	}

	pub fn action_row(mut self, action_row: Vec<CreateActionRow<'b>>) -> EmbedContent<'a, 'b> {
		self.action_row = action_row;
		self
	}

	pub fn images_url(mut self, images_url: Option<String>) -> EmbedContent<'a, 'b> {
		self.images_url = images_url;
		self
	}

	pub fn footer(mut self, footer: Option<String>) -> EmbedContent<'a, 'b> {
		self.footer = footer;
		self
	}
}

#[derive(Clone)]
pub struct EmbedImage<'a> {
	pub attachment: CreateAttachment<'a>,
	pub image: String,
}

impl<T: Command> Embed for T {
	async fn send_embed(&self, contents: Vec<EmbedContent<'_, '_>>) -> Result<()> {
		let ctx = self.get_ctx();

		let command_interaction = self.get_command_interaction();
		let mut embeds = vec![];
		let mut attachments = None;

		for content in contents.clone() {
			let mut embed = get_default_embed(content.colour);

			embed = embed.title(content.title).description(content.description);
			if let Some(thumbnail) = content.thumbnail {
				embed = embed.thumbnail(thumbnail);
			}
			if let Some(url) = content.url {
				embed = embed.url(url);
			}

			if let Some(footer) = content.footer {
				embed = embed.footer(CreateEmbedFooter::new(footer));
			}

			embed = embed.fields(content.fields);

			match (content.images, content.images_url) {
				(Some(images), None) => {
					let mut attachment = Vec::new();
					let mut first = true;
					for image in images {
						attachment.push(image.attachment);
						if first {
							first = false;
							embed = embed
								.image(format!("attachment://{}", &image.image))
								.attachment(image.image);
							embeds.push(embed.clone());
						} else {
							let mut embed = get_default_embed(content.colour);
							embed = embed.image(image.image.clone()).attachment(image.image);
							embeds.push(embed);
						}
					}
					attachments = Some(attachment)
				},
				(None, None) => {
					embeds.push(embed);
				},

				(None, Some(image_link)) => {
					embeds.push(embed.image(image_link.clone()));
				},
				_ => {
					return Err(anyhow!("There is both image."));
				},
			}
		}

		match contents[0].command_type {
			EmbedType::First => {
				let builder = CreateInteractionResponseMessage::new()
					.embeds(embeds)
					.files(attachments.unwrap_or_default());

				let builder = CreateInteractionResponse::Message(builder);
				command_interaction
					.create_response(&ctx.http, builder)
					.await?;
			},
			EmbedType::Followup => {
				let builder = CreateInteractionResponseFollowup::new()
					.embeds(embeds)
					.files(attachments.unwrap_or_default());
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

#[derive(Clone)]
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
