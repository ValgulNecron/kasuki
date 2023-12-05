use crate::anilist_struct::autocomplete::media::MediaPageWrapper;
use serenity::all::{CommandDataOptionValue, CommandInteraction};

pub async fn autocomplete(command: CommandInteraction) {
    let search = &command.data.options.first().unwrap().value;
    if let CommandDataOptionValue::String(data) = search {
        let anime = MediaPageWrapper::new_autocomplete_anime(data, 8, "ANIME").await;
        println!("{:?}", anime.data.page)
    }
}
