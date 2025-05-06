//! This file implements Lavalink event hooks to handle various events such as raw Lavalink events,
//! ready events, and track start events in a Discord bot using both `lavalink_rs` and `serenity` libraries.
use lavalink_rs::{hook, model::events, prelude::*};
use serenity::all::ChannelId;
use serenity::http::Http;
use tracing::{error, trace};

/// Asynchronous hook function to handle raw events from Lavalink.
///
/// This function is triggered whenever an event is received by the Lavalink client.
/// It processes events with operation types "event" or "playerUpdate", and logs the event details
/// for debugging or tracing purposes.
///
/// # Parameters
/// - `_: LavalinkClient` - The Lavalink client instance (not used directly in this function).
/// - `session_id: String` - The session ID associated with the Lavalink client.
/// - `event: &serde_json::Value` - The raw JSON representation of the event data.
///
/// # Behavior
/// - Checks the `op` field in the `event` JSON object to determine the operation type.
/// - If the `op` field is either "event" or "playerUpdate", logs the session ID and event data
///   using the `trace!` macro for debugging.
///
/// # Examples
/// ```
/// raw_event(client, "session123".to_string(), &event_json).await;
/// ```
///
/// # Notes
/// - This function is intended to be used as a hook with the Lavalink's event system.
/// - The function does not return any value, it simply logs the relevant event details.
///
/// # Logging
/// - Logs the session ID and event data at the trace level for the specified types of events.
#[hook]
pub async fn raw_event(_: LavalinkClient, session_id: String, event: &serde_json::Value) {
	if event["op"].as_str() == Some("event") || event["op"].as_str() == Some("playerUpdate") {
		trace!("{:?} -> {:?}", session_id, event);
	}
}

/// Hook that handles the `Ready` event for a Lavalink client.
///
/// This function is triggered when a `Ready` event is received by the Lavalink client.
/// It logs the session ID and the `Ready` event for debugging purposes and ensures
/// all player contexts are deleted from the client.
///
/// # Parameters
/// - `client`: An instance of `LavalinkClient` used to manage connections and players.
/// - `session_id`: A `String` representing the current session's unique identifier.
/// - `event`: A reference to the `Ready` event that triggered this hook.
///
/// # Behavior
/// - Logs the session ID and the received `Ready` event.
/// - Deletes all player contexts associated with the Lavalink client asynchronously.
///
/// # Panics
/// This function will panic if the deletion of player contexts fails (i.e., the `unwrap()` call on `delete_all_player_contexts()`).
///
/// # Example
/// ```rust
/// #[hook]
/// pub async fn ready_event(client: LavalinkClient, session_id: String, event: &events::Ready) {
///     trace!("{:?} -> {:?}", session_id, event);
///     client.delete_all_player_contexts().await.unwrap();
/// }
/// ```
#[hook]
pub async fn ready_event(client: LavalinkClient, session_id: String, event: &events::Ready) {
	trace!("{:?} -> {:?}", session_id, event);
	client.delete_all_player_contexts().await.unwrap();
}

/// Handles the `TrackStart` event in the Lavalink library.
///
/// # Arguments
///
/// * `client` - An instance of `LavalinkClient`, the primary structure needed to interact with Lavalink.
/// * `_session_id` - A string containing the Lavalink session ID. Currently unused.
/// * `event` - A reference to the `TrackStart` event containing details about the track that has started playing.
///
/// # Behavior
///
/// - Logs details of the `TrackStart` event for debugging purposes.
/// - Attempts to retrieve the player context for the provided `guild_id` from the Lavalink client.
/// - Fetches custom player data, extracting the `ChannelId` and `Http` required for message interactions.
/// - If the player data is successfully retrieved, constructs a message mentioning the currently playing track's details, including:
///   - Track author
///   - Track title
///   - URI (if available)
///   - The user who requested the track.
///
/// If the player context or associated data cannot be retrieved, logs the error and stops further processing.
///
/// # Notes
///
/// - This function is decorated with the `#[hook]` attribute, indicating it is triggered as an event hook.
/// - The `GuildId`, `ChannelId`, and other fields used are from the Serenity library.
/// - If the URI of the track is unavailable, it excludes the clickable link in the constructed message.
///
/// # Example Output Message
///
/// - With URI:
///   `Now playing: [Author - Track Title](<Track URI>) | Requested by <@!UserID>`
///
/// - Without URI:
///   `Now playing: Author - Track Title | Requested by <@!UserID>`
///
/// # Errors
///
/// If the player context data cannot be retrieved, logs the error and terminates execution without constructing the message.
///
/// # Dependencies
///
/// This function requires:
/// - `LavalinkClient` for interacting with Lavalink.
/// - `events::TrackStart` for handling track start events.
/// - An associated player context containing `ChannelId` and `Http` for message handling.
#[hook]
pub async fn track_start(client: LavalinkClient, _session_id: String, event: &events::TrackStart) {
	trace!("{:?}", event);
	let player_context = client.get_player_context(event.guild_id).unwrap();
	let data = match player_context.data::<(ChannelId, std::sync::Arc<Http>)>() {
		Ok(data) => data,
		Err(e) => {
			error!("{:#?}", e);
			return;
		},
	};
	let (..) = (&data.0, &data.1);

	let _ = {
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
