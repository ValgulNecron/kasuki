use serenity::all::{ChannelId, CommandInteraction, Context};

pub async fn get_nsfw(command: &CommandInteraction, ctx: &Context) -> bool {
    let channel_id: ChannelId = command.channel_id;

    let channel = channel_id.to_channel(&ctx.http).await.unwrap();

    if let Some(channel) = channel.guild() {
        return channel.nsfw;
    }

    false
}
