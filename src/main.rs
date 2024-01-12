use serenity::all::{
    ActivityData, Context, EventHandler, GatewayIntents, Interaction, Member, Ready, UserId,
};
use serenity::{async_trait, Client};
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, info, trace};

use struct_shard_manager::ShardManagerContainer;

use crate::activity::anime_activity::manage_activity;
use crate::command_autocomplete::autocomplete_dispatch::autocomplete_dispatching;
use crate::command_register::command_registration::creates_commands;
use crate::command_run::command_dispatch::command_dispatching;
use crate::common::calculate_user_color::calculate_users_color;
use crate::components::components_dispatch::components_dispatching;
use crate::constant::{ACTIVITY_NAME, USER_COLOR_UPDATE_TIME};
use crate::game_struct::steam_game_id_struct::get_game;
use crate::logger::{create_log_directory, init_logger, remove_old_logs};
use crate::sqls::general::sql::init_sql_database;

mod activity;
mod anilist_struct;
mod command_autocomplete;
mod command_register;
mod command_run;
mod common;
mod components;
mod constant;
mod error_enum;
mod error_management;
mod game_struct;
mod lang_struct;
mod logger;
mod sqls;
pub mod struct_shard_manager;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        let guilds = ctx.cache.guilds();
        let ctx_clone = ctx.clone();
        tokio::spawn(async move {
            loop {
                let mut members: Vec<Member> = Vec::new();
                for guild in &guilds {
                    let mut i = 0;
                    while members.len() == (1000 * i) {
                        let mut members_temp = if i == 0 {
                            guild
                                .members(&ctx_clone.http, Some(1000), None)
                                .await
                                .unwrap()
                        } else {
                            let user: UserId = members.last().unwrap().user.id;
                            guild
                                .members(&ctx_clone.http, Some(1000), Some(user))
                                .await
                                .unwrap()
                        };
                        members.append(&mut members_temp);
                        i += 1
                    }
                }
                match calculate_users_color(members.into_iter().collect()).await {
                    Ok(_) => {}
                    Err(e) => error!("{:?}", e),
                };
                sleep(Duration::from_secs((USER_COLOR_UPDATE_TIME * 60) as u64)).await;
            }
        });
        // Add activity to the bot as the type in activity_type and with ACTIVITY_NAME as name
        let activity_type = Some(ActivityData::custom(ACTIVITY_NAME));
        ctx.set_activity(activity_type);

        info!(
            "Shard {:?} of {} is connected!",
            ready.shard, ready.user.name
        );

        info!("{:?}", &ctx.cache.guilds().len());

        for guild in ctx.cache.guilds() {
            let partial_guild = guild.to_partial_guild(&ctx.http).await.unwrap();
            debug!(
                "guild id: {:#?} | guild name {:#?}",
                &partial_guild.id, &partial_guild.name
            )
        }

        let my_path = ".env";
        let path = std::path::Path::new(my_path);
        let _ = dotenv::from_path(path);

        trace!("{:#?}", env::var("REMOVE_OLD_COMMAND"));

        let is_ok = env::var("REMOVE_OLD_COMMAND")
            .unwrap_or("false".to_string())
            .to_lowercase()
            .as_str()
            == "true";

        trace!("{}", is_ok);

        creates_commands(&ctx.http, is_ok).await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command_interaction) = interaction.clone() {
            debug!(
                "Received command interaction: {}, Option: {:#?}, User: {}({})",
                command_interaction.data.name,
                command_interaction.data.options,
                command_interaction.user.name,
                command_interaction.user.id
            );
            trace!("{:#?}", command_interaction);
            if let Err(e) = command_dispatching(ctx, command_interaction).await {
                error_management::error_dispatch::command_dispatching(e).await
            }
        } else if let Interaction::Autocomplete(autocomplete_interaction) = interaction.clone() {
            autocomplete_dispatching(ctx, autocomplete_interaction).await
        } else if let Interaction::Component(component_interaction) = interaction.clone() {
            if let Err(e) = components_dispatching(ctx, component_interaction).await {
                trace!("{:#?}", e)
            }
        }
    }
}

#[tokio::main]
async fn main() {
    println!("Bot starting please wait.");
    // Configure the client with your Discord bot token in the environment.
    let my_path = ".env";
    let path = std::path::Path::new(my_path);
    let _ = dotenv::from_path(path);

    let env = env::var("RUST_LOG")
        .unwrap_or("info".to_string())
        .to_lowercase();
    let log = env.as_str();
    match create_log_directory() {
        Ok(_) => {}
        Err(_) => return,
    };

    let _ = remove_old_logs().is_ok();

    match init_logger(log) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{:?}", e);
            return;
        }
    };

    match init_sql_database().await {
        Ok(_) => {}
        Err(e) => {
            error!("{:?}", e);
            return;
        }
    }

    tokio::spawn(async move {
        info!("Launching the activity management thread!");
        manage_activity().await
    });

    tokio::spawn(async move {
        info!("Launching the game management thread!");
        get_game().await
    });

    info!("starting the bot.");
    let token = match env::var("DISCORD_TOKEN") {
        Ok(token) => {
            debug!("Successfully got the token from env.");
            token
        }
        Err(_) => {
            error!("Env variable not set exiting.");
            return;
        }
    };

    // Build our client.
    let gateway_intent = GatewayIntents::GUILD_MEMBERS;
    let mut client = match Client::builder(token, gateway_intent)
        .event_handler(Handler)
        .await
    {
        Ok(client) => {
            debug!("Client created.");
            client
        }
        Err(e) => {
            error!("Error while creating client: {}", e);
            return;
        }
    };

    {
        let mut data = client.data.write().await;

        let shard_manager = &client.shard_manager;

        data.insert::<ShardManagerContainer>(Arc::clone(shard_manager))
    }

    if let Err(why) = client.start_autosharded().await {
        error!("Client error: {:?}", why)
    }
}
