use crate::autocomplete::autocomplete_dispatch::autocomplete_dispatching;
use crate::command::command_dispatch::{dispatch_command, dispatch_user_command};
use crate::components::components_dispatch::components_dispatching;
use crate::error_management::error_dispatch;
use crate::event_handler::{BotData, Handler};
use crate::handlers::user_db::add_user_data_to_db;
use serenity::all::{CommandType, Interaction};
use serenity::prelude::Context as SerenityContext;
use tracing::{error, trace, warn};

impl Handler {
	pub(crate) async fn interaction_create(&self, ctx: &SerenityContext, interaction: Interaction) {
		let mut user = None;
		let bot_data = ctx.data::<BotData>().clone();
		trace!("Interaction received: {:?}", interaction.kind());

		match interaction {
			Interaction::Command(command_interaction) => {
				let mut message = String::from("");
				match command_interaction.data.kind {
					CommandType::ChatInput => {
						if let Err(e) = dispatch_command(ctx, &command_interaction).await {
							error!(error = ?e, "Error executing command");
							message = e.to_string();
						} else {
							return;
						}
					},
					CommandType::User => {
						if let Err(e) = dispatch_user_command(ctx, &command_interaction).await {
							error!(error = ?e, "Error executing user command");
							message = e.to_string();
						} else {
							return;
						}
					},
					CommandType::Message => trace!("{:?}", command_interaction),
					_ => {},
				}
				error_dispatch::command_dispatching(message, &command_interaction, ctx).await;
				user = Some(command_interaction.user);
			},
			Interaction::Autocomplete(autocomplete_interaction) => {
				user = Some(autocomplete_interaction.user.clone());
				autocomplete_dispatching(ctx, autocomplete_interaction).await;
			},
			Interaction::Component(component_interaction) => {
				let db_connection = bot_data.db_connection.clone();
				if let Err(e) =
					components_dispatching(ctx, &component_interaction, db_connection).await
				{
					warn!(error = ?e, "Failed to dispatch component interaction");
				}
				user = Some(component_interaction.user);
			},
			_ => {},
		}

		if let Some(user) = user {
			if let Err(e) = add_user_data_to_db(user.clone(), bot_data.db_connection.clone()).await
			{
				warn!(
					user_id = %user.id,
					error = ?e,
					"Failed to insert user data from interaction into database"
				);
			}
		}
	}
}
