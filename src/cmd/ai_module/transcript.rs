use std::env;
use std::fs::File;
use std::io::{empty, Write, copy};
use std::path::Path;

use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use reqwest::multipart;
use serde_json::{json, Value};
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::futures::AsyncWriteExt;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::channel::AttachmentType;
use serenity::model::prelude::ChannelId;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::command::CommandOptionType::Attachment;
use serenity::model::prelude::interaction::application_command::{ApplicationCommandInteraction, CommandDataOption};
use serenity::model::Timestamp;
use serenity::utils::Colour;

pub async fn run(options: &[CommandDataOption], ctx: &Context, command: &ApplicationCommandInteraction) -> String {
    let option = options
        .get(0)
        .expect("Expected attachement option")
        .resolved
        .as_ref()
        .expect("Expected attachement object");
    if let CommandDataOptionValue::Attachment(attachement) = option {
        let content_type = attachement.content_type.clone().unwrap();
        println!("{}", content_type);
        let content = attachement.proxy_url.clone();
        println!("{}", content);

        if !content_type.starts_with("audio/") && !content_type.starts_with("video/"){
            return "wrong file type".to_string();
        }

        let allowed_extensions = vec![
                "mp3", "mp4", "mpeg", "mpga", "m4a", "wav", "webm",
            ];
       let extension = match content.split('/').nth(1) {
        Some(extension) => extension,
            None => {
                return "invalid content type".to_string();
            }
        };

        if !allowed_extensions.contains(&extension) {
            return "wrong file extension".to_string();
        }

        let response = reqwest::get(content).await.expect("download");

        let client = reqwest::Client::new();

        let fname = Path::new("./").join(format!("video.{}", extension));
        let string_fname = format!("./video.{}", extension);
        let mut file = File::create(fname.clone()).expect("file name");
        let content = response.text().await.expect("response");
        copy(&mut content.as_bytes(),&mut file);

        let color = Colour::FABLED_PINK;

        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
            })
            .await
        {
            println!("Cannot respond to slash command: {}", why);
        }

        let message = command
            .create_followup_message(&ctx.http, |f| {
                f.embed(|e| {
                    e.title("In progress")
                        .description("The task is being processed...be patient, it may take some time!")
                        .timestamp(Timestamp::now())
                        .color(color)
                })
            })
            .await;

        let my_path = ".\\src\\.env";
        let path = std::path::Path::new(my_path);
        dotenv::from_path(path);
        let api_key = env::var("AI_API_TOKEN").expect("token");
        let api_base_url = env::var("AI_API_BASE_URL").expect("token");
        let api_url = format!("{}audio/transcriptions", api_base_url);
        let client = reqwest::Client::new();

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap());
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("multipart/form-data"));

        let data = json!({
            "model": "whisper-1"
        });

        let path = multipart::Part::stream(string_fname);
        println!("form");
        let form = multipart::Form::new()
            .part("file", path)
            .text("data", data.to_string());
        println!("after form");

        let res: Value = client.post(api_url)
            .headers(headers)
            .multipart(form)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        println!("after request");
        println!("{}", res)
    }
    return "good".to_string();
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("transcript").description("generate an image").create_option(
        |option| {
            option
                .name("video")
                .description("File of the video you want the transcript of 10mb max.")
                .kind(Attachment)
                .required(true)
        },
    )
}