use crate::command::command_trait::{Command, SlashCommand};
use crate::config::Config;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_dispatch;
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

pub struct RemoveTestSubCommand {
    pub ctx: Context,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
}

impl Command for RemoveTestSubCommand {
    fn get_ctx(&self) -> &Context {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for RemoveTestSubCommand {
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
    let user = map.get(&String::from("user"));
    let user = match user {
        Some(user) => user,
        None => {
            return Err(error_dispatch::Error::Sending(String::from("No user provided")).into());
        }
    };
    let entitlements = ctx
        .http
        .get_entitlements(Some(*user), None, None, None, None, None, None)
        .await?;
    let localization = load_localization_remove_test_sub(
        command_interaction.guild_id.unwrap().to_string(),
        config.db.clone(),
    )
    .await?;
    // defer the response
    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command_interaction
        .create_response(&ctx.http, builder_message)
        .await?;
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
        .await?;

    Ok(())
}
