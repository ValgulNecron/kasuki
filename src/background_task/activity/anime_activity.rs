use std::error::Error;
use std::io::{Cursor, Read};
use std::sync::Arc;
use std::time::Duration;

use base64::engine::general_purpose::STANDARD;
use base64::read::DecoderReader;
use chrono::Utc;
use moka::future::Cache;
use serenity::all::{Context, CreateAttachment, EditWebhook, ExecuteWebhook, Webhook};
use tokio::sync::RwLock;
use tracing::{error, trace};

use crate::command::run::admin::anilist::add_activity::get_minimal_anime_by_id;
use crate::database::data_struct::server_activity::ServerActivityFull;
use crate::database::manage::dispatcher::data_dispatch::{
    get_data_activity, remove_data_activity_status, set_data_activity,
};
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum;
use crate::structure::message::anilist_user::send_activity::load_localization_send_activity;

/// `manage_activity` is an asynchronous function that manages activities.
/// It takes a `ctx` as a parameter.
/// `ctx` is a Context that represents the context.
///
/// This function calls the `send_activity` function with the context.
///
/// # Arguments
///
/// * `ctx` - A Context that represents the context.
pub async fn manage_activity(
    ctx: Context,
    db_type: String,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) {
    send_activity(&ctx, db_type, anilist_cache).await;
}

/// `send_activity` is an asynchronous function that sends activities.
/// It takes a `ctx` as a parameter.
/// `ctx` is a reference to the Context.
///
/// This function first gets the current timestamp and retrieves the activity data based on the timestamp.
/// It then iterates over the retrieved activity data.
/// If the timestamp of the activity data is not set or does not match the current timestamp, it skips the activity data.
/// Otherwise, it clones the activity data and the guild ID from the activity data and clones the context.
/// If the delays of the activity data is not set, it spawns a new task to send the specific activity.
/// If the delays of the activity data is not zero, it spawns a new task to sleep for the delay duration and then send the specific activity.
/// If the delays of the activity data is zero, it spawns a new task to send the specific activity.
///
/// # Arguments
///
/// * `ctx` - A reference to the Context.
async fn send_activity(
    ctx: &Context,
    db_type: String,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) {
    let now = Utc::now().timestamp().to_string();
    let rows = match get_data_activity(now.clone(), db_type.clone()).await {
        Ok(rows) => rows,
        Err(e) => {
            error!("{}", e);
            return;
        }
    };
    for row in rows {
        if now != row.timestamp.to_string() {
            continue;
        }

        let row2 = row.clone();
        let guild_id = row.guild_id.clone();
        let ctx = ctx.clone();
        if row.delays != 0 {
            let db_type = db_type.clone();
            let anilist_cache = anilist_cache.clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(row2.delays as u64)).await;
                if let Err(e) =
                    send_specific_activity(row, guild_id, row2, &ctx, db_type, anilist_cache).await
                {
                    error!("{}", e)
                }
            });
        } else {
            let db_type = db_type.clone();
            let anilist_cache = anilist_cache.clone();
            tokio::spawn(async move {
                if let Err(e) =
                    send_specific_activity(row, guild_id, row2, &ctx, db_type, anilist_cache).await
                {
                    error!("{}", e);
                }
            });
        }
    }
}

