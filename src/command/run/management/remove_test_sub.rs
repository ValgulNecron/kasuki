use crate::config::Config;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::{FollowupError, ResponseError};
use crate::helper::get_option::command::get_option_map_user;
use crate::structure::message::management::remove_test_sub::load_localization_remove_test_sub;
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage,
};
use std::error::Error;
use std::sync::Arc;
use tracing::error;

pub async fn run(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
) -> Result<(), Box<dyn Error>> {
    let map = get_option_map_user(command_interaction);
    let user = map.get(&String::from("user"));
    let user = match user {
        Some(user) => user,
        None => {
            return Err(ResponseError::Sending(String::from("No user provided")).into());
        }
    };
    let entitlements = ctx
        .http
        .get_entitlements(Some(*user), None, None, None, None, None, None)
        .await
        .map_err(|e| {
            ResponseError::Sending(format!("Error while sending the premium: {:#?}", e))
        })?;
    let localization = load_localization_remove_test_sub(
        command_interaction.guild_id.unwrap().to_string(),
        config.bot.config.db_type.clone(),
    )
    .await?;
    // defer the response
    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command_interaction
        .create_response(&ctx.http, builder_message)
        .await
        .map_err(|e| ResponseError::Sending(format!("{:#?}", e)))?;
    for entitlement in entitlements {
        if let Err(e) = ctx.http.delete_test_entitlement(entitlement.id).await {
            error!("Error while deleting entitlement: {}", e);
        }
    }

    let embed = get_default_embed(None)
        .description(localization.success.replace("{user}", &user.to_string()));
    let builder = CreateInteractionResponseFollowup::new().embed(embed);
    command_interaction
        .create_followup(&ctx.http, builder)
        .await
        .map_err(|e| FollowupError::Sending(format!("{:#?}", e)))?;

    Ok(())
}
