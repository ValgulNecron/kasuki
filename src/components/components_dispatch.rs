use serenity::all::{ComponentInteraction, Context};
use std::error::Error;
use tracing::trace;

use crate::components::anilist::{list_all_activity, list_register_user};
use moka::future::Cache;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Dispatches component interactions based on their custom ID.
///
/// This function takes a context and a component interaction as parameters.
/// It retrieves the custom ID from the component interaction and checks if it starts with certain prefixes.
/// If the custom ID starts with "user_", it splits the custom ID to get the user ID and the previous ID,
/// and then calls the `list_register_user::update` function with these IDs.
/// If the custom ID starts with "next_activity_", it splits the custom ID to get the page number,
/// and then calls the `list_all_activity::update` function with this page number.
/// If the custom ID does not start with any of the known prefixes, it logs a trace message.
///
/// # Arguments
///
/// * `ctx` - A Context instance representing the current bot context.
/// * `component_interaction` - A ComponentInteraction instance representing the interaction that triggered this function.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError.
pub async fn components_dispatching(
    ctx: Context,
    component_interaction: ComponentInteraction,
    db_type: String,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<(), Box<dyn Error>> {
    match component_interaction.data.custom_id.as_str() {
        s if s.starts_with("user_") => {
            let user_id = s.split_at("_".len()).1;
            let prev_id = user_id.split_at("_".len()).1;
            list_register_user::update(
                &ctx,
                &component_interaction,
                user_id,
                prev_id,
                db_type,
                anilist_cache,
            )
            .await?
        }
        s if s.starts_with("next_activity_") => {
            let page_number = s.split_at("next_activity_".len()).1;
            list_all_activity::update(&ctx, &component_interaction, page_number, db_type).await?
        }
        _ => trace!("does not exist."),
    }
    Ok(())
}
