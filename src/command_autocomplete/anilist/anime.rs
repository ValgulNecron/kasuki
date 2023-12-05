use crate::anilist_struct::autocomplete::media::MediaPageWrapper;
use serenity::all::{AutocompleteChoice, CommandInteraction, Context, Interaction};
use serenity::builder::CreateAutocompleteResponse;

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
        choices.push(AutocompleteChoice::new(title, data.id))
    }

    let builder = CreateAutocompleteResponse::new().set_choices(choices);
    let interaction_id = interaction.id();
    let interaction_token = interaction.token();
    let test = ctx
        .http
        .create_interaction_response(
            interaction_id,
            interaction_token,
            (&builder).into(),
            Vec::new(),
        )
        .await;
    println!("{:?}", test);
}
