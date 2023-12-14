use crate::anilist_struct::run::user::UserWrapper;
use crate::constant::{COLOR, COMMAND_SENDING_ERROR, OPTION_ERROR};
use crate::error_enum::AppError;
use crate::error_enum::AppError::NoCommandOption;
use crate::lang_struct::anilist::compare::load_localization_compare;
use serenity::all::{CommandDataOption, CommandDataOptionValue, CommandInteraction, Context, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage, Timestamp};

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

    let option2 = &options.get(1).ok_or(OPTION_ERROR.clone())?.value;

    let value2 = match option2 {
        CommandDataOptionValue::String(lang) => lang,
        _ => {
            return Err(NoCommandOption(String::from(
                "The command contain no option.",
            )));
        }
    };

    let data: UserWrapper = if value.parse::<i32>().is_ok() {
        UserWrapper::new_user_by_id(value.parse().unwrap()).await?
    } else {
        UserWrapper::new_user_by_search(value).await?
    };

    let data2: UserWrapper = if value2.parse::<i32>().is_ok() {
        UserWrapper::new_user_by_id(value2.parse().unwrap()).await?
    } else {
        UserWrapper::new_user_by_search(value2).await?
    };
    let guild_id = match command.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let compare_localised = load_localization_compare(guild_id).await?;

    let user = data.data.user.clone();
    let user2 = data2.data.user.clone();
    let username = user.name.clone().unwrap_or(String::new());
    let username2 = user2.name.clone().unwrap_or(String::new());

    let mut desc = String::new();

    if &user.statistics.anime.count.unwrap_or(0) > &user2.statistics.anime.count.unwrap_or(0) {
        desc.push_str(
            compare_localised
                .more_anime
                .replace("$greater$", username.as_str())
                .replace("$lesser$", username2.as_str())
                .as_str(),
        )
    } else if user.statistics.anime.count.unwrap_or(0) < user2.statistics.anime.count.unwrap_or(0) {
        desc.push_str(
            compare_localised
                .more_anime
                .replace("$greater$", username2.as_str())
                .replace("$lesser$", username.as_str())
                .as_str(),
        )
    } else {
        desc.push_str(
            compare_localised
                .same_anime
                .replace("$2$", username2.as_str())
                .replace("$1$", username.as_str())
                .as_str(),
        )
    }

     if user.statistics.anime.minutes_watched.unwrap_or(0) > user2.statistics.anime.minutes_watched.unwrap_or(0) {
        desc.push_str(
            compare_localised
                .more_watch_time
                .replace("$greater$", username.as_str())
                .replace("$lesser$", username2.as_str())
                .as_str(),
        )
    } else if &user.statistics.anime.minutes_watched.unwrap_or(0) < &user2.statistics.anime.minutes_watched.unwrap_or(0) {
        desc.push_str(
            compare_localised
                .more_watch_time
                .replace("$greater$", username2.as_str())
                .replace("$lesser$", username.as_str())
                .as_str(),
        )
    } else {
        desc.push_str(
            compare_localised
                .same_watch_time
                .replace("$2$", username2.as_str())
                .replace("$1$", username.as_str())
                .as_str(),
        )
    }

    let tag = user.statistics.anime.tags[0].tag.name.clone().unwrap_or(String::new());
    let tag2 = user2.statistics.anime.tags[0].tag.name.clone().unwrap_or(String::new());

    let diff = tag != tag2;

    if diff {
        desc.push_str(
            compare_localised
                .tag_anime
                .replace("$1$", username.as_str())
                .replace("$2$", username2.as_str())
                .replace("$1a$",tag.as_str())
                .replace("$2a$", tag2.as_str())
                .as_str(),
        )
    } else {
        desc.push_str(
            compare_localised
                .same_tag_anime
                .replace("$1$", username.as_str())
                .replace("$2$", username2.as_str())
                .replace("$1a$", tag.as_str())
                .as_str()
        )
    }

    let genre = user.statistics.anime.genres[0].genre.clone().unwrap_or(String::new());
    let genre2 = user2.statistics.anime.genres[0].genre.clone().unwrap_or(String::new());

    let diff = genre != genre2;

    if diff {
        desc.push_str(
            compare_localised
                .genre_anime
                .replace("$1$", username.as_str())
                .replace("$2$", username2.as_str())
                .replace("$1a$", genre.as_str())
                .replace("$2a$", genre2.as_str())
                .as_str(),
        )
    } else {
        desc.push_str(
            compare_localised
                .same_tag_anime
                .replace("$1$", username.as_str())
                .replace("$2$", username2.as_str())
                .replace("$1a$", genre.as_str())
                .as_str()
        )
    }

    if user.statistics.manga.count.unwrap_or(0) > user2.statistics.manga.count.unwrap_or(0) {
        desc.push_str(
            compare_localised
                .more_manga
                .replace("$greater$", username.as_str())
                .replace("$lesser$", username2.as_str())
                .as_str(),
        )
    } else if user.statistics.manga.count.unwrap_or(0) < user2.statistics.manga.count.unwrap_or(0) {
        desc.push_str(
            compare_localised
                .more_manga
                .replace("$greater$", username2.as_str())
                .replace("$lesser$", username.as_str())
                .as_str(),
        )
    } else {
        desc.push_str(
            compare_localised
                .same_manga
                .replace("$2$", username2.as_str())
                .replace("$1$", username.as_str())
                .as_str(),
        )
    }

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .description(desc);

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command
        .create_response(&ctx.http, builder)
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())
}