/// `send_specific_activity` is an asynchronous function that sends a specific activity.
/// It takes `row`, `guild_id`, `row2`, and `ctx` as parameters.
/// `row` is an ActivityData that represents the activity data.
/// `guild_id` is a String that represents the ID of the guild.
/// `row2` is another ActivityData that represents the activity data.
/// `ctx` is a reference to the Context.
/// It returns a Result which is either an empty tuple or an AppError.
///
/// This function first loads the localized send activity text based on the guild ID.
/// It then retrieves the webhook URL from the `row` and creates a webhook from the URL.
/// It decodes the image from the `row` and creates an attachment from the decoded bytes.
/// The webhook is then edited to have the name from the `row` and the created attachment as the avatar.
/// An embed is created with the color, description, URL, and title set.
/// The embed is then sent using the webhook.
/// Finally, it spawns a new task to update the information of the activity.
///
/// # Arguments
///
/// * `row` - An ActivityData that represents the activity data.
/// * `guild_id` - A String that represents the ID of the guild.
/// * `row2` - Another ActivityData that represents the activity data.
/// * `ctx` - A reference to the Context.
///
/// # Returns
///
/// * `Result<(), AppError>` - A Result type which is either an empty tuple or an AppError.
async fn send_specific_activity(
    row: ServerActivityFull,
    guild_id: String,
    row2: ServerActivityFull,
    ctx: &Context,
    db_type: String,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<(), Box<dyn Error>> {
    let localised_text = load_localization_send_activity(guild_id.clone(), db_type.clone()).await?;
    let webhook_url = row.webhook.clone();
    let mut webhook = Webhook::from_url(&ctx.http, webhook_url.as_str())
        .await
        .map_err(|e| error_enum::Error::Webhook(format!("{:#?}", e)))?;

    let image = row.image;
    trace!(image);

    let cursor = Cursor::new(image);
    let mut decoder = DecoderReader::new(cursor, &STANDARD);

    // Read the decoded bytes into a Vec
    let mut decoded_bytes = Vec::new();
    decoder
        .read_to_end(&mut decoded_bytes)
        .map_err(|e| error_enum::Error::Byte(format!("{:#?}", e)))?;
    let name = row.name.clone();
    let trimmed_name = if name.len() > 100 {
        name[..100].to_string()
    } else {
        name
    };
    let attachment = CreateAttachment::bytes(decoded_bytes, "avatar");
    let edit_webhook = EditWebhook::new().name(trimmed_name).avatar(&attachment);
    webhook
        .edit(&ctx.http, edit_webhook)
        .await
        .map_err(|e| error_enum::Error::Webhook(format!("{:#?}", e)))?;

    let embed = get_default_embed(None)
        .description(
            localised_text
                .desc
                .replace("$ep$", &row.episode.to_string())
                .as_str()
                .replace("$anime$", row.name.as_str()),
        )
        .url(format!("https://anilist.co/anime/{}", row.anime_id))
        .title(&localised_text.title);

    let builder_message = ExecuteWebhook::new().embed(embed);

    webhook
        .execute(&ctx.http, false, builder_message)
        .await
        .map_err(|e| error_enum::Error::Webhook(format!("{:#?}", e)))?;

    tokio::spawn(async move {
        if let Err(e) = update_info(row2, guild_id, anilist_cache.clone(), db_type).await { error!("{}", e) }
    });
    Ok(())
}

/// `update_info` is an asynchronous function that updates the information of an activity.
/// It takes a `row` and `guild_id` as parameters.
/// `row` is an ActivityData that represents the activity data.
/// `guild_id` is a String that represents the ID of the guild.
/// It returns a Result which is either an empty tuple or an AppError.
///
/// This function first retrieves the minimal anime data by the anime ID from the `row`.
/// It then checks if there is a next airing episode for the anime.
/// If there is no next airing episode, it removes the activity and returns.
/// If there is a next airing episode, it retrieves the title of the anime and sets the name of the activity to the English title if it exists, otherwise it sets it to the Romaji title.
/// It then sets the activity data with the updated information.
///
/// # Arguments
///
/// * `row` - An ActivityData that represents the activity data.
/// * `guild_id` - A String that represents the ID of the guild.
///
/// # Returns
///
/// * `Result<(), AppError>` - A Result type which is either an empty tuple or an AppError.
async fn update_info(
    row: ServerActivityFull,
    guild_id: String,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
    db_type: String,
) -> Result<(), Box<dyn Error>> {
    let media = get_minimal_anime_by_id(row.anime_id, anilist_cache).await?;
    let next_airing = match media.next_airing_episode {
        Some(na) => na,
        None => return remove_activity(row, guild_id, db_type).await,
    };
    let title = media
        .title
        .ok_or(error_enum::Error::Option(String::from("no title")))?;
    let rj = title.romaji;
    let en = title.english;
    let name = en.unwrap_or(rj.unwrap_or(String::from("nothing")));
    set_data_activity(
        ServerActivityFull {
            anime_id: media.id,
            timestamp: next_airing.airing_at as i64,
            guild_id,
            webhook: row.webhook,
            episode: next_airing.episode,
            name,
            delays: row.delays,
            image: row.image,
        },
        db_type,
    )
    .await?;
    Ok(())
}

/// `remove_activity` is an asynchronous function that removes an activity.
/// It takes a `row` and `guild_id` as parameters.
/// `row` is an ActivityData that represents the activity data.
/// `guild_id` is a String that represents the ID of the guild.
/// It returns a Result which is either an empty tuple or an AppError.
///
/// # Arguments
///
/// * `row` - An ActivityData that represents the activity data.
/// * `guild_id` - A String that represents the ID of the guild.
///
/// # Returns
///
/// * `Result<(), AppError>` - A Result type which is either an empty tuple or an AppError.
async fn remove_activity(
    row: ServerActivityFull,
    guild_id: String,
    db_type: String,
) -> Result<(), Box<dyn Error>> {
    trace!("removing {:#?} for {}", row, guild_id);
    remove_data_activity_status(guild_id, row.anime_id.to_string(), db_type).await?;
    Ok(())
}
