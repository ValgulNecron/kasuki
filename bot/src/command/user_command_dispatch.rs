use std::error::Error;

use crate::command::command_trait::UserCommand;
use crate::command::user::avatar::AvatarCommand;
use crate::command::user::banner::BannerCommand;
use crate::command::user::profile::ProfileCommand;
use crate::event_handler::Handler;
use crate::helper::error_management::error_dispatch;
use serenity::all::{CommandInteraction, Context};

pub async fn dispatch_user_command(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    self_handler: &Handler,
) -> Result<(), Box<dyn Error>> {

    match command_interaction.data.name.as_str() {
        "avatar" => {
            AvatarCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
            }
            .run_user()
            .await
        }
        "banner" => {
            BannerCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
            }
            .run_user()
            .await
        }
        "profile" => {
            ProfileCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
            }
            .run_user()
            .await
        }
        _ => Err(Box::new(error_dispatch::Error::Option(String::from(
            "Unknown command",
        )))),
    }
}
