use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde_json::json;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    ApplicationCommandInteraction, CommandDataOption,
};
use serenity::model::Timestamp;
use serenity::utils::Colour;

use crate::cmd::anilist_module::struct_staff::*;
use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::html_parser::convert_to_markdown;
use crate::cmd::general_module::lang_struct::StaffLocalisedText;
use crate::cmd::general_module::request::make_request;

const QUERY: &str = "
query ($name: String, $limit1: Int = 5, $limit2: Int = 15) {
	Staff(search: $name){
    name {
      full
      native
    }
    id
    languageV2
    image {
      large
    }
    description
    primaryOccupations
    gender
    dateOfBirth {
      year
      month
      day
    }
    dateOfDeath {
      year
      month
      day
    }
    age
    yearsActive
    homeTown
    siteUrl
    staffMedia(perPage: $limit1){
      edges{
        node {
          title {
            romaji
            english
          }
        }
        roleNotes
        relationType
        staffRole
      }
    }
    characters(perPage: $limit2) {
      nodes {
        name {
          full
        }
        image {
          large
        }
      }
    }
  }
}
";

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
    if let CommandDataOptionValue::String(staff) = option {
        let json = json!({"query": QUERY, "variables": {"name": staff}});
        let resp = make_request(json).await;

        // Get json
        let data: StaffData = match serde_json::from_str(&resp) {
            Ok(result) => result,
            Err(e) => {
                println!("Failed to parse json: {}", e);
                return "Error: Failed to retrieve user data".to_string();
            }
        };
        let staff_url = format!("https://anilist.co/staff/{}", &data.data.staff.id);
        let color = Colour::FABLED_PINK;

        let staff_name = format!(
            "{}/{}",
            &data
                .data
                .staff
                .name
                .native
                .unwrap_or_else(|| "N/A".to_string()),
            &data
                .data
                .staff
                .name
                .full
                .unwrap_or_else(|| "N/A".to_string())
        );

        let mut desc = data.data.staff.description.clone();

        desc = convert_to_markdown(desc);

        let birth = format!(
            "{}/{}/{}",
            &data.data.staff.date_of_birth.month.unwrap_or_else(|| 0),
            &data.data.staff.date_of_birth.day.unwrap_or_else(|| 0),
            &data.data.staff.date_of_birth.year.unwrap_or_else(|| 0)
        );
        let death = format!(
            "{}/{}/{}",
            &data.data.staff.date_of_death.month.unwrap_or_else(|| 0),
            &data.data.staff.date_of_death.day.unwrap_or_else(|| 0),
            &data.data.staff.date_of_death.year.unwrap_or_else(|| 0)
        );

        let image = &data.data.staff.image.large;
        let lang = &data.data.staff.language_v2;

        let hometown = &data
            .data
            .staff
            .home_town
            .unwrap_or_else(|| "N/A".to_string());

        let max_limit = 5;
        let limited_occupations: Vec<String> = data
            .data
            .staff
            .primary_occupations
            .iter()
            .take(max_limit)
            .cloned()
            .collect();
        let occupations_string = limited_occupations.join(", ");

        let formatted_edges_role: Vec<String> = data
            .data
            .staff
            .staff_media
            .edges
            .iter()
            .map(|edge| format_edge(edge))
            .collect();
        let result_role: String = formatted_edges_role.join("\n");

        let formatted_nodes_va: Vec<String> = data
            .data
            .staff
            .characters
            .nodes
            .iter()
            .map(|character| format_node(character))
            .collect();
        let result_va: String = formatted_nodes_va.join(",\n");

        let mut file = File::open("lang_file/anilist/staff.json").expect("Failed to open file");
        let mut json = String::new();
        file.read_to_string(&mut json).expect("Failed to read file");

        let json_data: HashMap<String, StaffLocalisedText> =
            serde_json::from_str(&json).expect("Failed to parse JSON");

        let guild_id = command.guild_id.unwrap().0.to_string().clone();
        let lang_choice = get_guild_langage(guild_id).await;

        if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
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
                                        (&localised_text.desc_title, format!("{}", desc), false),
                                        (
                                            &"".to_string(),
                                            format!(
                                                "{}{}{}{}{}{}{}{}{}{}",
                                                &localised_text.date_of_birth,
                                                birth,
                                                &localised_text.date_of_death,
                                                death,
                                                &localised_text.hometown,
                                                hometown,
                                                &localised_text.primary_language,
                                                lang,
                                                &localised_text.primary_occupation,
                                                occupations_string
                                            ),
                                            false,
                                        ),
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
                println!("Cannot respond to slash command: {}", why);
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
        })
}

fn format_edge(edge: &Edge) -> String {
    let title = match &edge.node.title.romaji {
        Some(romaji) => romaji.clone(),
        None => match &edge.node.title.english {
            Some(english) => english.clone(),
            None => "".to_string(),
        },
    };
    let staff_role = &edge.staff_role;
    format!("{} ({})", title, staff_role)
}

fn format_node(character: &Character) -> String {
    let name_natif = character
        .name
        .native
        .clone()
        .unwrap_or_else(|| "N/A".to_string());
    let name_full = character
        .name
        .full
        .clone()
        .unwrap_or_else(|| "N/A".to_string());
    let name = format!("{} / {}", name_natif, name_full);
    format!("{}", name)
}
