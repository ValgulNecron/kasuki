use std::env;
use std::path::Path;

use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde_json::{json, Value};
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::ChannelId;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::command::CommandOptionType::Attachment;
use serenity::model::prelude::interaction::application_command::{ApplicationCommandInteraction, CommandDataOption};
use serenity::model::Timestamp;
use serenity::utils::Colour;

pub async fn run(options: &[CommandDataOption], ctx: &Context, command: &ApplicationCommandInteraction) -> String {
    let option = options
        .get(0)
        .expect("Expected username option")
        .resolved
        .as_ref()
        .expect("Expected username object");
    if let CommandDataOptionValue::String(description) = option {
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
        let prompt = description;
        let api_key = env::var("AI_API_TOKEN").expect("token");
        let api_base_url = env::var("AI_API_BASE_URL").expect("token");
        let api_url = format!("{}images/generations", api_base_url);
        let client = reqwest::Client::new();

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap());
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let data = json!({
            "prompt": prompt,
            "n": 1,
            "size": "1024x1024"
        });

        let res: Value = client.post(api_url)
            .headers(headers)
            .json(&data)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        let mut url_string = "";
        if let Some(data) = res.get("data") {
            if let Some(object) = data.get(0) {
                if let Some(url) = object.get("url") {
                    url_string = url.as_str().unwrap();
                }
            }
        }

        let mut real_message = message.unwrap();
        let response = reqwest::get(url_string).await.unwrap();
        let bytes = response.bytes().await.unwrap();
        let filename = "image.png";
        std::fs::write(filename, &bytes).unwrap();

        let path = Path::new(filename);

        real_message.edit(&ctx.http, |m|
            m.attachment(path)
                .embed((|e| {
                    e.title("Here your generated image")
                        .image(format!("attachment://{}", filename))
                        .timestamp(Timestamp::now())
                        .color(color)
                })
                )).await.expect("TODO");

        std::fs::remove_file(filename);
    }
    return "good".to_string();
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("image").description("generate an image").create_option(
        |option| {
            option
                .name("description")
                .description("Description of the image you want to gen.")
                .kind(CommandOptionType::String)
                .required(true)
        },
    )
}