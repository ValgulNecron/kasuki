use serenity::all::{ChannelId, CommandInteraction, Context};

/// Checks if the channel where a command was issued is marked as NSFW (Not Safe For Work).
///
/// This function takes a `CommandInteraction` and a `Context` as parameters.
/// It retrieves the `ChannelId` from the `CommandInteraction` and uses it to get the `Channel` object.
/// It then checks if the `Channel` is marked as NSFW by calling the `is_nsfw` method.
///
/// # Arguments
///
/// * `command` - A reference to the `CommandInteraction` that triggered the command.
/// * `ctx` - A reference to the `Context` in which the command was issued.
///
/// # Returns
///
/// * A boolean value indicating whether the channel is marked as NSFW. Returns `true` if the channel is NSFW, `false` otherwise.
pub async fn get_nsfw(command: &CommandInteraction, ctx: &Context) -> bool {
    let channel_id: ChannelId = command.channel_id;
    let channel = channel_id.to_channel(&ctx.http).await.unwrap();
    channel.is_nsfw()
}
