use crate::anilist_struct::run::media::{
    embed_title, get_banner, get_genre, get_tag, get_thumbnail, get_url, media_info, MediaWrapper,
};
use crate::constant::{COLOR, COMMAND_SENDING_ERROR, OPTION_ERROR};
use crate::error_enum::AppError;
use crate::error_enum::AppError::{LangageGuildIdError, NoCommandOption};
use crate::lang_struct::anilist::media::load_localization_media;
use serenity::all::{
    CommandDataOption, CommandDataOptionValue, CommandInteraction, Context, CreateEmbed,
    CreateInteractionResponse, CreateInteractionResponseMessage, Timestamp,
};

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<(), AppError> {
    let option = &options.get(0).ok_or(OPTION_ERROR.clone())?.value;

    let value = match option {
        CommandDataOptionValue::String(lang) => lang,
        _ => {
            return Err(NoCommandOption(String::from(
                "The command contain no option.",
            )));
        }
    };

    let data: MediaWrapper = if value.parse::<i32>().is_ok() {
        MediaWrapper::new_anime_by_id(value.parse().unwrap()).await?
    } else {
        MediaWrapper::new_anime_by_search(value).await?
    };

    let guild_id = command
        .guild_id
        .ok_or(LangageGuildIdError(String::from(
            "Guild id for langage not found.",
        )))?
        .to_string();

    let anime_localised = load_localization_media(guild_id).await?;

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .description(media_info(&data, &anime_localised))
        .title(embed_title(&data))
        .url(get_url(&data))
        .field(&anime_localised.field1_title, get_genre(&data), true)
        .field(&anime_localised.field2_title, get_tag(&data), true)
        .thumbnail(get_thumbnail(&data))
        .image(get_banner(&data));

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command
        .create_response(&ctx.http, builder)
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())
}
