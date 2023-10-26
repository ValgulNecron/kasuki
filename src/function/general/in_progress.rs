use crate::structure::embed::general::struct_lang_in_progress::InProgressLocalisedText;
use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::Timestamp;
use serenity::utils::Colour;

/// An asynchronous function that sends an embedded message indicating a command is in progress.
///
/// This function accepts a context and an application command interaction as parameters,
/// builds a localised in-progress message based on these parameters
/// and colorizes it with `Colour::FABLED_PINK`.
///
/// It creates a follow-up message and embeds the localised text message
/// with timestamp and color in it.
///
/// This embedded message is then sent asynchronously.
///
/// Ideally, this function could be used to notify a user that a certain command
/// they initiated is currently being processed.
///
/// # Parameters
/// * `ctx`: A reference to the context, which stores data such as the HTTP client for the bot to use.
/// * `command`: A reference to the ApplicationCommandInteraction which represents the user's command.
///
/// # Return
/// This function returns a `Result<Option<Message>, String>`.
/// If the function succeeds, `Ok(Some(message))` is returned where `message`
/// is the unwrapped followup message containing the embedded in-progress information.
/// If an error occurs while fetching the localised text, `Err(String)` is returned
/// which carries the error message.
///
/// # Errors
/// This function will return an `Err` variant if the call to `get_in_progress_localised` fails.
///
/// # Example
/// ```no_run
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let ctx = ...; // Some context
///     let command = ...; // Some command
///     let result = in_progress_embed(ctx, command).await;
///     println!("Result: {:?}", result);
///     Ok(())
/// }
/// ```
pub async fn in_progress_embed(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<Option<Message>, String> {
    let color = Colour::FABLED_PINK;
    let localised_text =
        match InProgressLocalisedText::get_in_progress_localised(color, ctx, command).await {
            Ok(data) => data,
            Err(data) => return Err(data.parse().unwrap()),
        };
    let message = command
        .create_followup_message(&ctx.http, |f| {
            f.embed(|e| {
                e.title(&localised_text.title)
                    .description(&localised_text.description)
                    .timestamp(Timestamp::now())
                    .color(color)
            })
        })
        .await;
    Ok(Some(message.unwrap()))
}
