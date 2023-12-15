use crate::anilist_struct::run::user::{send_embed, UserWrapper};
use crate::constant::OPTION_ERROR;
use crate::error_enum::AppError;
use crate::error_enum::AppError::NoCommandOption;
use crate::sqls::general::data::get_registered_user;
use serenity::all::{CommandDataOption, CommandDataOptionValue, CommandInteraction, Context};
use tracing::trace;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<(), AppError> {
    if let Some(_) = options.get(0) {
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

        send_embed(ctx, command, data).await
    } else {
        let user_id = &command.user.id.to_string();
        let row: (Option<String>, Option<String>) = get_registered_user(user_id).await?;
        trace!("{:?}", row);
        let (user, _): (Option<String>, Option<String>) = row;
        let user = user.ok_or(OPTION_ERROR.clone())?;
        let data = UserWrapper::new_user_by_id((&user).parse::<i32>().unwrap()).await?;
        send_embed(ctx, command, data).await
    }
}
