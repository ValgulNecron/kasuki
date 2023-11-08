use std::fs;
use std::path::PathBuf;

use serenity::client::Context;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::InteractionResponseType;

/// An asynchronous function to respond to an Application Command Interaction in "deferred" mode.
///
/// This function, which is part of a structure `Context`, receives two arguments:
/// - A reference (`&`) to `Context` (most likely the Discord communication context)
/// - A reference (`&`) to `ApplicationCommandInteraction` (which represents the command that triggered this function)
///
/// As a result of the function, a response to the initial interaction is created with a type set to `DeferredChannelMessageWithSource`.
/// If the `create_interaction_response()` method fails to create a response, the error is printed to the console.
///
/// # Arguments
///
/// * `ctx` - `Context` reference which represents the context in which this function operates and commands are processed.
/// * `command` - An `ApplicationCommandInteraction` reference, which represents the specific command interaction to respond to.
///
/// # Example
///
/// ```
/// let ctx = // insert how to get context here
/// let command = // insert how to get command here
/// differed_response(&ctx, &command).await;
/// ```
///
/// # Note
///
/// This function is defined as `pub async`, read more about asynchronous functions in Rust if you are unfamiliar.
pub async fn differed_response(ctx: &Context, command: &ApplicationCommandInteraction) {
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
        })
        .await
    {
        println!("Cannot respond to slash command: {}", why);
    }
}

/// Asynchronously responds to an application command interaction with a differed message which includes the command's source.
/// This function also attempts to delete a specified file regardless of whether the interaction response is successful or not.
///
/// # Arguments
///
/// * `ctx` - A reference to the context of the command interaction.
/// * `command` - A reference to the ApplicationCommandInteraction; the command that was interacted with.
/// * `file_to_delete` - A PathBuf object specifying the file to delete.
///
/// # Error
///
/// If the interaction response fails, it tries to remove the specified file
/// and logs the error message with the reason on the console. The error of file deletion
/// will be suppressed.
///
/// # Example
///
/// ```no_run
/// use serenity::client::Context;
/// use serenity::model::interactions::application_command::ApplicationCommandInteraction;
/// use std::path::PathBuf;
///
/// let ctx: Context = // ...
/// let command: ApplicationCommandInteraction = // ...
/// let file_to_delete: PathBuf = // ...
///
/// differed_response_with_file_deletion(&ctx, &command, file_to_delete).await;
/// ```
pub async fn differed_response_with_file_deletion(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    file_to_delete: PathBuf,
) {
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
        })
        .await
    {
        let _ = fs::remove_file(&file_to_delete);
        println!("Cannot respond to slash command: {}", why);
    }
}
