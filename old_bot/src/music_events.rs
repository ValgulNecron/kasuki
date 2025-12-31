//! This file implements Lavalink event hooks to handle various events such as raw Lavalink events, 
//! ready events, and track start events in a Discord bot using both `lavalink_rs` and `serenity` libraries.
use lavalink_rs::{hook, model::events, prelude::*};
use serenity::all::ChannelId;
use serenity::http::Http;
use tracing::{info, trace, warn};

/// Asynchronous hook to handle raw Lavalink events.
///
/// This function is triggered for any event received by the Lavalink client.
/// It specifically logs "event" and "playerUpdate" operations at the trace level for debugging.
///
/// # Parameters
/// - `_client`: The `LavalinkClient` instance (unused).
/// - `session_id`: The session ID of the Lavalink client.
/// - `event`: The raw JSON value of the event.
///
/// # Behavior
/// - Checks if the `op` field of the event is "event" or "playerUpdate".
/// - If it matches, logs the `session_id` and the `event` at the `trace` level.
#[hook]
pub async fn raw_event(_client: LavalinkClient, session_id: String, event: &serde_json::Value) {
	if event["op"].as_str() == Some("event") || event["op"].as_str() == Some("playerUpdate") {
		trace!(
			"Lavalink raw event received for session {}: {:?}",
			session_id,
			event
		);
	}
}

/// Hook that handles the `Ready` event for a Lavalink client.
///
/// This function is triggered when the Lavalink client establishes a successful connection.
/// It logs the session ID and the `Ready` event details, and then proceeds to delete all
/// existing player contexts to ensure a clean state.
///
/// # Parameters
/// - `client`: An instance of `LavalinkClient` used to manage connections and players.
/// - `session_id`: A `String` representing the current session's unique identifier.
/// - `event`: A reference to the `Ready` event that triggered this hook.
///
/// # Behavior
/// - Logs the `session_id` and the `Ready` event details at the `info` level.
/// - Asynchronously deletes all player contexts associated with the Lavalink client.
/// - If `delete_all_player_contexts` fails, a `warn` log is issued.
#[hook]
pub async fn ready_event(client: LavalinkClient, session_id: String, event: &events::Ready) {
	info!(
		"Lavalink client is ready for session {}: {:?}",
		session_id, event
	);
	if let Err(e) = client.delete_all_player_contexts().await {
		warn!(
			"Failed to delete all player contexts for session {}: {}",
			session_id, e
		);
	}
}

/// Handles the `TrackStart` event from Lavalink.
///
/// This function is triggered when a track starts playing. It logs the track details and
/// retrieves the player context to access custom data, such as the `ChannelId` and `Http`
/// instance, which are required for sending messages.
///
/// # Arguments
///
/// * `client` - An instance of `LavalinkClient` for interacting with Lavalink.
/// * `_session_id` - The Lavalink session ID (unused).
/// * `event` - A reference to the `TrackStart` event containing details about the track.
///
/// # Behavior
///
/// - Logs the `TrackStart` event details at the `info` level.
/// - Retrieves the player context for the corresponding `guild_id`.
/// - If the context cannot be retrieved, a `warn` log is issued, and the function returns.
/// - Retrieves custom data (e.g., `ChannelId`, `Http` instance) from the player context.
/// - If custom data retrieval fails, a `warn` log is issued, and the function returns.
/// - The function then constructs a message indicating the currently playing track, including its
///   author, title, URI (if available), and the user who requested it.
///
/// # Notes
///
/// - This function is decorated with the `#[hook]` attribute, indicating it's an event hook.
/// - The `user_data` field of the track is expected to contain a "requester_id" value, which
///   represents the Discord user ID of the person who requested the track.
#[hook]
pub async fn track_start(client: LavalinkClient, _session_id: String, event: &events::TrackStart) {
	let track = &event.track;
	info!("Track started: {:?} for guild {:?}", track, event.guild_id);

	let player_context = match client.get_player_context(event.guild_id) {
		Some(context) => context,
		None => {
			warn!("Could not get player context for guild {:?}", event.guild_id);
			return;
		},
	};

	let data = match player_context.data::<(ChannelId, std::sync::Arc<Http>)>() {
		Ok(data) => data,
		Err(e) => {
			warn!(
				"Could not get player context data for guild {:?}: {}",
				event.guild_id, e
			);
			return;
		},
	};
	let (..) = (&data.0, &data.1);

	let _ = {
		let track = &event.track;
		let requester_id = track
			.user_data
			.clone()
			.and_then(|ud| ud["requester_id"].as_str().map(String::from))
			.unwrap_or_else(|| "N/A".to_string());

		if let Some(uri) = &track.info.uri {
			format!(
				"Now playing: [{} - {}](<{}>) | Requested by <@!{}>",
				track.info.author, track.info.title, uri, requester_id
			)
		} else {
			format!(
				"Now playing: {} - {} | Requested by <@!{}>",
				track.info.author, track.info.title, requester_id
			)
		}
	};
}

