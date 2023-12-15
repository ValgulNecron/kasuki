use crate::anilist_struct::run::user::{get_color, get_user_url, UserWrapper};
use crate::constant::{COMMAND_SENDING_ERROR, OPTION_ERROR};
use crate::error_enum::AppError;
use crate::error_enum::AppError::NoCommandOption;
use crate::lang_struct::anilist::register::load_localization_register;
use crate::sqls::general::data::set_registered_user;
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

    let data: UserWrapper = if value.parse::<i32>().is_ok() {
        UserWrapper::new_user_by_id(value.parse().unwrap()).await?
    } else {
        UserWrapper::new_user_by_search(value).await?
    };

    let guild_id = match command.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let register_localised = load_localization_register(guild_id).await?;

    let user_data = data.data.user.clone();

    let user_id = &command.user.id.to_string();
    let username = &command.user.name;

    set_registered_user(user_id, &user_data.id.unwrap_or(0).to_string()).await?;

    let desc = register_localised
        .desc
        .replace("$user$", username.as_str())
        .replace("$id$", user_id)
        .replace(
            "$anilist$",
            user_data.name.clone().unwrap_or(String::new()).as_str(),
        );

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(get_color(user_data.clone()))
        .title(user_data.name.unwrap_or(String::new()))
        .url(get_user_url(&user_data.id.unwrap_or(0)))
        .thumbnail(user_data.avatar.large.unwrap())
        .description(desc);

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command
        .create_response(&ctx.http, builder)
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())
}
