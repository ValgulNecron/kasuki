use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::component::ButtonStyle;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::ChannelId;
use serenity::model::prelude::interaction::application_command::{ApplicationCommandInteraction, CommandDataOption};
use serenity::model::Timestamp;

pub async fn run(_options: &[CommandDataOption], ctx: &Context, command: &ApplicationCommandInteraction) -> String {
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.embed(
                    |m| {
                        m.title("Info")
                            .description("This bot use the anilist api to give information on a show or a user")
                            .footer(|f| f.text("creator valgul#8329"))
                            // Add a timestamp for the current time
                            // This also accepts a rfc3339 Timestamp
                            .timestamp(Timestamp::now())
                    })
                    .components(|components| {
                        components.create_action_row(|row| {
                            row.create_button(|button| {
                                button.label("See on github")
                                    .url("https://github.com/ValgulNecron/DIscordAnilistBotRS")
                                    .style(ButtonStyle::Link)
                            })
                                .create_button(|button| {
                                    button.label("Official website")
                                        .url("https://discord.com/api/oauth2/authorize?client_id=923286536445894697&permissions=17861158751296&scope=bot")
                                        .style(ButtonStyle::Link)
                                })
                                .create_button(|button| {
                                    button.label("Official discord")
                                        .url("https://discord.com/api/oauth2/authorize?client_id=923286536445894697&permissions=17861158751296&scope=bot")
                                        .style(ButtonStyle::Link)
                                })
                                .create_button(|button| {
                                    button.label("Add the bot.")
                                        .url("https://discord.com/api/oauth2/authorize?client_id=923286536445894697&permissions=17861158751296&scope=bot")
                                        .style(ButtonStyle::Link)
                                })
                        })
                    })
                )
        })
        .await
    {
        println!("Cannot respond to slash command: {}", why);
    }
    return "good".to_string();
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("info").description("bot info")
}