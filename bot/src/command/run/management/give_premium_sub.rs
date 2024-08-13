use std::error::Error;
use std::sync::Arc;

use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
    EntitlementOwner,
};

use crate::config::Config;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::ResponseError;
use crate::helper::get_option::command::{get_option_map_string, get_option_map_user};
use crate::structure::message::management::give_premium_sub::load_localization_give_premium_sub;

pub async fn run(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
) -> Result<(), Box<dyn Error>> {
    let map = get_option_map_user(command_interaction);
    let user = *map
        .get(&String::from("user"))
        .ok_or(ResponseError::Option(String::from("No option for user")))?;
    let map = get_option_map_string(command_interaction);
    let subscription = map
        .get(&String::from("subscription"))
        .ok_or(ResponseError::Option(String::from(
            "No option for subscription",
        )))?
        .clone();

    let skus = ctx
        .http
        .get_skus()
        .await
        .map_err(|e| ResponseError::WebRequest(format!("{:#?}", e)))?;
    let skus_id: Vec<String> = skus.iter().map(|sku| sku.id.to_string()).collect();
    if !skus_id.contains(&subscription) {
        Err(ResponseError::Option(String::from("Invalid sub id")))?
    }
    let mut sku_id = Default::default();
    for sku in skus {
        if sku.id.to_string() == subscription {
            sku_id = sku.id;
        }
    }

    let res = ctx
        .http
        .create_test_entitlement(sku_id, EntitlementOwner::User(user))
        .await
        .map_err(|e| ResponseError::WebRequest(format!("{:#?}", e)))?;

    let localization = load_localization_give_premium_sub(
        command_interaction.guild_id.unwrap().to_string(),
        config.bot.config.db_type.clone(),
        config.bot.config.clone(),
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
        .await
        .map_err(|e| ResponseError::Sending(format!("{:#?}", e)))?;
    Ok(())
}
