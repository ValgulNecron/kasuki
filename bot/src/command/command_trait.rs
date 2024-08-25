use crate::event_handler::Handler;
use serenity::all::{
    CommandInteraction, CreateInteractionResponseMessage, SkuFlags, SkuId, SkuKind,
};
use serenity::builder::{CreateButton, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseFollowup};
use serenity::client::Context;
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

pub trait AutoComplete {
    async fn autocomplete(&self) -> Result<(), Box<dyn Error>>;
}

pub trait Embed {
    async fn run_embed(&self, fields: Vec<(String, String, bool)>, image: Option<String>, title: String, description: String, thumbnail: Option<String>, url: Option<String>, command_type: EmbedType) -> Result<(), Box<dyn Error>>;
}

pub trait PremiumCommand {
    async fn check_hourly_limit(
        &self,
        command_name: impl Into<String> + Clone,
        handler: &Handler,
    ) -> Result<bool, Box<dyn Error>>;
}

async fn not_implemented(
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), Box<dyn Error>> {
    let builder_embed = CreateEmbed::new().title("Not Implemented");
    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);
    let builder = CreateInteractionResponse::Message(builder_message);
    command_interaction
        .create_response(&ctx.http, builder)
        .await?;

    Ok(())
}

// impl generic run for the trait MessageCommand
impl<T: Command> MessageCommand for T
where
    T: Fn() -> Result<(), Box<dyn Error>>,
{
    async fn run_message(&self) -> Result<(), Box<dyn Error>> {
        not_implemented(self.get_ctx(), self.get_command_interaction()).await
    }
}

impl<T: Command> SlashCommand for T
where
    T: Fn() -> Result<(), Box<dyn Error>>,
{
    async fn run_slash(&self) -> Result<(), Box<dyn Error>> {
        not_implemented(self.get_ctx(), self.get_command_interaction()).await
    }
}

impl<T: Command> UserCommand for T
where
    T: Fn() -> Result<(), Box<dyn Error>>,
{
    async fn run_user(&self) -> Result<(), Box<dyn Error>> {
        not_implemented(self.get_ctx(), self.get_command_interaction()).await
    }
}

pub async fn get_user_sub(
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(Option<SkuId>, Option<SkuId>), Box<dyn Error>> {
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

    Ok((user_sub, available_user_sku))
}

pub async fn send_premium_response(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    available_sub: Option<SkuId>,
) -> Result<(), Box<dyn Error>> {
    let premium_button = CreateButton::new_premium(available_sub.unwrap());
    let builder = CreateInteractionResponseMessage::new();
    let builder = builder.button(premium_button);
    let builder = CreateInteractionResponse::Message(builder);
    command_interaction
        .create_response(&ctx.http, builder)
        .await?;
    Ok(())
}

impl<T: Command> Embed for T
where
    T: Fn() -> Result<(), Box<dyn Error>>,
{
    async fn run_embed(&self, fields: Vec<(String, String, bool)>, image: Option<String>, title: String, description: String, thumbnail: Option<String>, url: Option<String>,
    command_type: EmbedType) -> Result<(), Box<dyn Error>> {
        let ctx = self.get_ctx();
        let command_interaction = self.get_command_interaction();
        let mut builder_embed = CreateEmbed::new();
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
}

pub enum EmbedType {
    First,
    Followup
}