use lavalink_rs::{hook, model::events, prelude::*};
use lavalink_rs::model::track::TrackData;
use serenity::all::ChannelId;
use serenity::http::Http;
use tracing::{error, info};

#[hook]
pub async fn raw_event(_: LavalinkClient, session_id: String, event: &serde_json::Value) {
	if event["op"].as_str() == Some("event") || event["op"].as_str() == Some("playerUpdate") {
		info!("{:?} -> {:?}", session_id, event);
	}
}

#[hook]
pub async fn ready_event(client: LavalinkClient, session_id: String, event: &events::Ready) {
	info!("{:?} -> {:?}", session_id, event);
	client.delete_all_player_contexts().await.unwrap();
}

#[hook]
pub async fn track_start(client: LavalinkClient, _session_id: String, event: &events::TrackStart) {
	info!("{:?}", event);
	let player_context = client.get_player_context(event.guild_id).unwrap();
	let data = match player_context
		.data::<(ChannelId, std::sync::Arc<Http>)>()
	{
		Ok(data) => data,
		Err(e) => {
			error!("{:#?}", e);
			return
		}
	};
	let (channel_id, http) = (&data.0, &data.1);

	let msg = {
		let track = &event.track;

		if let Some(uri) = &track.info.uri {
			format!(
				"Now playing: [{} - {}](<{}>) | Requested by <@!{}>",
				track.info.author,
				track.info.title,
				uri,
				track.user_data.clone().unwrap()["requester_id"]
			)
		} else {
			format!(
				"Now playing: {} - {} | Requested by <@!{}>",
				track.info.author,
				track.info.title,
				track.user_data.clone().unwrap()["requester_id"]
			)
		}
	};
}
