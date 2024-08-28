use crate::constant::{
    MAX_FREE_AI_IMAGES, MAX_FREE_AI_QUESTIONS, MAX_FREE_AI_TRANSCRIPTS, MAX_FREE_AI_TRANSLATIONS,
    PAID_IMAGE_MULTIPLIER, PAID_QUESTION_MULTIPLIER, PAID_TRANSCRIPT_MULTIPLIER,
    PAID_TRANSLATION_MULTIPLIER,
};
use crate::event_handler::Handler;
use crate::helper::create_default_embed::get_default_embed;
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, CreateInteractionResponseMessage, SkuFlags, SkuId, SkuKind,
};
use serenity::builder::{
    CreateButton, CreateInteractionResponse, CreateInteractionResponseFollowup,
};
use serenity::client::Context;
use serenity::model::Colour;
use std::error::Error;

pub trait Command {
    fn get_ctx(&self) -> &Context;
    fn get_command_interaction(&self) -> &CommandInteraction;
}

pub trait MessageCommand {
    async fn run_message(&self) -> Result<(), Box<dyn Error>>;
}

pub trait SlashCommand {
    async fn run_slash(&self) -> Result<(), Box<dyn Error>>;
}

pub trait UserCommand {
    async fn run_user(&self) -> Result<(), Box<dyn Error>>;
}

pub trait Embed {
    async fn send_embed(
        &self,
        fields: Vec<(String, String, bool)>,
        image: Option<String>,
        title: String,
        description: String,
        thumbnail: Option<String>,
        url: Option<String>,
        command_type: EmbedType,
        colour: Option<Colour>,
    ) -> Result<(), Box<dyn Error>>;

    async fn defer(&self) -> Result<(), Box<dyn Error>>;
}

pub trait PremiumCommand {
    async fn check_hourly_limit(
        &self,
        command_name: impl Into<String> + Clone,
        handler: &Handler,
        command: PremiumCommandType,
    ) -> Result<bool, Box<dyn Error>>;
}

impl<T: Command> Embed for T {
    async fn send_embed(
        &self,
        fields: Vec<(String, String, bool)>,
        image: Option<String>,
        title: String,
        description: String,
        thumbnail: Option<String>,
        url: Option<String>,
        command_type: EmbedType,
        colour: Option<Colour>,
    ) -> Result<(), Box<dyn Error>> {
        let ctx = self.get_ctx();
        let command_interaction = self.get_command_interaction();
        let mut builder_embed = get_default_embed(colour);
        if let Some(image) = image {
            builder_embed = builder_embed.image(image);
        }
        builder_embed = builder_embed.title(title);
        builder_embed = builder_embed.description(description);
        if let Some(thumbnail) = thumbnail {
            builder_embed = builder_embed.thumbnail(thumbnail);
        }
        if let Some(url) = url {
            builder_embed = builder_embed.url(url);
        }
        builder_embed = builder_embed.fields(fields);

        match command_type {
            EmbedType::First => {
                let builder = CreateInteractionResponseMessage::new().embed(builder_embed);
                let builder = CreateInteractionResponse::Message(builder);
                command_interaction
                    .create_response(&ctx.http, builder)
                    .await?;
            }
            EmbedType::Followup => {
                let builder = CreateInteractionResponseFollowup::new().embed(builder_embed);
                command_interaction
                    .create_followup(&ctx.http, builder)
                    .await?;
            }
        }

        Ok(())
    }

    async fn defer(&self) -> Result<(), Box<dyn Error>> {
        let ctx = self.get_ctx();
        let command_interaction = self.get_command_interaction();
        let builder_message = Defer(CreateInteractionResponseMessage::new());
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
        &self,
        command_name: impl Into<String> + Clone,
        handler: &Handler,
        command: PremiumCommandType,
    ) -> Result<bool, Box<dyn Error>> {
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
        if !handler.bot_data.config.bot.respect_premium {
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
            match available_sku.kind {
                SkuKind::Subscription => {
                    if available_sku.flags == SkuFlags::USER_SUBSCRIPTION {
                        available_user_sku = Some(available_sku.id);
                        if user_sub.is_none() && user_skus.contains(&available_sku.id) {
                            user_sub = Some(available_sku.id);
                        }
                    }
                }
                SkuKind::SubscriptionGroup => {}
                SkuKind::Unknown(_) => {}
                _ => {}
            }
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
