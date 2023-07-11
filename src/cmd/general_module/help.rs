use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;

pub struct Command {
    command_name: String,
    command_description: String,
    command_option: Vec<Option>,
}

pub struct Option {
    option_name: String,
    option_type: String,
    option_description: String,
}

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> String {
    let command_list: Vec<Command> = Vec::new();

    let user_option1 = Option {
        option_name: "username".parse().unwrap(),
        option_type: "String".parse().unwrap(),
        option_description: "Username of the anilist user you want to check"
            .parse()
            .unwrap(),
    };

    let mut user_option: Vec<Option> = Vec::new();
    user_option.push(user_option1);

    let user_command = Command {
        command_name: "user".parse().unwrap(),
        command_description: "Info of an anilist user".parse().unwrap(),
        command_option: user_option,
    };

    return "good".to_string();
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("random").description("Get a random anime.")
}
