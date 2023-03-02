use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::prelude::ChannelId;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::model::Timestamp;

pub async fn run(_options: &[CommandDataOption], msg_id: ChannelId, ctx: &Context) -> String {
    msg_id.send_message(&ctx.http, |m| {
        m.content("Discord bot that integrate anilist api")
        .embed(|e| {
            e.title("Bot Name")
            .description("Description of the bot")
            .field("This is the third field", "This is not an inline field", false)
            .footer(|f| f.text("This is a footer"))
            // Add a timestamp for the current time
            // This also accepts a rfc3339 Timestamp
            .timestamp(Timestamp::now())
            })
    });
    return "good".to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("info").description("bot info")
}