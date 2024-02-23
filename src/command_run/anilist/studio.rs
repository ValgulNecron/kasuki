use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp,
};

use crate::anilist_struct::run::studio::StudioWrapper;
use crate::command_run::get_option::get_option_map_string;
use crate::constant::COLOR;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::lang_struct::anilist::studio::load_localization_studio;

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let map = get_option_map_string(command_interaction);
    let value = map.get(&String::from("studio")).ok_or(AppError::new(
        String::from("There is no option"),
        ErrorType::Option,
        ErrorResponseType::Message,
    ))?;
    let data: StudioWrapper = if value.parse::<i32>().is_ok() {
        StudioWrapper::new_studio_by_id(value.parse().unwrap()).await?
    } else {
        StudioWrapper::new_studio_by_search(value).await?
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
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {}", e),
                ErrorType::Command,
                ErrorResponseType::Message,
            )
        })
}
