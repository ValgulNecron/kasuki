use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use serenity::all::{CommandInteraction, Context};

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };
    let map = get_option_map_string_subcommand(command_interaction);
    let game = map.get(&String::from("name")).ok_or(AppError::new(
        String::from("There is no option 1"),
        ErrorType::Option,
        ErrorResponseType::Message,
    ))?;

    Ok(())
}
