use serenity::client::Context;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::ChannelId;

pub async fn get_nsfw(command: &ApplicationCommandInteraction, ctx: &Context) -> bool {
    let channel_id: ChannelId = command.channel_id;
    let is_nsfw;
    let channel = channel_id.to_channel(&ctx.http).await.unwrap();
    is_nsfw = channel.is_nsfw();
    is_nsfw
}
