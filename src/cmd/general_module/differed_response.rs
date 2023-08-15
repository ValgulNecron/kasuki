use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use serenity::client::Context;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::InteractionResponseType;

use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::lang_struct::DifferedResponseLocalisedText;

pub async fn differed_response(ctx: &Context, command: &ApplicationCommandInteraction) -> String {
    let mut file = File::open("lang_file/embed/ai/transcript.json").expect("Failed to open file");
    let mut json = String::new();
    file.read_to_string(&mut json).expect("Failed to read file");

    let json_data: HashMap<String, DifferedResponseLocalisedText> =
        serde_json::from_str(&json).expect("Failed to parse JSON");

    let guild_id = command.guild_id.unwrap().0.to_string().clone();
    let lang_choice = get_guild_langage(guild_id).await;

    if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
            })
            .await
        {
            println!("Cannot respond to slash command: {}", why);
            return format!("{}: {}", localised_text.error_slash_command, why);
        }
        return "good".to_string();
    } else {
        return "Language not found".to_string();
    }
}

pub async fn differed_response_with_file_deletion(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    file_to_delete: PathBuf,
) -> String {
    let mut file = File::open("lang_file/ai/transcript.json").expect("Failed to open file");
    let mut json = String::new();
    file.read_to_string(&mut json).expect("Failed to read file");

    let json_data: HashMap<String, DifferedResponseLocalisedText> =
        serde_json::from_str(&json).expect("Failed to parse JSON");

    let guild_id = command.guild_id.unwrap().0.to_string().clone();
    let lang_choice = get_guild_langage(guild_id).await;

    if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
            })
            .await
        {
            let _ = fs::remove_file(&file_to_delete);
            println!("Cannot respond to slash command: {}", why);
            return format!("{}: {}", localised_text.error_slash_command, why);
        }
        return "good".to_string();
    } else {
        return "Language not found".to_string();
    }
}
