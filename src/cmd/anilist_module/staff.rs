use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde_json::json;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::autocomplete::AutocompleteInteraction;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    ApplicationCommandInteraction, CommandDataOption,
};
use serenity::model::Timestamp;
use serenity::utils::Colour;

use crate::cmd::anilist_module::struct_autocomplete_staff::StaffPageWrapper;
use crate::cmd::anilist_module::struct_staff::*;
use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::lang_struct::StaffLocalisedText;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> String {
    let option = options
        .get(0)
        .expect("Expected name option")
        .resolved
        .as_ref()
        .expect("Expected name object");
    if let CommandDataOptionValue::String(value) = option {
        let data;
        if match value.parse::<i32>() {
            Ok(_) => true,
            Err(_) => false,
        } {
            data = match StaffWrapper::new_anime_by_id(value.parse().unwrap()).await {
                Ok(user_wrapper) => user_wrapper,
                Err(error) => return error,
            }
        } else {
            data = match StaffWrapper::new_anime_by_search(value).await {
                Ok(user_wrapper) => user_wrapper,
                Err(error) => return error,
            }
        }

        let staff_url = data.get_url();
        let color = Colour::FABLED_PINK;

        let staff_name = data.get_name();

        let image = data.get_image();

        let result_role: String = data.format_role();

        let result_va: String = data.format_va();

        let mut file = File::open("lang_file/anilist/staff.json").expect("Failed to open file");
        let mut json = String::new();
        file.read_to_string(&mut json).expect("Failed to read file");

        let json_data: HashMap<String, StaffLocalisedText> =
            serde_json::from_str(&json).expect("Failed to parse JSON");

        let guild_id = command.guild_id.unwrap().0.to_string().clone();
        let lang_choice = get_guild_langage(guild_id).await;

        if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
            let desc = data.get_desc(localised_text);

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.embed(|m| {
                                m.title(staff_name)
                                    .timestamp(Timestamp::now())
                                    .color(color)
                                    .fields(vec![
                                        (&localised_text.desc_title, desc, false),
                                        (&localised_text.media, format!("{}", result_role), true),
                                        (&localised_text.va, format!("{}", result_va), true),
                                    ])
                                    .url(staff_url)
                                    .image(image)
                            })
                        })
                })
                .await
            {
                println!("{}: {}", localised_text.error_slash_command, why);
            }
        } else {
            return "Language not found".to_string();
        }
    }
    return "good".to_string();
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("staff")
        .description("Get info of a staff")
        .create_option(|option| {
            option
                .name("staff_name")
                .description("Name of the staff you want info about.")
                .kind(CommandOptionType::String)
                .required(true)
                .set_autocomplete(true)
        })
}

pub async fn autocomplete(ctx: Context, command: AutocompleteInteraction) {
    let search = &command.data.options.first().unwrap().value;
    if let Some(search) = search {
        let data = StaffPageWrapper::new_autocomplete_staff(search, 8).await;

        let choices = data.get_choice();
        // doesn't matter if it errors
        let choices_json = json!(choices);
        _ = command
            .create_autocomplete_response(ctx.http.clone(), |response| {
                response.set_choices(choices_json)
            })
            .await;
    }
}
