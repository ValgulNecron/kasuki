extern crate core;

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use std::time::Duration;

use log::{debug, error, info, trace};
use serenity::async_trait;
use serenity::client::Context;
use serenity::model::application::command::Command;
use serenity::model::application::interaction::Interaction;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::gateway::Activity;
use serenity::model::gateway::Ready;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::Timestamp;
use serenity::prelude::*;
use serenity::utils::Colour;
use structure::struct_shard_manager::ShardManagerContainer;
use tokio::time::sleep;

use crate::cmd::ai_module::{image, transcript, translation};
use crate::cmd::anilist_module::send_activity::manage_activity;
use crate::cmd::anilist_module::{
    add_activity, anime, character, compare, level, ln, manga, random, register, search, seiyuu,
    staff, studio, user, waifu,
};
use crate::cmd::general_module::module_activation::check_activation_status;
use crate::cmd::general_module::{
    avatar, banner, credit, info, lang, module_activation, ping, profile,
};
use crate::constant::{ACTIVITY_NAME, COLOR, PING_UPDATE_DELAYS};
use crate::function::error_management::no_lang_error::no_langage_error;
use crate::function::general::get_guild_langage::get_guild_langage;
use crate::function::sqls::general::data::set_data_ping_history;
use crate::function::sqls::general::sql::init_sql_database;

use crate::logger::{create_log_directory, init_logger, remove_old_logs};
use crate::structure::anilist::media::struct_autocomplete_media;
use crate::structure::anilist::user::struct_autocomplete_user;
use crate::structure::embed::error::ErrorLocalisedText;

