mod config_structure;
mod event_handler;
mod command;

use anyhow::{Context, Result};
use serenity::gateway::ActivityData;

#[tokio::main]

async fn main() -> Result<()> {
    // launch migration


    // get config


    // create logger


    // create bot client
    let handler = ;


    let intent = serenity::prelude::GatewayIntents::non_privileged();
    let privileged = serenity::prelude::GatewayIntents::GUILD_PRESENCES
        | serenity::prelude::GatewayIntents::GUILD_MEMBERS
        | serenity::prelude::GatewayIntents::MESSAGE_CONTENT;

    let all = intent | privileged;


    let client = serenity::Client::builder(token, all)
        .event_handler(handler)
        .activity(ActivityData::custom(

        ))
        .await
        .context("Failed to create client")?;

    let shard_manager = client.shard_manager.clone();

    Ok(())
}
