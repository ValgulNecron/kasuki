use crate::anilist_struct::autocomplete::media::MediaPageWrapper;
use serenity::all::{CommandInteraction, Context};

pub async fn autocomplete(ctx: Context, command: CommandInteraction) {
    let search = &command.data.options.first().unwrap().value;
    if let Some(search) = search {
        let data = MediaPageWrapper::new_autocomplete_anime(search, 8, "ANIME").await;
        let choices = data.get_choices();
        // doesn't matter if it errors
    }
}
