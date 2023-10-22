use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::Timestamp;
use serenity::utils::Colour;

use crate::cmd::lang_struct::embed::general::struct_lang_credit::CreditLocalisedText;
use crate::cmd::lang_struct::register::general::struct_credit_register::RegisterLocalisedCredit;

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) {
    let color = Colour::FABLED_PINK;

    let credit_localised =
        match CreditLocalisedText::get_credit_localised(color, ctx, command).await {
            Ok(data) => data,
            Err(_) => return,
        };
    let mut desc: String = "".to_string();
    for x in credit_localised.list {
        desc += x.text.as_str()
    }
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.embed(|m| {
                        m.title(&credit_localised.title)
                            // Add a timestamp for the current time
                            // This also accepts a rfc3339 Timestamp
                            .timestamp(Timestamp::now())
                            .color(color)
                            .description(desc)
                    })
                })
        })
        .await
    {
        println!("Cannot respond to slash command: {}", why);
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    let credits = RegisterLocalisedCredit::get_credit_register_localised().unwrap();
    let command = command.name("credit").description("List of credit");
    for credit in credits.values() {
        command
            .name_localized(&credit.code, &credit.name)
            .description_localized(&credit.code, &credit.desc);
    }
    command
}
