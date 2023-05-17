use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::prelude::interaction::application_command::CommandDataOption;

pub fn run(_options: &[CommandDataOption]) -> String {
    let mut builder = String::new();
    builder.push_str("Hey, I'm alive!");
    builder.to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("ping").description("A ping command")
}

