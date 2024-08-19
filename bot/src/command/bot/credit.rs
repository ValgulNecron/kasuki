use std::error::Error;
use std::sync::Arc;

use crate::command::command_trait::{Command, SlashCommand};
use crate::config::Config;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::ResponseError;
use crate::structure::message::bot::credit::load_localization_credit;
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};

pub struct CreditCommand {
    pub ctx: Context,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
}

impl Command for CreditCommand {
    fn get_ctx(&self) -> &Context {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for CreditCommand {
    async fn run_slash(&self) -> Result<(), Box<dyn Error>> {
        send_embed(&self.ctx, &self.command_interaction, self.config.clone()).await
    }
}

async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
) -> Result<(), Box<dyn Error>> {
    // Retrieve the guild ID from the command interaction or use "0" if it does not exist
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    // Load the localized strings for the credits
    let credit_localised = load_localization_credit(guild_id, config.bot.config.clone()).await?;

    // Construct a description by concatenating the descriptions of all credits
    let mut desc: String = "".to_string();
    for x in credit_localised.credits {
        desc += x.desc.as_str()
    }

    // Create a default embed with the description and the title
    let builder_embed = get_default_embed(None)
        .description(desc)
        .title(&credit_localised.title);

    // Construct a message for the response with the embed
    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    // Construct the response
    let builder = CreateInteractionResponse::Message(builder_message);

    // Send the response to the command interaction
    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|e| ResponseError::Sending(format!("{:#?}", e)))?;
    Ok(())
}
