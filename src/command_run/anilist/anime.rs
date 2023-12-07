use crate::anilist_struct::run::media::MediaWrapper;
use crate::common::html_parser::convert_to_discord_markdown;
use crate::common::trimer::trim;
use crate::constant::{COLOR, COMMAND_SENDING_ERROR, OPTION_ERROR};
use crate::error_enum::AppError;
use crate::error_enum::AppError::{LangageGuildIdError, NoCommandOption};
use serenity::all::{
    CommandDataOption, CommandDataOptionValue, CommandInteraction, Context, CreateEmbed,
    CreateInteractionResponse, CreateInteractionResponseMessage, Timestamp,
};

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<(), AppError> {
    let option = &options.get(0).ok_or(OPTION_ERROR.clone())?.value;

    let value = match option {
        CommandDataOptionValue::String(lang) => lang,
        _ => {
            return Err(NoCommandOption(String::from(
                "The command contain no option.",
            )));
        }
    };

    let data: MediaWrapper = if value.parse::<i32>().is_ok() {
        MediaWrapper::new_anime_by_id(value.parse().unwrap()).await?
    } else {
        MediaWrapper::new_anime_by_search(value).await?
    };

    let guild_id = command
        .guild_id
        .ok_or(LangageGuildIdError(String::from(
            "Guild id for langage not found.",
        )))?
        .to_string();

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .description(embed_desc(&data))
        .title(embed_title(&data));

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command
        .create_response(&ctx.http, builder)
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())
}

fn embed_title(data: &MediaWrapper) -> String {
    let en = data.data.media.title.english.clone();
    let rj = data.data.media.title.romaji.clone();
    let en = en.unwrap_or(String::from(""));
    let rj = rj.unwrap_or(String::from(""));
    let mut title = String::new();
    let mut total = 0;
    match en.as_str() {
        "\"\"" => {}
        _ => {
            total += 1;
            title.push_str(en.as_str())
        }
    }

    match rj.as_str() {
        "\"\"" => {}
        _ => {
            if total == 1 {
                title.push_str(" / ");
                title.push_str(en.as_str())
            } else {
                title.push_str(en.as_str())
            }
        }
    }

    title
}

fn embed_desc(data: &MediaWrapper) -> String {
    let mut desc = data
        .data
        .media
        .description
        .clone()
        .unwrap_or_else(|| "".to_string());
    desc = convert_to_discord_markdown(desc);
    let lenght_diff = 4096 - desc.len() as i32;
    if lenght_diff <= 0 {
        desc = trim(desc, lenght_diff)
    }
    desc
}
