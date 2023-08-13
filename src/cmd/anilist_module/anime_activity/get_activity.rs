use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use crate::cmd::general_module::differed_response::differed_response;
use crate::cmd::general_module::pool::get_pool;

pub async fn run(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> String {
    let result_diff = differed_response(ctx, command).await;

    if result_diff != "good".as_ref() {
        return result_diff;
    }
    let database_url = "./data.db";
    let pool = get_pool(database_url).await;
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("get_activity")
        .description("List all anime activity")
}