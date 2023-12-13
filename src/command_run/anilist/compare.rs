use crate::anilist_struct::run::user::UserWrapper;
use crate::constant::OPTION_ERROR;
use crate::error_enum::AppError;
use crate::error_enum::AppError::NoCommandOption;
use crate::lang_struct::anilist::compare::load_localization_compare;
use serenity::all::{CommandDataOption, CommandDataOptionValue, CommandInteraction, Context};

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

    let option2 = &options.get(0).ok_or(OPTION_ERROR.clone())?.value;

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
        UserWrapper::new_user_by_id(value.parse().unwrap()).await?
    } else {
        UserWrapper::new_user_by_search(value).await?
    };
    let guild_id = match command.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let compare_localised = load_localization_compare(guild_id).await?;

    let user = data.data.user.clone();
    let user2 = data2.data.user.clone();

    let mut desc = String::new();

    if user.statistics.anime.count.unwrap_or(0) > user2.statistics.anime.count.unwrap_or(0) {
        desc.push_str(
            compare_localised
                .more_anime
                .replace("$greater$", user.name.unwrap_or(String::new()).as_str())
                .replace("$lesser$", user2.name.unwrap_or(String::new()).as_str())
                .as_str(),
        )
    } else if user.statistics.anime.count.unwrap_or(0) < user2.statistics.anime.count.unwrap_or(0) {
        desc.push_str(
            compare_localised
                .more_anime
                .replace("$greater$", user2.name.unwrap_or(String::new()).as_str())
                .replace("$lesser$", user.name.unwrap_or(String::new()).as_str())
                .as_str(),
        )
    } else {
        desc.push_str(
            compare_localised
                .same_anime
                .replace("$2$", user2.name.unwrap_or(String::new()).as_str())
                .replace("$1$", user.name.unwrap_or(String::new()).as_str())
                .as_str(),
        )
    }

    Ok(())
}
