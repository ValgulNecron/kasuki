use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::sync::Arc;

use crate::command::command_trait::{Command, SlashCommand};
use crate::config::Config;
use crate::constant::{NEW_MEMBER_IMAGE_PATH, NEW_MEMBER_PATH};
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::ResponseError;
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

pub struct NewMemberSettingCommand {
    pub ctx: Context,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
}

impl Command for NewMemberSettingCommand {
    fn get_ctx(&self) -> &Context {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for NewMemberSettingCommand {
    async fn run_slash(&self) -> Result<(), Box<dyn Error>> {
        send_embed(&self.ctx, &self.command_interaction, self.config.clone()).await
    }
}

async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
) -> Result<(), Box<dyn Error>> {
    let bool_map = get_option_map_boolean_subcommand_group(command_interaction);
    let attachment = get_option_map_attachment_subcommand_group(command_interaction);
    let channel = get_option_map_channel_subcommand_group(command_interaction);
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };
    let show_username =
        *bool_map
            .get(&String::from("show_username"))
            .ok_or(ResponseError::Option(String::from(
                "There is no option for show_username",
            )))?;
    let show_time = *bool_map
        .get(&String::from("show_time"))
        .ok_or(ResponseError::Option(String::from(
            "There is no option for show_time",
        )))?;
    let channel_id = channel.get(&String::from("custom_channel"));
    let attachment = attachment.get(&String::from("custom_image"));

    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command_interaction
        .create_response(&ctx.http, builder_message)
        .await?;
    let content = fs::read_to_string(NEW_MEMBER_PATH).unwrap_or_else(|_| String::new());
    let mut hashmap: HashMap<String, NewMemberSetting> =
        serde_json::from_str(&content).unwrap_or_else(|_| HashMap::new());
    let guild_specific = hashmap
        .get(&guild_id.to_string())
        .cloned()
        .unwrap_or(NewMemberSetting::default());

    let channel_id = match channel_id {
        Some(channel_id) => channel_id.to_string().parse::<u64>()?,
        None => guild_specific.channel_id,
    };
    let attachment = match attachment {
        Some(attachment) => {
            // create NEW_MEMBER_IMAGE_PATH if it doesn't exist
            fs::create_dir_all(NEW_MEMBER_IMAGE_PATH)?;
            let bytes = attachment.download().await?;
            fs::write(format!("{}{}.png", NEW_MEMBER_IMAGE_PATH, guild_id), bytes)?;
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
    // insert or update the new_member_setting to the hashmap
    hashmap.insert(guild_id.clone(), new_member_setting);
    // save the hashmap to the file
    fs::write(NEW_MEMBER_PATH, serde_json::to_string(&hashmap)?)?;

    let localised =
        load_localization_new_member_setting(guild_id.clone(), config.bot.config.clone()).await?;

    let embed = get_default_embed(None)
        .title(localised.title)
        .description(localised.description);
    let builder = CreateInteractionResponseFollowup::new().embed(embed);
    command_interaction
        .create_followup(&ctx.http, builder)
        .await?;

    Ok(())
}
