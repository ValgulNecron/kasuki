use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::model::Timestamp;

pub fn run(_options: &[CommandDataOption]) -> String {
    msg.channel_id.send_message(&ctx.http, |m| {
        m.content("Discord bot that integrate anilist api")
        .embed(|e| {
            e.title("Bot Name")
            .description("Description of the bot")
            .fields(vec![
            ])
            .field("This is the third field", "This is not an inline field", false)
            .footer(|f| f.text("This is a footer"))
            // Add a timestamp for the current time
            // This also accepts a rfc3339 Timestamp
            .timestamp(Timestamp::now())
            })
    })
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("info").description("bot info")
}