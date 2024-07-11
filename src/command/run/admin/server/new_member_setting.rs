use crate::config::Config;
use crate::constant::{NEW_MEMBER_IMAGE_PATH, NEW_MEMBER_PATH};
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::get_option::subcommand_group::{
    get_option_map_attachment_subcommand_group, get_option_map_boolean_subcommand_group,
    get_option_map_channel_subcommand_group,
};
use crate::new_member::NewMemberSetting;
use crate::structure::message::admin::server::new_member_setting::load_localization_new_member_setting;
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage,
};
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tracing::trace;

pub async fn run(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
) -> Result<(), AppError> {
    let bool_map = get_option_map_boolean_subcommand_group(command_interaction);
    let attachment = get_option_map_attachment_subcommand_group(command_interaction);
    let channel = get_option_map_channel_subcommand_group(command_interaction);
    trace!(?bool_map, ?attachment, ?channel);
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };
    let show_username = *bool_map
        .get(&String::from("show_username"))
        .ok_or(AppError::new(
            String::from("There is no option 1"),
            ErrorType::Option,
            ErrorResponseType::Message,
        ))?;
    let show_time = *bool_map
        .get(&String::from("show_time"))
        .ok_or(AppError::new(
            String::from("There is no option 2"),
            ErrorType::Option,
            ErrorResponseType::Message,
        ))?;
    let channel_id = channel.get(&String::from("custom_channel"));
    let attachment = attachment.get(&String::from("custom_image"));

    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command_interaction
        .create_response(&ctx.http, builder_message)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {}", e),
                ErrorType::Option,
                ErrorResponseType::Message,
            )
        })?;
    let content = fs::read_to_string(NEW_MEMBER_PATH).unwrap_or_else(|_| String::new());
    let mut hashmap: HashMap<String, NewMemberSetting> =
        serde_json::from_str(&content).unwrap_or_else(|_| HashMap::new());
    let guild_specific = hashmap
        .get(&guild_id.to_string())
        .unwrap_or(&NewMemberSetting {
            custom_channel: false,
            channel_id: 0,
            custom_image: false,
            show_username: true,
            show_time_join: true,
        });

    trace!(?show_username, ?show_time, ?channel_id, ?attachment);
    let channel_id = match channel_id {
        Some(channel_id) => channel_id.to_string().parse::<u64>().map_err(|e| {
            AppError::new(
                format!("Error while parsing the channel id {}", e),
                ErrorType::Option,
                ErrorResponseType::Followup,
            )
        })?,
        None => guild_specific.channel_id,
    };
    let attachment = match attachment {
        Some(attachment) => {
            // create NEW_MEMBER_IMAGE_PATH if it doesn't exist
            fs::create_dir_all(NEW_MEMBER_IMAGE_PATH).map_err(|e| {
                AppError::new(
                    format!("Error while creating the directory {}", e),
                    ErrorType::Option,
                    ErrorResponseType::Followup,
                )
            })?;
            let bytes = attachment.download().await.map_err(|e| {
                AppError::new(
                    format!("Error while downloading the attachment {}", e),
                    ErrorType::Option,
                    ErrorResponseType::Followup,
                )
            })?;
            fs::write(format!("{}{}.png", NEW_MEMBER_IMAGE_PATH, guild_id), bytes).map_err(
                |e| {
                    AppError::new(
                        format!("Error while saving the attachment {}", e),
                        ErrorType::Option,
                        ErrorResponseType::Followup,
                    )
                },
            )?;
            true
        }
        None => guild_specific.custom_image,
    };
    let new_member_setting = NewMemberSetting {
        custom_channel: channel_id != 0,
        channel_id,
        custom_image: attachment,
        show_username,
        show_time_join: show_time,
    };
    trace!(?new_member_setting);
    // insert or update the new_member_setting to the hashmap
    hashmap.insert(guild_id.clone(), new_member_setting);
    trace!(?hashmap);
    // save the hashmap to the file
    fs::write(NEW_MEMBER_PATH, serde_json::to_string(&hashmap).unwrap()).map_err(|e| {
        AppError::new(
            format!("Error while saving the new member setting {}", e),
            ErrorType::Option,
            ErrorResponseType::Followup,
        )
    })?;

    let localised =
        load_localization_new_member_setting(guild_id.clone(), config.bot.config.db_type.clone())
            .await?;

    let embed = get_default_embed(None)
        .title(localised.title)
        .description(localised.description);
    let builder = CreateInteractionResponseFollowup::new().embed(embed);
    command_interaction
        .create_followup(&ctx.http, builder)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {}", e),
                ErrorType::Option,
                ErrorResponseType::Followup,
            )
        })?;

    Ok(())
}
