use chrono::Utc;
use serenity::all::{CommandDataOption, CommandDataOptionValue, CommandInteraction, Context};
use crate::constant::OPTION_ERROR;
use crate::error_enum::AppError;
use crate::sqls::general::cache::{get_database_cache, get_database_random_cache};

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<(), AppError> {
    let option = &options.get(0).ok_or(OPTION_ERROR.clone())?.value;
    if let CommandDataOptionValue::String(random_type) = option {
        let row: (Option<String>, Option<i64>, Option<i64>) = get_database_random_cache(random_type);
        let (response, last_updated, last_page): (Option<String>, Option<i64>, Option<i64>) = row;
        let page_number = last_page.unwrap_or(1567); // This is as today date the last page, i will update it sometime.
        let previous_page = page_number - 1;
        let cached_response = response.unwrap_or("Nothing".to_string());
        if let Some(updated) = last_updated {
            let duration_since_updated = Utc::now().timestamp() - updated;
            if duration_since_updated < 24 * 60 * 60 {
                embed(page_number, random_type.to_string(), ctx, command).await;
            } else {
                update_cache(
                    page_number,
                    random_type,
                    ctx,
                    command,
                    previous_page,
                    cached_response,
                )
                .await
            }
        } else {
            update_cache(
                page_number,
                random_type,
                ctx,
                command,
                previous_page,
                cached_response,
            )
            .await
        }
        Ok(())
    } else {
        return Err(AppError::NoCommandOption(String::from(
            "The command contain no option.",
        )));
    }
}

pub async fn embed(
    last_page: i64,
    random_type: String,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {

}

pub async fn update_cache(
    mut page_number: i64,
    random_type: &String,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    mut previous_page: i64,
    mut cached_response: String,
) {

}