mod available_lang;
mod cmd;
mod constant;
mod function;
mod logger;
mod structure;
mod tests;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        // Add activity to the bot as the type in activity_type and with ACTIVITY_NAME as name
        let activity_type: Activity = Activity::playing(ACTIVITY_NAME);
        ctx.set_activity(activity_type).await;

        info!(
            "Shard {:?} of {} is connected!",
            ready.shard, ready.user.name
        );

        // Create all the commande found in cmd. (if a command is added it will need to be added here).
        let guild_command = Command::set_global_application_commands(&ctx.http, |commands| {
            commands
                // General module.
                .create_application_command(|command| ping::register(command))
                .create_application_command(|command| lang::register(command))
                .create_application_command(|command| info::register(command))
                .create_application_command(|command| banner::register(command))
                .create_application_command(|command| profile::register(command))
                .create_application_command(|command| module_activation::register(command))
                .create_application_command(|command| credit::register(command))
                .create_application_command(|command| avatar::register(command))
                // Anilist module.
                .create_application_command(|command| anime::register(command))
                .create_application_command(|command| character::register(command))
                .create_application_command(|command| compare::register(command))
                .create_application_command(|command| level::register(command))
                .create_application_command(|command| ln::register(command))
                .create_application_command(|command| manga::register(command))
                .create_application_command(|command| random::register(command))
                .create_application_command(|command| register::register(command))
                .create_application_command(|command| search::register(command))
                .create_application_command(|command| staff::register(command))
                .create_application_command(|command| user::register(command))
                .create_application_command(|command| waifu::register(command))
                .create_application_command(|command| studio::register(command))
                .create_application_command(|command| add_activity::register(command))
                .create_application_command(|command| seiyuu::register(command))
                // AI module.
                .create_application_command(|command| image::register(command))
                .create_application_command(|command| transcript::register(command))
                .create_application_command(|command| translation::register(command))
        })
        .await;
        match guild_command {
            Ok(commands) => {
                for command in commands {
                    trace!("Command {}, Version {}", command.name, command.version);
                }
            }
            Err(e) => {
                error!("Error while creating command: {}", e)
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            info!(
                "Received command interaction: {}, Option: {:#?}, User: {}({})",
                command.data.name, command.data.options, command.user.name, command.user.id
            );
            match command.data.name.as_str() {
                // General module.
                "ping" => ping::run(&ctx, &command).await,
                "lang" => lang::run(&command.data.options, &ctx, &command).await,
                "info" => info::run(&ctx, &command).await,
                "banner" => banner::run(&command.data.options, &ctx, &command).await,
                "profile" => profile::run(&command.data.options, &ctx, &command).await,
                "module" => module_activation::run(&command.data.options, &ctx, &command).await,
                "credit" => credit::run(&ctx, &command).await,
                "avatar" => avatar::run(&command.data.options, &ctx, &command).await,

                // Anilist module
                "anime" => {
                    if !check_if_anime_is_on(&command, COLOR, &ctx).await {
                        return;
                    }
                    anime::run(&command.data.options, &ctx, &command).await
                }
                "character" => {
                    if !check_if_anime_is_on(&command, COLOR, &ctx).await {
                        return;
                    }
                    character::run(&command.data.options, &ctx, &command).await
                }
                "compare" => {
                    if !check_if_anime_is_on(&command, COLOR, &ctx).await {
                        return;
                    }
                    compare::run(&command.data.options, &ctx, &command).await
                }
                "level" => {
                    if !check_if_anime_is_on(&command, COLOR, &ctx).await {
                        return;
                    }
                    level::run(&command.data.options, &ctx, &command).await
                }
                "ln" => {
                    if !check_if_anime_is_on(&command, COLOR, &ctx).await {
                        return;
                    }
                    ln::run(&command.data.options, &ctx, &command).await
                }
                "manga" => {
                    if !check_if_anime_is_on(&command, COLOR, &ctx).await {
                        return;
                    }
                    manga::run(&command.data.options, &ctx, &command).await
                }
                "random" => {
                    if !check_if_anime_is_on(&command, COLOR, &ctx).await {
                        return;
                    }
                    random::run(&command.data.options, &ctx, &command).await
                }
                "register" => {
                    if !check_if_anime_is_on(&command, COLOR, &ctx).await {
                        return;
                    }
                    register::run(&command.data.options, &ctx, &command).await
                }
                "search" => {
                    if !check_if_anime_is_on(&command, COLOR, &ctx).await {
                        return;
                    }
                    search::run(&command.data.options, &ctx, &command).await
                }
                "staff" => {
                    if !check_if_anime_is_on(&command, COLOR, &ctx).await {
                        return;
                    }
                    staff::run(&command.data.options, &ctx, &command).await
                }
                "user" => {
                    if !check_if_anime_is_on(&command, COLOR, &ctx).await {
                        return;
                    }
                    user::run(&command.data.options, &ctx, &command).await
                }
                "waifu" => {
                    if !check_if_anime_is_on(&command, COLOR, &ctx).await {
                        return;
                    }
                    waifu::run(&ctx, &command).await
                }
                "studio" => {
                    if !check_if_anime_is_on(&command, COLOR, &ctx).await {
                        return;
                    }
                    studio::run(&command.data.options, &ctx, &command).await
                }
                "add_activity" => {
                    if !check_if_anime_is_on(&command, COLOR, &ctx).await {
                        return;
                    }
                    add_activity::run(&command.data.options, &ctx, &command).await
                }
                "seiyuu" => {
                    if !check_if_anime_is_on(&command, COLOR, &ctx).await {
                        return;
                    }
                    seiyuu::run(&command.data.options, &ctx, &command).await
                }

                // AI module
                "image" => {
                    if !check_if_ai_is_on(&command, COLOR, &ctx).await {
                        return;
                    }
                    image::run(&command.data.options, &ctx, &command).await
                }
                "transcript" => {
                    if !check_if_ai_is_on(&command, COLOR, &ctx).await {
                        return;
                    }
                    transcript::run(&command.data.options, &ctx, &command).await
                }
                "translation" => {
                    if !check_if_ai_is_on(&command, COLOR, &ctx).await {
                        return;
                    }
                    translation::run(&command.data.options, &ctx, &command).await
                }

                _ => return,
            };

            // check if the command was successfully done.
        } else if let Interaction::Autocomplete(command) = interaction {
            match command.data.name.as_str() {
                "anime" => struct_autocomplete_media::autocomplete(ctx, command).await,
                "manga" => manga::autocomplete(ctx, command).await,
                "ln" => ln::autocomplete(ctx, command).await,
                "character" => character::autocomplete(ctx, command).await,
                "staff" => staff::autocomplete(ctx, command).await,
                "seiyuu" => staff::autocomplete(ctx, command).await,
                "user" => struct_autocomplete_user::autocomplete(ctx, command).await,
                "compare" => compare::autocomplete(ctx, command).await,
                "level" => struct_autocomplete_user::autocomplete(ctx, command).await,
                "studio" => studio::autocomplete(ctx, command).await,
                "add_activity" => struct_autocomplete_media::autocomplete(ctx, command).await,
                _ => print!(""),
            }
        }
    }
}

