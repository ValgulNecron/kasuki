use crate::constant::OPTION_ERROR;
use crate::error_enum::AppError;
use crate::lang_struct::anilist::list_register_user::load_localization_list_user;
use crate::lang_struct::general::guild::load_localization_guild;
use crate::sqls::general::data::get_registered_user;
use serenity::all::{CommandInteraction, Context};

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

    let members = guild
        .members(&ctx.http, Some(1000u64), None)
        .await
        .map_err(|_| OPTION_ERROR.clone())?;

    let mut anilist_user = Vec::new();
    for member in members {
        let user_id = member.user.id.to_string();
        let row: (Option<String>, Option<String>) = get_registered_user(&user_id).await?;
        let data = Data {
            user_id: row.0.unwrap(),
            anilist_id: row.1.unwrap(),
        };
        anilist_user.push(data)
    }

    Ok(())
}

struct Data {
    pub user_id: String,
    pub anilist_id: String,
}
