use crate::function::general::differed_response::differed_response;
use crate::function::sql::sqlite::pool::get_sqlite_pool;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> String {
    differed_response(ctx, command).await;

    let database_url = "./data.db";
    let pool = get_sqlite_pool(database_url).await;

    "good".to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("get_activity")
        .description("List all anime activity")
}
