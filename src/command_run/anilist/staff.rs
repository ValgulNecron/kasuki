use serenity::all::{
    CommandDataOption, CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp,
};

use crate::anilist_struct::run::staff::StaffWrapper;
use crate::common::get_option_value::get_option;
use crate::constant::{COLOR, };
use crate::error_enum::AppError;
use crate::error_enum::AppError::CommandSendingError;
use crate::lang_struct::anilist::staff::load_localization_staff;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), AppError> {
    let value = get_option(options);

    let data: StaffWrapper = if value.parse::<i32>().is_ok() {
        StaffWrapper::new_staff_by_id(value.parse().unwrap()).await?
    } else {
        StaffWrapper::new_staff_by_search(&value).await?
    };

    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let staff_localised = load_localization_staff(guild_id).await?;
    let staff = data.data.staff.clone();

    let mut date = String::new();
    let mut day = false;
    let mut month = false;

    if let Some(m) = staff.date_of_birth.month {
        month = true;
        date.push_str(m.to_string().as_str())
    }

    if let Some(d) = staff.date_of_birth.day {
        day = true;
        if month {
            date.push('/')
        }
        date.push_str(d.to_string().as_str())
    }

    if let Some(y) = staff.date_of_birth.year {
        if day {
            date.push('/')
        }
        date.push_str(y.to_string().as_str())
    }

    let dob = staff_localised
        .date_of_birth
        .replace("$date$", date.as_str());

    let mut date = String::new();
    let mut day = false;
    let mut month = false;
    if let Some(m) = staff.date_of_death.month {
        month = true;
        date.push_str(m.to_string().as_str())
    }

    if let Some(d) = staff.date_of_death.day {
        day = true;
        if month {
            date.push('/')
        }
        date.push_str(d.to_string().as_str())
    }

    if let Some(y) = staff.date_of_death.year {
        if day {
            date.push('/')
        }
        date.push_str(y.to_string().as_str())
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
            let full = x.name.full.as_deref();
            let native = x.name.native.as_deref();
            get_full_name(full, native)
        })
        .take(5)
        .collect::<Vec<String>>()
        .join("\n");

    let media = staff
        .staff_media
        .edges
        .iter()
        .filter_map(|x| {
            let romaji = x.node.title.romaji.as_deref();
            let english = x.node.title.english.as_deref();
            get_full_name(romaji, english)
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

    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|e| CommandSendingError(format!("Error while sending the command {}", e)))
}

fn get_full_name(a: Option<&str>, b: Option<&str>) -> Option<String> {
    match (a, b) {
        (Some(a), Some(b)) => Some(format!("{}/{}", a, b)),
        (Some(a), None) => Some(a.to_string()),
        (None, Some(b)) => Some(b.to_string()),
        (None, None) => None,
    }
}
