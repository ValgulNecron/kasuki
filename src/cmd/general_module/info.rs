use crate::structure::embed::general::struct_lang_info::InfoLocalisedText;
use crate::structure::register::general::struct_info_register::RegisterLocalisedInfo;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::component::ButtonStyle;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::Timestamp;
use serenity::utils::Colour;

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) {
    let color = Colour::FABLED_PINK;
    let localised_text = match InfoLocalisedText::get_info_localised(color, ctx, command).await {
        Ok(data) => data,
        Err(_) => return,
    };

    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.embed(
                    |m| {
                        m.title(&localised_text.title)
                            .description(&localised_text.description)
                            .footer(|f| f.text(&localised_text.footer))
                            .timestamp(Timestamp::now())
                            .color(color)
                    })
                    .components(|components| {
                        components.create_action_row(|row| {
                            row.create_button(|button| {
                                button.label(&localised_text.button_see_on_github)
                                    .url("https://github.com/ValgulNecron/DIscordAnilistBotRS")
                                    .style(ButtonStyle::Link)
                            })
                                .create_button(|button| {
                                    button.label(&localised_text.button_official_website)
                                        .url("https://kasuki.valgul.moe/")
                                        .style(ButtonStyle::Link)
                                })
                        })
                            .create_action_row(|button| {
                                button.create_button(|button| {
                                    button.label(&localised_text.button_official_discord)
                                        .url("https://discord.gg/dWGU6mkw7J")
                                        .style(ButtonStyle::Link)
                                })
                                    .create_button(|button| {
                                        button.label(&localised_text.button_add_the_bot)
                                            .url("https://discord.com/api/oauth2/authorize?client_id=923286536445894697&permissions=533113194560&scope=bot")
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
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    let infos = RegisterLocalisedInfo::get_info_register_localised().unwrap();
    let command = command
        .name("info")
        .description("Get information on the bot");
    for info in infos.values() {
        command
            .name_localized(&info.code, &info.name)
            .description_localized(&info.code, &info.desc);
    }
    command
}
