use serenity::all::{ComponentInteraction, Context};
use tracing::trace;

use crate::components::anilist::{list_all_activity, list_register_user};
use crate::error_management::error_enum::AppError;

pub async fn components_dispatching(
    ctx: Context,
    component_interaction: ComponentInteraction,
) -> Result<(), AppError> {
    match component_interaction.data.custom_id.as_str() {
        s if s.starts_with("user_") => {
            let user_id = s.split_at("_".len()).1;
            let prev_id = user_id.split_at("_".len()).1;
            list_register_user::update(&ctx, &component_interaction, user_id, prev_id).await?
        }
        s if s.starts_with("next_activity_") => {
            let page_number = s.split_at("next_activity_".len()).1;
            list_all_activity::update(&ctx, &component_interaction, page_number).await?
        }
        _ => trace!("does not exist."),
    }
    Ok(())
}
