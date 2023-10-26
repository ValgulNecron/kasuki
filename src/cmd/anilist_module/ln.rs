use crate::function::anilist::command_media::embed;
use crate::structure::anilist::media::struct_autocomplete_media::MediaPageWrapper;
use crate::structure::register::anilist::struct_ln_register::RegisterLocalisedLN;
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
    embed(options, ctx, command, "NOVEL").await;
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    let lns = RegisterLocalisedLN::get_ln_register_localised().unwrap();
    let command = command
        .name("ln")
        .description("Info of a light novel")
        .create_option(|option| {
            let option = option
                .name("ln_name")
                .description("Name of the light novel you want to check")
                .kind(CommandOptionType::String)
                .required(true)
                .set_autocomplete(true);
            for ln in lns.values() {
                option
                    .name_localized(&ln.code, &ln.option1)
                    .description_localized(&ln.code, &ln.option1_desc);
            }
            option
        });
    for ln in lns.values() {
        command
            .name_localized(&ln.code, &ln.name)
            .description_localized(&ln.code, &ln.desc);
    }
    command
}

pub async fn autocomplete(ctx: Context, command: AutocompleteInteraction) {
    let search = &command.data.options.first().unwrap().value;
    if let Some(search) = search {
        let data = MediaPageWrapper::new_autocomplete_ln(search, 8, "MANGA", "NOVEL").await;
        let choices = data.get_choices();
        // doesn't matter if it errors
        _ = command
            .create_autocomplete_response(ctx.http, |response| {
                response.set_choices(choices.clone())
            })
            .await;
    }
}
