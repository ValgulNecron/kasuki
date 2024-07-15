use std::error::Error;
use std::sync::Arc;

use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};

use crate::config::Config;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::ResponseError;
use crate::structure::message::bot::credit::load_localization_credit;

/// Executes the command to display the bot's credits.
///
/// This function retrieves the guild ID from the command interaction, loads the localized strings for the credits,
/// constructs a description by concatenating the descriptions of all credits, creates a default embed with the description and the title,
/// constructs a message for the response with the embed, and sends the response to the command interaction.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command executed successfully, or `Err` if an error occurred.
pub async fn run(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
) -> Result<(), Box<dyn Error>> {
    let db_type = config.bot.config.db_type.clone();
    // Retrieve the guild ID from the command interaction or use "0" if it does not exist
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    // Load the localized strings for the credits
    let credit_localised = load_localization_credit(guild_id, db_type).await?;

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
