use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::Timestamp;
use serenity::utils::Colour;

use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::lang_struct::TranslationLocalisedText;

pub async fn translation_embed(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    text: String,
    message: Message,
) -> String {
    let color = Colour::FABLED_PINK;
    let mut real_message = message.clone();
    let mut file = File::open("lang_file/ai/translation.json").expect("Failed to open file");
    let mut json = String::new();
    file.read_to_string(&mut json).expect("Failed to read file");

    let json_data: HashMap<String, TranslationLocalisedText> =
        serde_json::from_str(&json).expect("Failed to parse JSON");

    let guild_id = command.guild_id.unwrap().0.to_string().clone();
    let lang_choice = get_guild_langage(guild_id).await;

    if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
        if let Err(why) = real_message
            .edit(&ctx.http, |m| {
                m.embed(|e| {
                    e.title(&localised_text.title)
                        .description(format!("{}", text))
                        .timestamp(Timestamp::now())
                        .color(color)
                })
            })
            .await
        {
            println!("{}: {}", &localised_text.error_slash_command, why);
        }
    } else {
        return "Language not found".to_string();
    }
    return "good".to_string();
}
