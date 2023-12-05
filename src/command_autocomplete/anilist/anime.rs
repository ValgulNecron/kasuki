use crate::anilist_struct::autocomplete::media::MediaPageWrapper;
use reqwest::Client;
use serde_json::json;
use serenity::all::{AutocompleteChoice, CommandInteraction, Context, Interaction};
use serenity::builder::CreateAutocompleteResponse;
use serenity::http::Request;

pub async fn autocomplete(ctx: Context, interaction: Interaction, command: CommandInteraction) {
    let search = &command
        .data
        .options
        .first()
        .unwrap()
        .value
        .as_str()
        .unwrap();
    let anime = MediaPageWrapper::new_autocomplete_anime(&search.to_string(), 8, "ANIME").await;
    let mut choices = Vec::new();
    for page in anime.data.page.media.unwrap() {
        let data = page.unwrap();
        let title_data = data.title.unwrap();
        let english = title_data.english;
        let romaji = title_data.romaji;
        let title = english.unwrap_or(romaji);
        choices.push(json!({
            "type": 3,
            "name": title,
            "value": data.id,
            "focused": true,
        }));
    }
    let interaction_id = interaction.id();
    let interaction_token = interaction.token();
    let interaction_version = command.version;
    let interaction_name = command.data.name.as_str();

    let autocomplete_interaction = json!({
      "type": 4,
      "data": {
          "id": interaction_id,
          "name": interaction_name,
          "type": 1,
          "version": interaction_version,
          "options": choices
      }
    });

    let mut request = Request::new("POST");
    request.set_url(
        "https://discord.com/api/v8/interactions/{}/{}/callback",
        interaction_id,
        interaction_token,
    );
    request.set_body(&json!(autocomplete_interaction));

    let client = Client::new();
    let response = client.send_request(&request).await;

    match response {
        Ok(_) => println!("Autocomplete response sent successfully"),
        Err(e) => println!("Failed to send autocomplete response: {}", e),
    }
}
