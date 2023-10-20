use serenity::builder::CreateApplicationCommand;
use serenity::model::application::command::CommandOptionType;
use crate::cmd::lang_struct::register::general::struct_avatar_register::RegisterLocalisedAvatar;



pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    let profiles = RegisterLocalisedAvatar::get_avatar_register_localised().unwrap();
    let command = command
        .name("profile")
        .description("Show the profile of a user")
        .create_option(|option| {
            let option = option
                .name("user")
                .description("The user you wan the profile of")
                .kind(CommandOptionType::User)
                .required(false);
            for profile in profiles.values() {
                option
                    .name_localized(&profile.code, &profile.option1)
                    .description_localized(&profile.code, &profile.option1_desc);
            }
            option
        });
    for  profile in profiles.values() {
        command
            .name_localized(&profile.code, &profile.name)
            .description_localized(&profile.code, &profile.description);
    }
    command
}