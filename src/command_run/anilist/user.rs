use serenity::all::{CommandDataOption, CommandInteraction, Context};
use tracing::trace;

use crate::anilist_struct::run::user::{send_embed, UserWrapper};
use crate::constant::OPTION_ERROR;
use crate::error_enum::AppError;
use crate::sqls::general::data::get_registered_user;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<(), AppError> {
    trace!("{:?}", options);
    for option in options {
        if option.name.as_str() != "type" {
             if let Some(a) =  option.value.as_str() {
                    let value = &a.to_string();

                    let data: UserWrapper = if value.parse::<i32>().is_ok() {
                        UserWrapper::new_user_by_id(value.parse().unwrap()).await?
                    } else {
                        UserWrapper::new_user_by_search(value).await?
                    };

                    return send_embed(ctx, command, data).await;
            }
        }
    }
    let user_id = &command.user.id.to_string();
    let row: (Option<String>, Option<String>) = get_registered_user(user_id).await?;
    trace!("{:?}", row);
    let (user, _): (Option<String>, Option<String>) = row;
    let user = user.ok_or(OPTION_ERROR.clone())?;
    let data = UserWrapper::new_user_by_id((user).parse::<i32>().unwrap()).await?;
    return send_embed(ctx, command, data).await;
}
