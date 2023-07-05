use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::Timestamp;
use serenity::utils::Colour;

use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::lang_struct::InProgressLocalisedText;

pub async fn in_progress_embed(ctx: &Context, command: &ApplicationCommandInteraction) -> serenity::Result<Message> {
    let color = Colour::FABLED_PINK;
    let mut file = File::open("lang_file/general/in_progress.json").expect("Failed to open file");
    let mut json = String::new();
    file.read_to_string(&mut json).expect("Failed to read file");

    let json_data: HashMap<String, InProgressLocalisedText> =
        serde_json::from_str(&json).expect("Failed to parse JSON");

    let guild_id = command.guild_id.unwrap().0.to_string().clone();
    let lang_choice = get_guild_langage(guild_id).await;

    let message;
    if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
        message = command
            .create_followup_message(&ctx.http, |f| {
                f.embed(|e| e.title(&localised_text.title)
                    .description(&localised_text.description)
                    .timestamp(Timestamp::now())
                    .color(color))
            })
            .await;
    } else {
        message = command.create_followup_message(&ctx.http, |f| {
            f.embed(|e| e.title("Error"))
        }).await;
    }

    return message;
}