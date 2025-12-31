use crate::command::command::Command;
use crate::constant::{
	MAX_FREE_AI_IMAGES, MAX_FREE_AI_QUESTIONS, MAX_FREE_AI_TRANSCRIPTS, MAX_FREE_AI_TRANSLATIONS,
	PAID_IMAGE_MULTIPLIER, PAID_QUESTION_MULTIPLIER, PAID_TRANSCRIPT_MULTIPLIER,
	PAID_TRANSLATION_MULTIPLIER,
};
use crate::event_handler::BotData;
use serenity::all::{SkuFlags, SkuId};
use serenity::builder::{
	CreateButton, CreateInteractionResponse, CreateInteractionResponseMessage,
};
pub trait PremiumCommand {
	async fn check_hourly_limit(
		&self, command_name: impl Into<String> + Clone, bot_data: &BotData,
		command: PremiumCommandType,
	) -> anyhow::Result<bool>;
}

impl<T: Command> PremiumCommand for T {
	async fn check_hourly_limit(
		&self, command_name: impl Into<String> + Clone, handler: &BotData,
		command: PremiumCommandType,
	) -> anyhow::Result<bool> {
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
