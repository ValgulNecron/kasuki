use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};

use crate::anilist_struct::run::studio::StudioWrapper;
use crate::helper::create_normalise_embed::get_default_embed;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::lang_struct::anilist_user::studio::load_localization_studio;

/// Executes the command to fetch and display information about a studio from AniList.
///
/// This function retrieves the name or ID of the studio from the command interaction and fetches the studio's data from AniList.
/// It then formats the studio's data and sends it as a response to the command interaction.
/// The function also handles errors that may occur during the execution of the command, such as errors in fetching data from AniList or sending the response.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command executed successfully, or `Err` if an error occurred.
pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    // Retrieve the name or ID of the studio from the command interaction
    let map = get_option_map_string_subcommand(command_interaction);
    let value = map.get(&String::from("studio")).ok_or(AppError::new(
        String::from("There is no option"),
        ErrorType::Option,
        ErrorResponseType::Message,
    ))?;

    // Fetch the studio's data from AniList
    let data: StudioWrapper = if value.parse::<i32>().is_ok() {
        StudioWrapper::new_studio_by_id(value.parse().unwrap()).await?
    } else {
        StudioWrapper::new_studio_by_search(value).await?
    };

    // Retrieve the guild ID from the command interaction
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    // Clone the studio data
    let studio = data.data.studio.clone();

    // Load the localized studio strings
    let studio_localised = load_localization_studio(guild_id).await?;

    // Initialize a string to store the content of the response
    let mut content = String::new();

    // Iterate over the nodes of the studio's media
    for m in studio.media.nodes {
        // Clone the title of the media
        let title = m.title.clone();

        // Retrieve the romaji and user-preferred titles
        let rj = title.romaji;
        let en = title.user_preferred;

        // Format the text for the response
        let text = format!("[{}/{}]({})", rj, en, m.site_url);

        // Append the text to the content string
        content.push_str(text.as_str());
        content.push('\n')
    }

    // Construct the description for the response
    let desc = studio_localised
        .desc
        .replace("$id$", studio.id.to_string().as_str())
        .replace("$fav$", studio.favourites.to_string().as_str())
        .replace(
            "$animation$",
            studio.is_animation_studio.to_string().as_str(),
        )
        .replace("$list$", content.as_str());

    // Retrieve the name of the studio
    let name = studio.name;

    // Construct the embed for the response
    let builder_embed = get_default_embed(None)
        .description(desc)
        .title(name)
        .url(studio.site_url);

    // Construct the message for the response
    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    // Construct the response
    let builder = CreateInteractionResponse::Message(builder_message);

    // Send the response to the command interaction
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
