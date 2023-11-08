use crate::function::anilist::command_media::embed;
use crate::structure::anilist::media::struct_autocomplete_media::MediaPageWrapper;
use crate::structure::register::anilist::struct_manga_register::RegisterLocalisedManga;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::prelude::autocomplete::AutocompleteInteraction;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    ApplicationCommandInteraction, CommandDataOption,
};

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    embed(options, ctx, command, "MANGA").await;
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    let mangas = RegisterLocalisedManga::get_manga_register_localised().unwrap();
    let command = command
        .name("manga")
        .description("Info of a manga")
        .create_option(|option| {
            let option = option
                .name("manga_name")
                .description("Name of the manga you want to check")
                .kind(CommandOptionType::String)
                .required(true)
                .set_autocomplete(true);
            for manga in mangas.values() {
                option
                    .name_localized(&manga.code, &manga.option1)
                    .description_localized(&manga.code, &manga.option1_desc);
            }
            option
        });
    for manga in mangas.values() {
        command
            .name_localized(&manga.code, &manga.name)
            .description_localized(&manga.code, &manga.desc);
    }
    command
}

pub async fn autocomplete(ctx: Context, command: AutocompleteInteraction) {
    let search = &command.data.options.first().unwrap().value;
    if let Some(search) = search {
        let data = MediaPageWrapper::new_autocomplete_manga(search, 8, "MANGA", "NOVEL").await;
        let choices = data.get_choices();
        // doesn't matter if it errors
        _ = command
            .create_autocomplete_response(ctx.http, |response| {
                response.set_choices(choices.clone())
            })
            .await;
    }
}
