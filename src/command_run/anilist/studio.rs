use serenity::all::{
    CommandDataOption, CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp,
};

use crate::anilist_struct::run::studio::StudioWrapper;
use crate::constant::{COLOR, COMMAND_SENDING_ERROR};
use crate::error_enum::AppError;
use crate::lang_struct::anilist::studio::load_localization_studio;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<(), AppError> {
    let mut value = String::new();
    for option_data in options {
        if option_data.name.as_str() != "type" {
            let option_value = option_data.value.as_str().clone().unwrap();
            value = option_value.to_string().clone()
        }
    }

    let data: StudioWrapper = if value.parse::<i32>().is_ok() {
        StudioWrapper::new_studio_by_id(value.parse().unwrap()).await?
    } else {
        StudioWrapper::new_studio_by_search(&value).await?
    };

    let guild_id = match command.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let studio = data.data.studio.clone();
    let studio_localised = load_localization_studio(guild_id).await?;

    let mut content = String::new();
    for m in studio.media.nodes {
        let title = m.title.clone();
        let rj = title.romaji;
        let en = title.user_preferred;
        content.push_str(rj.as_str());
        content.push_str("/");
        content.push_str(en.as_str());
        content.push_str("\n");
    }

    let desc = studio_localised
        .desc
        .replace("$id$", studio.id.to_string().as_str())
        .replace("$fav$", studio.favourites.to_string().as_str())
        .replace(
            "$animation$",
            studio.is_animation_studio.to_string().as_str(),
        )
        .replace("$list$", content.as_str());

    let name = studio.name;

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .description(desc)
        .title(name)
        .url(studio.site_url);
    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command
        .create_response(&ctx.http, builder)
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())
}
