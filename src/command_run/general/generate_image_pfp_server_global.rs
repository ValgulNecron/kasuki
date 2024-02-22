use serenity::all::{
    CommandInteraction, Context
    ,
};

use crate::command_run::general::generate_image_pfp_server::send_embed;
use crate::error_management::error_enum::AppError;

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    send_embed(ctx, command_interaction, "global").await
}
