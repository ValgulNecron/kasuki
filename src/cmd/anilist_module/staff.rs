use std::u32;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::ChannelId;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{ApplicationCommandInteraction, CommandDataOption};
use serenity::model::Timestamp;
use serenity::utils::Colour;

#[derive(Debug, Deserialize, Serialize)]
struct Name {
    full: Option<String>,
    native: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Image {
    large: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Date {
    year: Option<i32>,
    month: Option<i32>,
    day: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Title {
    romaji: Option<String>,
    english: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Node {
    title: Title,
}

#[derive(Debug, Deserialize, Serialize)]
struct StaffMedia {
    edges: Vec<Edge>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Edge {
    node: Node,
    #[serde(rename = "roleNotes")]
    role_notes: Option<String>,
    #[serde(rename = "relationType")]
    relation_type: Option<String>,
    #[serde(rename = "staffRole")]
    staff_role: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Character {
    name: Name,
    image: Image,
}

#[derive(Debug, Deserialize, Serialize)]
struct Characters {
    nodes: Vec<Character>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Staff {
    name: Name,
    id: i32,
    #[serde(rename = "languageV2")]
    language_v2: String,
    image: Image,
    description: String,
    #[serde(rename = "primaryOccupations")]
    primary_occupations: Vec<String>,
    gender: String,
    #[serde(rename = "dateOfBirth")]
    date_of_birth: Date,
    #[serde(rename = "dateOfDeath")]
    date_of_death: Date,
    age: Option<i32>,
    #[serde(rename = "yearsActive")]
    years_active: Vec<i32>,
    #[serde(rename = "homeTown")]
    home_town: Option<String>,
    #[serde(rename = "siteUrl")]
    site_url: String,
    #[serde(rename = "staffMedia")]
    staff_media: StaffMedia,
    characters: Characters,
}

#[derive(Debug, Deserialize, Serialize)]
struct Data {
    #[serde(rename = "Staff")]
    staff: Staff,
}

#[derive(Debug, Deserialize, Serialize)]
struct Response {
    data: Data,
}

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

pub async fn run(options: &[CommandDataOption], ctx: &Context, command: &ApplicationCommandInteraction) -> String {
    let option = options
        .get(0)
        .expect("Expected name option")
        .resolved
        .as_ref()
        .expect("Expected name object");
    if let CommandDataOptionValue::String(staff) = option {
        let client = Client::new();
        let json = json!({"query": QUERY, "variables": {"name": staff}});
        let resp = client.post("https://graphql.anilist.co/")
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(json.to_string())
            .send()
            .await
            .unwrap()
            .text()
            .await;

        // Get json
        let data: Response = match serde_json::from_str(&resp.unwrap()) {
            Ok(result) => result,
            Err(e) => {
                println!("Failed to parse json: {}", e);
                return "Error: Failed to retrieve user data".to_string();
            }
        };
        let staff_url = format!("https://anilist.co/staff/{}", &data.data.staff.id);
        let mut color = Colour::FABLED_PINK;

        let staff_name = format!("{}/{}", &data.data.staff.name.native.
            unwrap_or_else(|| "N/A".to_string()), &data.data.staff.name.full.unwrap_or_else(|| "N/A".to_string()));

        let desc = &data.data.staff.description;

        let birth = format!("{}/{}/{}", &data.data.staff.date_of_birth.month
            .unwrap_or_else(|| 0), &data.data.staff.date_of_birth.day.unwrap_or_else(|| 0),
                            &data.data.staff.date_of_birth.year.unwrap_or_else(|| 0));
        let death = format!("{}/{}/{}", &data.data.staff.date_of_death.month
            .unwrap_or_else(|| 0), &data.data.staff.date_of_death.day.unwrap_or_else(|| 0),
                            &data.data.staff.date_of_death.year.unwrap_or_else(|| 0));

        let image = &data.data.staff.image.large;
        let lang = &data.data.staff.language_v2;

        let hometown = &data.data.staff.home_town.unwrap_or_else(|| "N/A".to_string());

        let max_limit = 5;
        let limited_occupations: Vec<String> = data.data.staff.primary_occupations.iter().take(max_limit)
            .cloned().collect();
        let occupations_string = limited_occupations.join(", ");

        let formatted_edges_role: Vec<String> = data.data.staff.staff_media.edges.iter()
            .map(|edge| format_edge(edge)).collect();
        let result_role: String = formatted_edges_role.join("\n");

        let formatted_nodes_va: Vec<String> = data.data.staff.characters.nodes.iter()
            .map(|character| format_node(character)).collect();
        let result_va: String = formatted_nodes_va.join(",\n");

        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message.embed(
                        |m| {
                            m.title(staff_name)
                                .timestamp(Timestamp::now())
                                .color(color)
                                .fields(vec![
                                    ("Info".to_string(), format!("{}", desc), false),
                                    ("".to_string(), format!("Date of birth: {}. \n Date of death: {}. \
                                    \n Hometown: {}. Primary langage: {}. \n Occupation: {}", birth, death, hometown, lang, occupations_string), false),
                                    ("MANGA".to_string(), format!("{}", result_role), true),
                                    ("VA".to_string(), format!("{}", result_va), true),
                                ])
                                .url(staff_url)
                                .image(image)
                        })
                    )
            })
            .await
        {
            println!("Cannot respond to slash command: {}", why);
        }
    }
    return "good".to_string();
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("staff").description("Get info of a staff").create_option(
        |option| {
            option
                .name("staff_name")
                .description("Name of the staff you want info about.")
                .kind(CommandOptionType::String)
                .required(true)
        },
    )
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
    let name_natif = character.name.native.clone().unwrap_or_else(|| "N/A".to_string());
    let name_full = character.name.full.clone().unwrap_or_else(|| "N/A".to_string());
    let name = format!("{}/{}", name_natif, name_full);
    format!("{}", name)
}