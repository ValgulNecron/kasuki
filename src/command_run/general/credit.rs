use crate::error_enum::AppError;

pub async fn run() -> Result<(), AppError> {
    ctx: &Context, command: &ApplicationCommandInteraction) -> Result < (), AppError > {
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