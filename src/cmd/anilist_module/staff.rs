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
    roleNotes: Option<String>,
    relationType: Option<String>,
    staffRole: String,
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
    languageV2: String,
    image: Image,
    description: String,
    primaryOccupations: Vec<String>,
    gender: String,
    dateOfBirth: Date,
    dateOfDeath: Date,
    age: Option<i32>,
    yearsActive: Vec<i32>,
    homeTown: Option<String>,
    siteUrl: String,
    staffMedia: StaffMedia,
    characters: Characters,
}

#[derive(Debug, Deserialize, Serialize)]
struct Data {
    Staff: Staff,
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
        let staff_url = format!("https://anilist.co/staff/{}", &data.data.Staff.id);
        let mut color = Colour::FABLED_PINK;

        let staff_name = format!("{}/{}", &data.data.Staff.name.native.
            unwrap_or_else(|| "N/A".to_string()), &data.data.Staff.name.full.unwrap_or_else(|| "N/A".to_string()));

        let desc = &data.data.Staff.description;

        let birth = format!("{}/{}/{}", &data.data.Staff.dateOfBirth.month
            .unwrap_or_else(|| 0), &data.data.Staff.dateOfBirth.day.unwrap_or_else(|| 0),
                            &data.data.Staff.dateOfBirth.year.unwrap_or_else(|| 0));
        let death = format!("{}/{}/{}", &data.data.Staff.dateOfDeath.month
            .unwrap_or_else(|| 0), &data.data.Staff.dateOfDeath.day.unwrap_or_else(|| 0),
                            &data.data.Staff.dateOfDeath.year.unwrap_or_else(|| 0));

        let image = &data.data.Staff.image.large;
        let lang = &data.data.Staff.languageV2;

        let hometown = &data.data.Staff.homeTown.unwrap_or_else(|| "N/A".to_string());

        let max_limit = 5;
        let limited_occupations: Vec<String> = data.data.Staff.primaryOccupations.iter().take(max_limit)
            .cloned().collect();
        let occupations_string = limited_occupations.join(", ");

        let formatted_edges_role: Vec<String> = data.data.Staff.staffMedia.edges.iter()
            .map(|edge| format_edge(edge)).collect();
        let result_role: String = formatted_edges_role.join("\n");

        let formatted_nodes_va: Vec<String> = data.data.Staff.characters.nodes.iter()
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
    let staff_role = &edge.staffRole;
    format!("{} ({})", title, staff_role)
}

fn format_node(character: &Character) -> String {
    let name_natif = character.name.native.clone().unwrap_or_else(|| "N/A".to_string());
    let name_full = character.name.full.clone().unwrap_or_else(|| "N/A".to_string());
    let name = format!("{}/{}", name_natif, name_full);
    format!("{}", name)
}