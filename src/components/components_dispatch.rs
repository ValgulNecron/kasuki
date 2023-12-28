use crate::error_enum::AppError;
use serenity::all::{ComponentInteraction, Context};

pub async fn components_dispatching(
    ctx: Context,
    component_interaction: ComponentInteraction,
) -> Result<(), AppError> {
    Ok(())
}
