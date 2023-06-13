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

pub async fn run(options: &[CommandDataOption], ctx: &Context, command: &ApplicationCommandInteraction) -> String {
    let command_list: Vec<Command> = Vec::new();

    let user_option1 = Option {
        option_name: "username",
        option_type: "String",
        option_description: "Username of the anilist user you want to check",
    };

    let user_option: Vec<Option> = Vec::new;
    user_option.push(user_option1);

    let user_command = Command {
        command_name: "user",
        command_description: "Info of an anilist user",
        command_option: user_option,
    };

    return "good".to_string();
}


pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("random").description("Get a random anime.")
}
