use serenity::all::{
    CommandDataOption, CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp,
};

use crate::anilist_struct::run::studio::StudioWrapper;
use crate::common::get_option_value::get_option;
use crate::constant::COLOR;
use crate::error_enum::AppError;
use crate::error_enum::AppError::CommandSendingError;
use crate::lang_struct::anilist::studio::load_localization_studio;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), AppError> {
    let value = get_option(options);
    let data: StudioWrapper = if value.parse::<i32>().is_ok() {
        StudioWrapper::new_studio_by_id(value.parse().unwrap()).await?
    } else {
        StudioWrapper::new_studio_by_search(&value).await?
    };

    let guild_id = match command_interaction.guild_id {
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
        let text = format!("[{}/{}]({})", rj, en, m.site_url);
        content.push_str(text.as_str());
        content.push('\n')
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

    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|e| CommandSendingError(format!("Error while sending the command {}", e)))
}
