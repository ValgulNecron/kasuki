use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use dashmap::DashMap;
use serenity::all::{GuildId, UserId};
use serenity::async_trait;
use songbird::input::Compose;
use songbird::input::YoutubeDl;
use songbird::tracks::Track;
use songbird::{
    model::payload::{ClientDisconnect, Speaking},
    Event, EventContext,
};
use songbird::{Call, EventHandler};
use songbird::TrackEvent;
use tokio::sync::Mutex;
use tracing::{debug, error};

#[derive(Clone)]
pub struct Receiver {
    inner: Arc<InnerReceiver>,
}
struct InnerReceiver {
    last_tick_was_empty: AtomicBool,
    known_ssrcs: DashMap<u32, UserId>,
}
impl Receiver {
    pub fn new() -> Self {
        // You can manage state here, such as a buffer of audio packet bytes so
        // you can later store them in intervals.
        Self {
            inner: Arc::new(InnerReceiver {
                last_tick_was_empty: AtomicBool::default(),
                known_ssrcs: DashMap::new(),
            }),
        }
    }
}
#[async_trait]
impl EventHandler for Receiver {
    #[allow(unused_variables)]
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        use EventContext as Ctx;
        match ctx {
            Ctx::SpeakingStateUpdate(Speaking {
                speaking,
                ssrc,
                user_id,
                ..
            }) => {
                // Discord voice calls use RTP, where every sender uses a randomly allocated
                // *Synchronisation Source* (SSRC) to allow receivers to tell which audio
                // stream a received packet belongs to. As this number is not derived from
                // the sender's user_id, only Discord Voice Gateway messages like this one
                // inform us about which random SSRC a user has been allocated. Future voice
                // packets will contain *only* the SSRC.
                //
                // You can implement logic here so that you can differentiate users'
                // SSRCs and map the SSRC to the User ID and maintain this state.
                // Using this map, you can map the `ssrc` in `voice_packet`
                // to the user ID and handle their audio packets separately.
                debug!(
                    "Speaking state update: user {:?} has SSRC {:?}, using {:?}",
                    user_id, ssrc, speaking,
                );

                if let Some(user) = user_id {
                    self.inner.known_ssrcs.insert(*ssrc, UserId::from(user.0));
                }
            }
            Ctx::VoiceTick(tick) => {
                let speaking = tick.speaking.len();
                let total_participants = speaking + tick.silent.len();
                let last_tick_was_empty = self.inner.last_tick_was_empty.load(Ordering::SeqCst);

                if speaking == 0 && !last_tick_was_empty {
                    self.inner.last_tick_was_empty.store(true, Ordering::SeqCst);
                } else if speaking != 0 {
                    self.inner
                        .last_tick_was_empty
                        .store(false, Ordering::SeqCst);
                }
            }
            Ctx::ClientDisconnect(ClientDisconnect { user_id, .. }) => {
                // You can implement your own logic here to handle a user who has left the
                // voice channel e.g., finalise processing of statistics etc.
                // You will typically need to map the User ID to their SSRC; observed when
                // first speaking.

                debug!("Client disconnected: user {:?}", user_id);
            }
            Ctx::Track(track_list) => {
                for (state, handle) in *track_list {
                    error!(
                        "Track {:?} encountered an error: {:?}",
                        handle.uuid(),
                        state.playing
                    );
                }
            }
            _ => {}
        }

        None
    }
}

pub struct TrackErrorNotifier;
#[async_trait]
impl EventHandler for TrackErrorNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            for (state, handle) in *track_list {
                error!(
                    "Track {:?} encountered an error: {:?}",
                    handle.uuid(),
                    state.playing
                );
            }
        }
        None
    }
}

pub struct TrackEndNotifier {
    pub manager: Arc<Mutex<Call>>,
    pub url: String,
    pub guild_id: GuildId,
}
#[async_trait]
impl EventHandler for TrackEndNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        let handler_mutex = self.manager.clone();
        let url = self.url.clone();
        let url = self.url.clone();
        if let EventContext::Track(track_list) = ctx {
            let handler_mutex_clone = handler_mutex.clone();
            let mut handler_lock = handler_mutex_clone.lock().await;
            for (state, handle) in *track_list {
                debug!("Track {:?} ended", handle.uuid());
                let http_client = reqwest::Client::new();
                let mut src = YoutubeDl::new(http_client, url.clone());
                let (track, meta) = futures::join!(
                    handler_lock.enqueue(Track::from(src.clone())),
                    src.aux_metadata()
                );
                let url = match meta {
                    Ok(meta) => meta.source_url.unwrap_or(url),
                    Err(_) => url,
                };
                handler_lock.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);
                handler_lock.add_global_event(
                    TrackEvent::End.into(),
                    TrackEndNotifier {
                        manager: handler_mutex,
                        url,
                        guild_id: self.guild_id,
                    },
                );
                break;
            }
        }
        None
    }
}
