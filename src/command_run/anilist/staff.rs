use serenity::all::{
    CommandDataOption, CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp,
};

use crate::anilist_struct::run::staff::StaffWrapper;
use crate::constant::{COLOR, COMMAND_SENDING_ERROR};
use crate::error_enum::AppError;
use crate::lang_struct::anilist::staff::load_localization_staff;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<(), AppError> {
    let mut value = String::new();
    for option_data in options {
        if option_data.name.as_str() != "type" {
            let option_value = option_data.value.as_str().clone().unwrap();
            value = option_value.to_string().clone()
        }
    }

    let data: StaffWrapper = if value.parse::<i32>().is_ok() {
        StaffWrapper::new_staff_by_id(value.parse().unwrap()).await?
    } else {
        StaffWrapper::new_staff_by_search(&value).await?
    };

    let guild_id = match command.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let staff_localised = load_localization_staff(guild_id).await?;
    let staff = data.data.staff.clone();

    let mut date = String::new();
    let mut day = false;
    let mut month = false;

    match staff.date_of_birth.month {
        Some(m) => {
            month = true;
            date.push_str(m.to_string().as_str())
        }
        None => {}
    }
    match staff.date_of_birth.day {
        Some(d) => {
            day = true;
            if month {
                date.push_str("/")
            }
            date.push_str(d.to_string().as_str())
        }
        None => {}
    }
    match staff.date_of_birth.year {
        Some(y) => {
            if day {
                date.push_str("/")
            }
            date.push_str(y.to_string().as_str())
        }
        None => {}
    }
    let dob = staff_localised
        .date_of_birth
        .replace("$date$", date.as_str());

    let mut date = String::new();
    let mut day = false;
    let mut month = false;

    match staff.date_of_death.month {
        Some(m) => {
            month = true;
            date.push_str(m.to_string().as_str())
        }
        None => {}
    }
    match staff.date_of_death.day {
        Some(d) => {
            day = true;
            if month {
                date.push_str("/")
            }
            date.push_str(d.to_string().as_str())
        }
        None => {}
    }
    match staff.date_of_death.year {
        Some(y) => {
            if day {
                date.push_str("/")
            }
            date.push_str(y.to_string().as_str())
        }
        None => {}
    }
    let dod = staff_localised
        .date_of_death
        .replace("$date$", date.as_str());
    let desc = staff_localised
        .desc
        .replace("$dob$", dob.as_str())
        .replace("$dod$", dod.as_str())
        .replace("$job$", staff.primary_occupations[0].as_str())
        .replace(
            "$gender$",
            staff
                .gender
                .clone()
                .unwrap_or(String::from("Unknown."))
                .as_str(),
        )
        .replace("$age$", staff.age.unwrap_or(0).to_string().as_str());

    let name = staff
        .name
        .full
        .clone()
        .unwrap_or(staff.name.native.clone().unwrap());

    let va = staff
        .characters
        .nodes
        .iter()
        .filter_map(|x| {
            let full = x.name.full.as_ref().map(|s| s.as_str());
            let native = x.name.native.as_ref().map(|s| s.as_str());
            match (full, native) {
                (Some(full), Some(native)) => Some(format!("{}/{}", full, native)),
                (Some(full), None) => Some(full.to_string()),
                (None, Some(native)) => Some(native.to_string()),
                (None, None) => None,
            }
        })
        .take(5)
        .collect::<Vec<String>>()
        .join("\n");

    let media = staff
        .staff_media
        .edges
        .iter()
        .filter_map(|x| {
            let romaji = x.node.title.romaji.as_ref().map(|s| s.as_str());
            let english = x.node.title.english.as_ref().map(|s| s.as_str());
            match (romaji, english) {
                (Some(romaji), Some(english)) => Some(format!("{}/{}", romaji, english)),
                (Some(romaji), None) => Some(romaji.to_string()),
                (None, Some(english)) => Some(english.to_string()),
                (None, None) => None,
            }
        })
        .take(5)
        .collect::<Vec<String>>()
        .join("\n");

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .description(desc)
        .title(name)
        .url(staff.site_url)
        .thumbnail(staff.image.large)
        .field(&staff_localised.field2_title, va, true)
        .field(&staff_localised.field1_title, media, true);
    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command
        .create_response(&ctx.http, builder)
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())
}
