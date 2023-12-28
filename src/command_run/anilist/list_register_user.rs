use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context, CreateButton, CreateEmbed, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage, PartialGuild, Timestamp, User, UserId,
};
use tracing::log::trace;

use crate::anilist_struct::run::user::UserWrapper;
use crate::constant::{
    COLOR, COMMAND_SENDING_ERROR, DIFFERED_COMMAND_SENDING_ERROR, MEMBER_LIMIT, OPTION_ERROR,
    PASS_LIMIT,
};
use crate::error_enum::AppError;
use crate::lang_struct::anilist::list_register_user::{
    load_localization_list_user, ListUserLocalised,
};
use crate::sqls::general::data::get_registered_user;

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let list_user_localised = load_localization_list_user(guild_id).await?;

    let guild_id = command_interaction.guild_id.ok_or(OPTION_ERROR.clone())?;

    let guild = guild_id
        .to_partial_guild_with_counts(&ctx.http)
        .await
        .map_err(|_| OPTION_ERROR.clone())?;

    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command_interaction
        .create_response(&ctx.http, builder_message)
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())?;

    let (builder_message, len, last_id): (CreateEmbed, usize, Option<UserId>) =
        get_the_list(guild, ctx, &list_user_localised, None).await?;
    trace!("{:#?}", builder_message);
    trace!("{:#?}", len);
    trace!("{:#?}", MEMBER_LIMIT);
    trace!("{:#?}", last_id);
    let mut response = CreateInteractionResponseFollowup::new().embed(builder_message);
    if len == (MEMBER_LIMIT - 1) as usize {
        response = response.button(
            CreateButton::new(format!("next_{}", last_id.unwrap())).label(list_user_localised.next),
        )
    }

    let _ = command_interaction
        .create_followup(&ctx.http, response)
        .await
        .map_err(|_| DIFFERED_COMMAND_SENDING_ERROR.clone())?;
    Ok(())
}

struct Data {
    pub user: User,
    pub anilist: UserWrapper,
}

pub async fn get_the_list(
    guild: PartialGuild,
    ctx: &Context,
    list_user_localised: &ListUserLocalised,
    last_id: Option<UserId>,
) -> Result<(CreateEmbed, usize, Option<UserId>), AppError> {
    let mut anilist_user = Vec::new();
    let mut last_id: Option<UserId> = last_id;
    let mut pass = 0;
    while anilist_user.len() < MEMBER_LIMIT as usize && pass < PASS_LIMIT {
        let members = guild
            .members(&ctx.http, Some(MEMBER_LIMIT), last_id)
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
                data.anilist.data.user.id.unwrap_or(0)
            )
        })
        .collect();
    let joined_string = user_links.join("\n\n");

    Ok((
        CreateEmbed::new()
            .timestamp(Timestamp::now())
            .color(COLOR)
            .title(&list_user_localised.title)
            .description(joined_string),
        anilist_user.len(),
        last_id,
    ))
}
