use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage, Timestamp, User, UserId,
};
use tracing::log::trace;

use crate::anilist_struct::run::user::UserWrapper;
use crate::constant::{
    COLOR, COMMAND_SENDING_ERROR, DIFFERED_COMMAND_SENDING_ERROR, OPTION_ERROR, PASS_LIMIT,
};
use crate::error_enum::AppError;
use crate::lang_struct::anilist::list_register_user::load_localization_list_user;
use crate::sqls::general::data::get_registered_user;

pub async fn run(ctx: &Context, command: &CommandInteraction) -> Result<(), AppError> {
    let guild_id = match command.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let list_user_localised = load_localization_list_user(guild_id).await?;

    let guild_id = command.guild_id.ok_or(OPTION_ERROR.clone())?;

    let guild = guild_id
        .to_partial_guild_with_counts(&ctx.http)
        .await
        .map_err(|_| OPTION_ERROR.clone())?;

    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command
        .create_response(&ctx.http, builder_message)
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())?;

    let mut anilist_user = Vec::new();
    let mut last_id: Option<UserId> = None;
    let mut pass = 0;
    while anilist_user.len() < 10 && pass < PASS_LIMIT {
        let members = guild
            .members(&ctx.http, Some(25u64), last_id)
            .await
            .map_err(|_| OPTION_ERROR.clone())?;

        for member in members {
            last_id = Some(member.user.id);
            let user_id = member.user.id.to_string();
            let row: (Option<String>, Option<String>) = get_registered_user(&user_id).await?;
            let user_date = match row.0 {
                Some(a) => {
                    trace!("{}", a);
                    match a.parse::<i32>() {
                        Ok(b) => UserWrapper::new_user_by_id(b).await?,
                        Err(_) => UserWrapper::new_user_by_search(&a).await?,
                    }
                }
                None => continue,
            };
            let data = Data {
                user: member.user,
                anilist: user_date,
            };
            anilist_user.push(data)
        }
        pass += 1;
    }

    let user_links: Vec<String> = anilist_user
        .iter()
        .map(|data| {
            format!(
                "[{}](<https://anilist.co/user/{}>)",
                data.user.name,
                data.anilist.data.user.name.clone().unwrap_or_default()
            )
        })
        .collect();
    let joined_string = user_links.join("\n\n");

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .title(list_user_localised.title)
        .description(joined_string);

    let builder_message = CreateInteractionResponseFollowup::new().embed(builder_embed);

    let _ = command
        .create_followup(&ctx.http, builder_message)
        .await
        .map_err(|_| DIFFERED_COMMAND_SENDING_ERROR.clone())?;
    Ok(())
}

struct Data {
    pub user: User,
    pub anilist: UserWrapper,
}
