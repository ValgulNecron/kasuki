use crate::anilist_struct::run::user::{get_banner, get_color, get_user_url, User, UserWrapper};
use crate::constant::{COLOR, COMMAND_SENDING_ERROR};
use crate::error_enum::AppError;
use crate::lang_struct::anilist::level::load_localization_level;
use crate::lang_struct::anilist::user::load_localization_user;
use serenity::all::{
    CommandDataOption, CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp,
};

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<(), AppError> {
}

pub async fn send_embed(
    ctx: &Context,
    command: &CommandInteraction,
    data: UserWrapper,
) -> Result<(), AppError> {
    let guild_id = match command.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let level_localised = load_localization_level(guild_id).await?;

    let user = data.data.user.clone();

    let (actual, next) = get_level_progression(user.clone());

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(get_color(user.clone()))
        .title(user.name.unwrap_or(String::new()))
        .url(get_user_url(&user.id.unwrap_or(0)))
        .image(get_banner(&user.id.unwrap_or(0)))
        .thumbnail(user.avatar.large.unwrap())
        .description(
            level_localised
                .desc
                .replace("$username$", &user.name.unwrap().as_str())
                .replace("$level$", get_level(user.clone()).as_str())
                .replace("$xp$", get_total_xp(user.clone()).as_str())
                .replace("$actual$")
                .replace("$next$"),
        );

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command
        .create_response(&ctx.http, builder)
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())
}

fn get_level(user: User) -> String {}
