use crate::event_handler::Handler;
use crate::helper::error_management::error_dispatch;
use serenity::all::{CommandInteraction, Context};
use std::error::Error;

pub async fn dispatch_message_command(
    _ctx: &Context,
    command_interaction: &CommandInteraction,
    _self_handler: &Handler,
) -> Result<(), Box<dyn Error>> {
    match command_interaction.data.name.as_str() {
        _ => Err(Box::new(error_dispatch::Error::Option(String::from(
            "Unknown command",
        )))),
    }
}