#[tokio::main]
async fn main() {
    init_sql_database().await;
    // Configure the client with your Discord bot token in the environment.
    let my_path = "./.env";
    let path = std::path::Path::new(my_path);
    let _ = dotenv::from_path(path);
    let env = env::var("LOG").unwrap_or("info".to_string()).to_lowercase();
    let log = env.as_str();

    match create_log_directory() {
        Ok(_) => {}
        Err(_) => return,
    };

    let _ = remove_old_logs().is_ok();

    match init_logger(log) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    let nsfw_env = env::var("NSFW");
    println!("{:?}", nsfw_env);

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
    let mut client = match Client::builder(token, GatewayIntents::empty())
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

    tokio::spawn(async move {
        // create_server().await.expect("Web server running");
    });

    let manager = client.shard_manager.clone();

    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(PING_UPDATE_DELAYS)).await;

            let lock = manager.lock().await;
            let shard_runners = lock.runners.lock().await;

            for (id, runner) in shard_runners.iter() {
                let shard_id = id.0.to_string();
                let latency_content = runner.latency.unwrap_or(Duration::from_secs(0));
                let latency = format!("{:?}", latency_content);
                set_data_ping_history(shard_id, latency.clone()).await;
                debug!("{}", latency)
            }
        }
    });

    tokio::spawn(async move { manage_activity().await });

    {
        let mut data = client.data.write().await;

        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }

    if let Err(why) = client.start_autosharded().await {
        error!("Client error: {:?}", why);
    }
}

async fn check_if_anime_is_on(
    command: &ApplicationCommandInteraction,
    color: Colour,
    ctx: &Context,
) -> bool {
    let guild_id = match command.guild_id {
        Some(id) => id,
        None => {
            error!("No guild id");
            return true;
        }
    }
    .0
    .to_string()
    .clone();
    if !check_activation_status("ANILIST", guild_id.clone()).await {
        send_deactivated_message(command, color, ctx, guild_id).await;
        false
    } else {
        true
    }
}

async fn check_if_ai_is_on(
    command: &ApplicationCommandInteraction,
    color: Colour,
    ctx: &Context,
) -> bool {
    let guild_id = match command.guild_id {
        Some(id) => id,
        None => {
            error!("No guild id");
            return true;
        }
    }
    .0
    .to_string()
    .clone();
    if !check_activation_status("AI", guild_id.clone()).await {
        send_deactivated_message(command, color, ctx, guild_id).await;
        false
    } else {
        true
    }
}

async fn send_deactivated_message(
    command: &ApplicationCommandInteraction,
    color: Colour,
    ctx: &Context,
    guild_id: String,
) {
    let path = "./lang_file/embed/error.json";
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(e) => {
            error!("Error while opening lang file at {}:  {}", path, e);
            return;
        }
    };
    let mut json = String::new();
    match file.read_to_string(&mut json) {
        Ok(_) => {}
        Err(e) => {
            error!("Failed to read the file {}: {}", path, e)
        }
    };
    let json_data: HashMap<String, ErrorLocalisedText> = match serde_json::from_str(&json) {
        Ok(json) => json,
        Err(e) => {
            error!("Error when paring the json file: {}", e);
            return;
        }
    };

    let lang_choice = get_guild_langage(guild_id.clone()).await;
    if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.embed(|m| {
                            m.title(&localised_text.error_title)
                                .description(&localised_text.module_off)
                                .timestamp(Timestamp::now())
                                .color(color)
                        })
                    })
            })
            .await
        {
            error!("Cannot respond to slash command: {}", why);
        }
    } else {
        no_langage_error(ctx, command).await
    }
}
