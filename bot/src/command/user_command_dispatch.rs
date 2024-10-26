use anyhow::{anyhow, Result};

use crate::command::command_trait::UserCommand;
use crate::command::user::avatar::AvatarCommand;
use crate::command::user::banner::BannerCommand;
use crate::command::user::profile::ProfileCommand;
use crate::event_handler::BotData;
use serenity::all::{CommandInteraction, Context as SerenityContext};

pub async fn dispatch_user_command(
    ctx: &SerenityContext,
    command_interaction: &CommandInteraction,
) -> Result<()> {
    let bot_data = ctx.data::<BotData>().clone();
    match command_interaction.data.name.as_str() {
        "avatar" => {
            AvatarCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: bot_data.config.clone(),
            }
            .run_user()
            .await
        }
        "banner" => {
            BannerCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: bot_data.config.clone(),
            }
            .run_user()
            .await
        }
        "profile" => {
            ProfileCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: bot_data.config.clone(),
            }
            .run_user()
            .await
        }
        _ => Err(anyhow!("Unknown command")),
    }
}
