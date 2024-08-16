use crate::command::command_trait::{Command, SlashCommand};
use crate::components::anilist::list_all_activity::get_formatted_activity_list;
use crate::config::Config;
use crate::constant::ACTIVITY_LIST_LIMIT;
use crate::get_url;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::{FollowupError, ResponseError};
use crate::structure::database::prelude::ActivityData;
use crate::structure::message::anilist_server::list_all_activity::load_localization_list_activity;
use prost::bytes::BufMut;
use sea_orm::{EntityOrSelect, EntityTrait};
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context, CreateButton, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage,
};
use std::error::Error;
use std::sync::Arc;
use tracing::trace;

pub struct ListAllActivity {
    pub ctx: Context,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
}

impl Command for ListAllActivity {
    fn get_ctx(&self) -> &Context {
        &self.ctx
    }
    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for ListAllActivity {
    async fn run_slash(&self) -> Result<(), Box<dyn Error>> {
        send_embed(&self.ctx, &self.command_interaction, self.config.clone()).await
    }
}

async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
) -> Result<(), Box<dyn Error>> {
    let db_type = config.bot.config.db_type.clone();
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let list_activity_localised_text =
        load_localization_list_activity(guild_id, db_type.clone(), config.bot.config.clone())
            .await?;

    let guild_id = command_interaction
        .guild_id
        .ok_or(ResponseError::Option(String::from(
            "Could not get the id of the guild",
        )))?;

    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command_interaction
        .create_response(&ctx.http, builder_message)
        .await?;

    let connection = sea_orm::Database::connect(get_url(config.bot.config.clone())).await?;
    let list = ActivityData::select(EntityOrSelect::Select(
        ActivityData::find().filter(ActivityData::Column::GuildId.eq(&guild_id.to_string())),
    ))
    .all(connection)
    .await?;
    let len = list.len();
    let next_page = 1;

    let activity: Vec<String> = get_formatted_activity_list(list, 0);

    let join_activity = activity.join("\n");

    let builder_message = get_default_embed(None)
        .title(list_activity_localised_text.title)
        .description(join_activity);

    let mut response = CreateInteractionResponseFollowup::new().embed(builder_message);
    trace!("{:?}", len);
    trace!("{:?}", ACTIVITY_LIST_LIMIT);
    if len > ACTIVITY_LIST_LIMIT as usize {
        response = response.button(
            CreateButton::new(format!("next_activity_{}", next_page))
                .label(&list_activity_localised_text.next),
        )
    }

    let _ = command_interaction
        .create_followup(&ctx.http, response)
        .await?;
    Ok(())
}
