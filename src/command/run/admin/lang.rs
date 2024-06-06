use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};

use crate::database::data_struct::guild_language::GuildLanguage;
use crate::database::manage::dispatcher::data_dispatch::set_data_guild_language;
use crate::helper::create_normalise_embed::get_default_embed;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::get_option::subcommand_group::get_option_map_string_subcommand_group;
use crate::structure::message::admin::lang::load_localization_lang;

/// This asynchronous function runs the command interaction for setting the language of a guild.
///
/// It first retrieves the language choice from the command interaction options.
/// If the language choice is not found, it returns an `AppError` indicating that there is no option.
///
/// It retrieves the guild ID from the command interaction. If the command interaction does not have a guild ID, it uses "0" as the guild ID.
///
/// It sets the language of the guild in the database.
///
/// It loads the localized language data for the guild.
///
/// It creates an embed for the response message, including the current timestamp, a color, a description containing the localized language data and the chosen language, and a title containing the localized language data.
///
/// It creates a response message with the embed.
///
/// It creates a response with the response message.
///
/// It sends the response to the command interaction. If an error occurs during this process, it returns an `AppError` indicating that there was an error while sending the command.
///
/// # Arguments
///
/// * `ctx` - The context in which this function is being called.
/// * `command_interaction` - The command interaction that triggered this function.
///
/// # Returns
///
/// A `Result` indicating whether the function executed successfully. If an error occurred, it contains an `AppError`.
pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let map = get_option_map_string_subcommand_group(command_interaction);
    let lang = map.get(&String::from("lang_choice")).ok_or(AppError::new(
        String::from("There is no option"),
        ErrorType::Option,
        ErrorResponseType::Followup,
    ))?;

    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };
    let guild_language = GuildLanguage {
        guild: guild_id.clone(),
        lang: lang.clone(),
    };
    let _ = set_data_guild_language(guild_language).await;
    let lang_localised = load_localization_lang(guild_id).await?;

    let builder_embed = get_default_embed(None)
        .description(lang_localised.desc.replace("$lang$", lang.as_str()))
        .title(&lang_localised.title);

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {}", e),
                ErrorType::Command,
                ErrorResponseType::Message,
            )
        })
}
