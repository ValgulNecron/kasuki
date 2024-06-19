use crate::helper::error_management::error_enum::AppError;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::helper::vndbapi::staff::get_staff;
use crate::structure::message::vn::staff::load_localization_staff;
use serenity::all::{CommandInteraction, Context};
use tracing::trace;

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };
    let map = get_option_map_string_subcommand(command_interaction);
    trace!("{:?}", map);
    let staff = map
        .get(&String::from("name"))
        .cloned()
        .unwrap_or(String::new());
    let staff_localised = load_localization_staff(guild_id).await?;

    let staff = get_staff(staff.clone()).await?;
    let staff = staff.results[0].clone();
    Ok(())
}
