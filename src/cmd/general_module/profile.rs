use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::http::client::Http;
use serenity::model::application::command::CommandOptionType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::application_command::{CommandDataOption, CommandDataOptionValue};
use serenity::model::user::User;
use serenity::model::Timestamp;
use serenity::utils::{Colour, Member};

use crate::cmd::general_module::error_handling::{error_cant_read_file, error_file_not_found, error_message, error_message_with_why, error_no_avatar, error_no_guild_id, error_parsing_json, no_langage_error};
use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::lang_struct::ProfileLocalisedText;
use crate::cmd::general_module::lang_struct_register::RegisterLocalisedProfile;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    return if let Some(option) = options.get(0) {
        let resolved = option.resolved.as_ref().unwrap();
        if let CommandDataOptionValue::User(user, ..) = resolved {
            let result = profile_with_user(ctx, command, &user).await;
            result
        } else {
            let result = profile_without_user(ctx, command).await;
            result
        }
    } else {
        let result = profile_without_user(ctx, command).await;
        result
    };
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    type RegisterLocalisedProfileList = HashMap<String, RegisterLocalisedProfile>;
    let mut file =
        File::open("lang_file/command_register/general/profile.json").expect("Failed to open file");
    let mut json = String::new();
    file.read_to_string(&mut json).expect("Failed to read file");
    let profiles: RegisterLocalisedProfileList = serde_json::from_str(&json).unwrap();
    let command = command
        .name("profile")
        .description("Show the profile of a user")
        .create_option(|option| {
            let option = option
                .name("user")
                .description("The user you wan the profile of")
                .kind(CommandOptionType::User)
                .required(false);
            for (_key, profile) in &profiles {
                option
                    .name_localized(&profile.code, &profile.option1)
                    .description_localized(&profile.code, &profile.option1_desc);
            }
            option
        });
    for (_key, profile) in &profiles {
        command
            .name_localized(&profile.code, &profile.profile)
            .description_localized(&profile.code, &profile.desc);
    }
    command
}

pub async fn profile_without_user(ctx: &Context, command: &ApplicationCommandInteraction) {
    let mut file = File::open("lang_file/embed/general/profile.json").expect("Failed to open file");
    let mut json = String::new();
    file.read_to_string(&mut json).expect("Failed to read file");

    let json_data: HashMap<String, ProfileLocalisedText> =
        serde_json::from_str(&json).expect("Failed to parse JSON");

    let guild_id = command.guild_id.unwrap().0.to_string().clone();
    let lang_choice = get_guild_langage(guild_id).await;

    return if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
        let color = Colour::FABLED_PINK;

        let user = command.user.id.0;
        let real_user = Http::get_user(&ctx.http, user).await;
        let result = if let Ok(user) = real_user {
            user
        } else {
            error_message(color, ctx, command, &localised_text.error_no_user).await;
            return;
        };
        let avatar_url = result.avatar_url().unwrap();

        let desc = description(result.clone(), command, ).await;

        send_embed(
            avatar_url,
            desc,
            color,
            ctx,
            command,
            localised_text.clone(),
            result,
        )
        .await
    } else {
        let color = Colour::FABLED_PINK;
        no_langage_error(color, ctx, command).await
    };
}

pub async fn profile_with_user(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    user_data: &User,
) {
    let color = Colour::FABLED_PINK;
    let mut file = match File::open("lang_file/embed/general/profile.json") {
        Ok(mut file) => file,
        Err(_) => {
            error_file_not_found(color, ctx, command).await;
            return;
        }
    };
    let mut json = String::new();
    match file.read_to_string(&mut json) {
        Ok(_) => {
        }
        Err(_) => {
            error_cant_read_file(color, ctx, command).await
        }
    }

    let json_data: HashMap<String, ProfileLocalisedText> = match serde_json::from_str(&json) {
        Ok(data) => data,
        Err(_) => {
            error_parsing_json(color, ctx, command).await;
            return;
        }
    };

    let guild_id = match command.guild_id {
        Some(id) => id.0.to_string(),
        None => {
            error_no_guild_id(color, ctx, command).await;
            return;
        }
    };
    let lang_choice = get_guild_langage(guild_id).await;

    if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
        let user = user_data.id.0;
        let real_user = Http::get_user(&ctx.http, user).await;
        let result = if let Ok(user) = real_user {
            user
        } else {
            error_message(color, ctx, command, &localised_text.error_no_user).await;
            return;
        };

        let avatar_url = match result.avatar_url() {
            Some(url) => url,
            None => {
                error_no_avatar(color, ctx, command).await;
                return;
            }
        };

        let desc = description(result.clone(), command, localised_text).await;

        send_embed(
            avatar_url,
            desc,
            color,
            ctx,
            command,
            localised_text.clone(),
            result,
        )
        .await
    } else {
        no_langage_error(color, ctx, command).await
    }
}

pub async fn description(user: User, command: &ApplicationCommandInteraction, localised_text: &ProfileLocalisedText) -> String {
    let is_bot = &user.bot;
    let public_flag = &user.public_flags.unwrap();
    let user_id = &user.id;
    let created_at = &user.created_at();
    let member = &command.member.clone().unwrap();
    let joined_at = member.joined_at.unwrap_or(Timestamp::from(0));
    let desc = format!(
        "\n {}{} \n {}{} \n {}{:?} \n {}{} \n {}{}",
        &localised_text.user_id,
        user_id,
        &localised_text.is_bot,
        is_bot,
        &localised_text.public_flag,
        public_flag,
        &localised_text.created_at,
        created_at,
        &localised_text.joined_at,
        joined_at
    );

    return desc;
}

pub async fn send_embed(
    avatar_url: String,
    desc: String,
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    localised_text: ProfileLocalisedText,
    result: User,
) {
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.embed(|m| {
                        m.title(format!("{}{}", &localised_text.title, result.name))
                            // Add a timestamp for the current time
                            // This also accepts a rfc3339 Timestamp
                            .timestamp(Timestamp::now())
                            .color(color)
                            .thumbnail(avatar_url)
                            .description(desc)
                    })
                })
        })
        .await
    {
        println!("{}: {}", &localised_text.error_slash_command, why);
        error_message_with_why(
            color,
            ctx,
            command,
            &localised_text.error_slash_command,
            why,
        )
        .await
    }
}
