use crate::constant::COLOR;
use crate::function::sql::sqlite::pool::get_sqlite_pool;
use crate::structure::embed::anilist::struct_lang_register::RegisterLocalisedText;
use crate::structure::register::anilist::struct_register_register::RegisterLocalisedRegister;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    ApplicationCommandInteraction, CommandDataOption,
};
use serenity::model::Timestamp;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    let database_url = "./data.db";
    let pool = get_sqlite_pool(database_url).await;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS registered_user (
            user_id TEXT PRIMARY KEY,
            anilist_username TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .unwrap();
    let option = options
        .get(0)
        .expect("Expected username option")
        .resolved
        .as_ref()
        .expect("Expected username object");

    if let CommandDataOptionValue::String(_) = option {
        let user_id = &command.user.id.to_string();
        let username = &command.user.name;
        let user_pfp_ref = &command.user.avatar.as_ref().unwrap();
        let profile_picture;
        if let Some(first) = user_pfp_ref.split('_').next() {
            if first == "a" {
                profile_picture = format!(
                    "https://cdn.discordapp.com/avatars/{}/{}.gif?size=1024",
                    user_id, user_pfp_ref
                );
            } else {
                profile_picture = format!(
                    "https://cdn.discordapp.com/avatars/{}/{}.webp?size=1024",
                    user_id, user_pfp_ref
                );
            }
        } else {
            profile_picture = format!(
                "https://cdn.discordapp.com/avatars//{}/{}.webp?size=1024",
                user_id, user_pfp_ref
            );
        }
        sqlx::query(
            "INSERT OR REPLACE INTO registered_user (user_id, anilist_username) VALUES (?, ?)",
        )
        .bind(user_id)
        .bind(username)
        .execute(&pool)
        .await
        .unwrap();

        let localised_text = match RegisterLocalisedText::get_register_localised(ctx, command).await
        {
            Ok(data) => data,
            Err(_) => return,
        };
        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.embed(|m| {
                            m.title(username)
                                // Add a timestamp for the current time
                                // This also accepts a rfc3339 Timestamp
                                .timestamp(Timestamp::now())
                                .thumbnail(profile_picture)
                                .color(COLOR)
                                .description(format!(
                                    "{}{}({}){}{}{}",
                                    &localised_text.part_1,
                                    username,
                                    user_id,
                                    &localised_text.part_2,
                                    username,
                                    &localised_text.part_3
                                ))
                        })
                    })
            })
            .await
        {
            println!("{}: {}", localised_text.error_slash_command, why);
        }
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    let registers = RegisterLocalisedRegister::get_register_register_localised().unwrap();
    let command = command
        .name("register")
        .description("Register your anilist username for ease of use.")
        .create_option(|option| {
            let option = option
                .name("username")
                .description("Your anilist user name.")
                .kind(CommandOptionType::String)
                .required(true);
            for register in registers.values() {
                option
                    .name_localized(&register.code, &register.option1)
                    .description_localized(&register.code, &register.option1_desc);
            }
            option
        });
    for register in registers.values() {
        command
            .name_localized(&register.code, &register.name)
            .description_localized(&register.code, &register.desc);
    }
    command
}
