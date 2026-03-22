pub use crate::bot_data::BotData;
use serenity::all::FullEvent;
use serenity::async_trait;
use serenity::prelude::{Context as SerenityContext, EventHandler};
use tracing::trace;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
	async fn dispatch(&self, ctx: &SerenityContext, event: &FullEvent) {
		match event {
			FullEvent::GuildCreate { guild, is_new } => {
				self.guild_create(ctx, guild.clone(), *is_new).await;
			},
			FullEvent::GuildMemberAddition { new_member } => {
				self.guild_member_addition(ctx, new_member.clone()).await;
			},
			FullEvent::GuildMembersChunk { chunk } => {
				self.guild_members_chunk(ctx, chunk.clone()).await;
			},
			FullEvent::PresenceUpdate { old_data, new_data } => {
				self.presence_update(ctx, old_data.clone(), new_data.clone())
					.await;
			},
			FullEvent::Ready { data_about_bot } => {
				self.ready(ctx, data_about_bot.clone()).await;
			},
			FullEvent::InteractionCreate { interaction } => {
				self.interaction_create(ctx, interaction.clone()).await;
			},
			FullEvent::EntitlementCreate { entitlement } => {
				self.entitlement_create(ctx, entitlement.clone()).await;
			},
			FullEvent::EntitlementUpdate { entitlement } => {
				self.entitlement_update(ctx, entitlement.clone()).await;
			},
			FullEvent::EntitlementDelete { entitlement } => {
				self.entitlement_delete(ctx, entitlement.clone()).await;
			},
			FullEvent::Message { new_message } => {
				self.new_message(ctx, new_message.clone()).await;
			},
			FullEvent::VoiceStateUpdate { old, new } => {
				self.voice_state_update(ctx, old.clone(), new.clone()).await;
			},
			_ => {
				trace!("this event is not handled nothing to worry {:?}", event)
			},
		}
	}
}
