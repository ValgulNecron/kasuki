use crate::anilist_struct::autocomplete::media::MediaPageWrapper;
use serenity::all::{AutocompleteChoice, CommandInteraction, Context, CreateInteractionResponse};
use serenity::builder::CreateAutocompleteResponse;

pub async fn autocomplete(ctx: Context, command: CommandInteraction) {
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
        choices.push(AutocompleteChoice::new(title, data.id.to_string()))
    }

    let data = CreateAutocompleteResponse::new().set_choices(choices);
    let builder = CreateInteractionResponse::Autocomplete(data);

    let test = command.create_response(ctx.http, builder).await;
    println!("{:?}", test)
}
