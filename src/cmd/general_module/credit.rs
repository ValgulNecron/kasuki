use crate::constant::COLOR;
use crate::error_enum::AppError::LangageGuildIdError;
use crate::error_enum::{AppError, COMMAND_SENDING_ERROR};
use crate::structure::embed::general::struct_lang_credit::CreditLocalisedText;
use crate::structure::register::general::struct_credit_register::RegisterLocalisedCredit;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::Timestamp;

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> Result<(), AppError> {
    let guild_id = command
        .guild_id
        .ok_or(LangageGuildIdError(String::from(
            "Guild id for langage not found.",
        )))?
        .0
        .to_string();
    let credit_localised = CreditLocalisedText::get_credit_localised(guild_id).await?;
    let mut desc: String = "".to_string();
    for x in credit_localised.list {
        desc += x.text.as_str()
    }
    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.embed(|m| {
                        m.title(&credit_localised.title)
                            // Add a timestamp for the current time
                            // This also accepts a rfc3339 Timestamp
                            .timestamp(Timestamp::now())
                            .color(COLOR)
                            .description(desc)
                    })
                })
        })
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())
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
