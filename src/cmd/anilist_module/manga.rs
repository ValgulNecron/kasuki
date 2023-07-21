use serde_json::json;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::prelude::autocomplete::AutocompleteInteraction;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    ApplicationCommandInteraction, CommandDataOption,
};

use crate::cmd::anilist_module::command_media_ln::embed;
use crate::cmd::anilist_module::struct_autocomplete::AutocompleteOption;
use crate::cmd::anilist_module::struct_autocomplete_media::MediaPageWrapper;
use crate::cmd::general_module::request::make_request;

const QUERY_ID: &str = "
    query ($search: Int, $limit: Int = 5, $format: MediaFormat = NOVEL) {
		Media (id: $search, type: MANGA, format_not: $format){
    id
      description
    title{
      romaji
      english
    }
    type
    format
    source
    isAdult
    startDate {
      year
      month
      day
    }
    endDate {
      year
      month
      day
    }
    chapters
    volumes
    status
    season
    isLicensed
    coverImage {
      extraLarge
    }
    bannerImage
    genres
    tags {
      name
    }
    averageScore
    meanScore
    popularity
    favourites
    siteUrl
    staff(perPage: $limit) {
      edges {
        node {
          id
          name {
            full
            userPreferred
          }
        }
        id
        role
      }
    }
  }
}
";

const QUERY_STRING: &str = "
    query ($search: String, $limit: Int = 5, $format: MediaFormat = NOVEL) {
		Media (search: $search, type: MANGA, format_not: $format){
    id
      description
    title{
      romaji
      english
    }
    type
    format
    source
    isAdult
    startDate {
      year
      month
      day
    }
    endDate {
      year
      month
      day
    }
    chapters
    volumes
    status
    season
    isLicensed
    coverImage {
      extraLarge
    }
    bannerImage
    genres
    tags {
      name
    }
    averageScore
    meanScore
    popularity
    favourites
    siteUrl
    staff(perPage: $limit) {
      edges {
        node {
          id
          name {
            full
            userPreferred
          }
        }
        id
        role
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
    return embed(options, ctx, command, QUERY_ID, QUERY_STRING).await;
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("manga")
        .description("Info of a manga")
        .create_option(|option| {
            option
                .name("manga_name")
                .description("Name of the manga you want to check")
                .kind(CommandOptionType::String)
                .required(true)
                .set_autocomplete(true)
        })
}

pub async fn autocomplete(ctx: Context, command: AutocompleteInteraction) {
    let search = &command.data.options.first().unwrap().value;
    if let Some(search) = search {
        let query_str = "query($search: String, $type: MediaType, $count: Int, $format: MediaFormat) {
          Page(perPage: $count) {
		    media(search: $search, type: $type, format_not: $format) {
		      id
		      title {
		        romaji
		        english
		      }
			}
		  }
		}";
        let json = json!({"query": query_str, "variables": {
            "search": search,
            "type": "MANGA",
            "count": 8,
            "format": "NOVEL"
        }});

        let res = make_request(json).await;
        let data: MediaPageWrapper = serde_json::from_str(&res).unwrap();

        if let Some(media) = data.data.page.media {
            let suggestions: Vec<AutocompleteOption> = media
                .iter()
                .filter_map(|item| {
                    if let Some(item) = item {
                        Some(AutocompleteOption {
                            name: match &item.title {
                                Some(title) => {
                                    let english = title.english.clone();
                                    let romaji = title.romaji.clone();
                                    String::from(english.unwrap_or(romaji))
                                }
                                None => String::default(),
                            },
                            value: item.id.to_string(),
                        })
                    } else {
                        None
                    }
                })
                .collect();
            let choices = json!(suggestions);

            // doesn't matter if it errors
            _ = command
                .create_autocomplete_response(ctx.http, |response| response.set_choices(choices))
                .await;
        }
    }
}