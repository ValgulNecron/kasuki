use std::error::Error;
use std::sync::Arc;

use crate::command::command_trait::{Command, SlashCommand};
use crate::config::Config;
use crate::error_management::error_dispatch;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::get_option::command::{get_option_map_string, get_option_map_user};
use crate::structure::message::management::give_premium_sub::load_localization_give_premium_sub;
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
    EntitlementOwner,
};

pub struct GivePremiumSubCommand {
    pub ctx: Context,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
}

impl Command for GivePremiumSubCommand {
    fn get_ctx(&self) -> &Context {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for GivePremiumSubCommand {
    async fn run_slash(&self) -> Result<(), Box<dyn Error>> {
        send_embed(&self.ctx, &self.command_interaction, self.config.clone()).await
    }
}

async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
) -> Result<(), Box<dyn Error>> {
    let map = get_option_map_user(command_interaction);

    let user = *map
        .get(&String::from("user"))
        .ok_or(error_dispatch::Error::Option(String::from(
            "No option for user",
        )))?;

    let map = get_option_map_string(command_interaction);

    let subscription = map
        .get(&String::from("subscription"))
        .ok_or(error_dispatch::Error::Option(String::from(
            "No option for subscription",
        )))?
        .clone();

    let skus = ctx.http.get_skus().await?;

    let skus_id: Vec<String> = skus.iter().map(|sku| sku.id.to_string()).collect();

    if !skus_id.contains(&subscription) {
        Err(error_dispatch::Error::Option(String::from(
            "Invalid sub id",
        )))?
    }

    let mut sku_id = Default::default();

    for sku in skus {
        if sku.id.to_string() == subscription {
            sku_id = sku.id;
        }
    }

    let _ = ctx
        .http
        .create_test_entitlement(sku_id, EntitlementOwner::User(user))
        .await?;

    let localization = load_localization_give_premium_sub(
        command_interaction.guild_id.unwrap().to_string(),
        config.db.clone(),
    )
    .await?;

    let embed = get_default_embed(None).description(
        localization
            .success
            .replace("{user}", &user.to_string())
            .replace("{subscription}", &subscription),
    );

    let builder_message = CreateInteractionResponseMessage::new().embed(embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command_interaction
        .create_response(&ctx.http, builder)
        .await?;

    Ok(())
}
