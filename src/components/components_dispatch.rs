use crate::components::anilist::{list_all_activity, list_register_user};
use crate::error_enum::AppError;
use log::trace;
use serenity::all::{ComponentInteraction, Context};

pub async fn components_dispatching(
    ctx: Context,
    component_interaction: ComponentInteraction,
) -> Result<(), AppError> {
    match component_interaction.data.custom_id.as_str() {
        s if s.starts_with("next_user_") => {
            let user_id = s.split_at("next_user_".len()).1;
            list_register_user::update(&ctx, &component_interaction, user_id).await?
        }
        s if s.starts_with("next_activity_") => {
            let page_number = s.split_at("next_activity_".len()).1;
            list_all_activity::update(&ctx, &component_interaction, page_number).await?
        }
        _ => trace!("does not exist."),
    }
    Ok(())
}
