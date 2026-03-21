use leptos::prelude::*;
use leptos::logging::log;
use wasm_bindgen_futures::spawn_local;

use crate::api::{fetch_guild_activities, add_activity, delete_activity, search_anilist};
use crate::app::UserSessionData;

#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
pub struct ActivityInfo {
    pub anime_id: i32,
    pub name: String,
    pub episode: i32,
    pub delay: i32,
    pub image: String,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct AnilistSearchResult {
    pub id: i32,
    pub title: Option<AnilistTitle>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct AnilistTitle {
    pub english: Option<String>,
    pub romaji: Option<String>,
}

fn get_jwt() -> Option<String> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item("jwt").ok().flatten())
}

#[component]
pub fn Activities(
    #[prop(into)] user_session_data: Signal<Option<UserSessionData>>,
    guild_id: String,
) -> impl IntoView {
    let (activities, set_activities) = signal(Vec::<ActivityInfo>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (success, set_success) = signal(None::<String>);

    let (search_query, set_search_query) = signal(String::new());
    let (search_results, set_search_results) = signal(Vec::<AnilistSearchResult>::new());
    let (selected_anime_id, set_selected_anime_id) = signal(None::<i32>);
    let (selected_anime_name, set_selected_anime_name) = signal(String::new());
    let (channel_id, set_channel_id) = signal(String::new());
    let (delay_input, set_delay_input) = signal(String::new());
    let (adding, set_adding) = signal(false);

    let guild_id_load = guild_id.clone();
    Effect::new(move |_| {
        if user_session_data.get().is_some() {
            let gid = guild_id_load.clone();
            set_loading.set(true);
            spawn_local(async move {
                match fetch_guild_activities(get_jwt(), &gid).await {
                    Ok(a) => {
                        set_activities.set(a);
                        set_loading.set(false);
                    }
                    Err(e) => {
                        log!("Failed to fetch activities: {:?}", e);
                        set_error.set(Some("Failed to load activities.".to_string()));
                        set_loading.set(false);
                    }
                }
            });
        }
    });

    let guild_id_for_back = guild_id.clone();
    let guild_id_for_add = guild_id.clone();
    let guild_id_for_delete = guild_id.clone();

    view! {
        <main class="legal-page">
            <div class="legal-container">
                <header style="text-align: center;">
                    <h1>"Anime Activities"</h1>
                    <p class="last-updated">
                        <a href={format!("#/server/{}", guild_id_for_back)} style="color: var(--secondary);">"Back to Settings"</a>
                        " | "
                        <a href="#/profile" style="color: var(--secondary);">"Profile"</a>
                    </p>
                </header>

                {move || {
                    if user_session_data.get().is_none() {
                        return view! { <p>"Please log in."</p> }.into_any();
                    }
                    if loading.get() {
                        return view! { <p>"Loading activities..."</p> }.into_any();
                    }

                    let gid_add = guild_id_for_add.clone();
                    let gid_del = guild_id_for_delete.clone();

                    view! {
                        <div>
                            {move || error.get().map(|e| view! { <div class="settings-error">{e}</div> })}
                            {move || success.get().map(|s| view! { <div class="settings-success">{s}</div> })}

                            <h2>"Add Activity"</h2>
                            <div class="activity-add-form">
                                <div class="settings-form-row">
                                    <input
                                        type="text"
                                        class="settings-input"
                                        placeholder="Search anime..."
                                        prop:value=move || search_query.get()
                                        on:input=move |ev| {
                                            let target: web_sys::HtmlInputElement = leptos::prelude::event_target(&ev);
                                            set_search_query.set(target.value());
                                        }
                                    />
                                    <button class="settings-btn" on:click=move |_| {
                                        let q = search_query.get_untracked();
                                        if q.is_empty() { return; }
                                        spawn_local(async move {
                                            match search_anilist(get_jwt(), &q).await {
                                                Ok(results) => set_search_results.set(results),
                                                Err(e) => log!("AniList search failed: {:?}", e),
                                            }
                                        });
                                    }>"Search"</button>
                                </div>

                                {move || {
                                    let results = search_results.get();
                                    if results.is_empty() {
                                        return view! { <div></div> }.into_any();
                                    }
                                    view! {
                                        <div class="activity-search-results">
                                            {results.iter().map(|r| {
                                                let id = r.id;
                                                let name = r.title.as_ref().map(|t| {
                                                    t.english.as_deref().or(t.romaji.as_deref()).unwrap_or("Unknown")
                                                }).unwrap_or("Unknown").to_string();
                                                let name_clone = name.clone();
                                                let is_selected = move || selected_anime_id.get() == Some(id);
                                                view! {
                                                    <button
                                                        class="activity-search-item"
                                                        style=move || if is_selected() { "background: var(--primary); color: white;" } else { "" }
                                                        on:click=move |_| {
                                                            set_selected_anime_id.set(Some(id));
                                                            set_selected_anime_name.set(name_clone.clone());
                                                        }
                                                    >
                                                        {name} " (ID: " {id} ")"
                                                    </button>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }.into_any()
                                }}

                                {move || {
                                    let name = selected_anime_name.get();
                                    if name.is_empty() {
                                        return view! { <div></div> }.into_any();
                                    }
                                    view! {
                                        <p style="margin: 10px 0;">"Selected: " <strong>{name}</strong></p>
                                    }.into_any()
                                }}

                                <div class="settings-form-row" style="margin-top: 10px;">
                                    <input
                                        type="text"
                                        class="settings-input"
                                        placeholder="Channel ID (right-click channel in Discord, Copy Channel ID)"
                                        prop:value=move || channel_id.get()
                                        on:input=move |ev| {
                                            let target: web_sys::HtmlInputElement = leptos::prelude::event_target(&ev);
                                            set_channel_id.set(target.value());
                                        }
                                    />
                                </div>
                                <div class="settings-form-row" style="margin-top: 10px;">
                                    <input
                                        type="number"
                                        class="settings-input"
                                        placeholder="Delay in seconds (optional)"
                                        prop:value=move || delay_input.get()
                                        on:input=move |ev| {
                                            let target: web_sys::HtmlInputElement = leptos::prelude::event_target(&ev);
                                            set_delay_input.set(target.value());
                                        }
                                    />
                                    <button
                                        class="settings-btn"
                                        on:click={
                                            let gid = gid_add;
                                            move |_| {
                                                let anime_id = selected_anime_id.get_untracked();
                                                let cid = channel_id.get_untracked();
                                                if anime_id.is_none() || cid.is_empty() {
                                                    set_error.set(Some("Select an anime and enter a channel ID.".to_string()));
                                                    return;
                                                }
                                                let delay: i32 = delay_input.get_untracked().parse().unwrap_or(0);
                                                let gid = gid.clone();
                                                let gid2 = gid.clone();
                                                set_adding.set(true);
                                                set_error.set(None);
                                                set_success.set(None);
                                                spawn_local(async move {
                                                    let body = serde_json::json!({
                                                        "anime_id": anime_id,
                                                        "channel_id": cid,
                                                        "delay": delay,
                                                    });
                                                    match add_activity(get_jwt(), &gid, body.to_string()).await {
                                                        Ok(_) => {
                                                            set_success.set(Some("Activity added.".to_string()));
                                                            set_adding.set(false);
                                                            set_selected_anime_id.set(None);
                                                            set_selected_anime_name.set(String::new());
                                                            set_channel_id.set(String::new());
                                                            set_delay_input.set(String::new());
                                                            set_search_results.set(Vec::new());
                                                            set_search_query.set(String::new());
                                                            if let Ok(a) = fetch_guild_activities(get_jwt(), &gid2).await {
                                                                set_activities.set(a);
                                                            }
                                                        }
                                                        Err(e) => {
                                                            log!("Failed to add activity: {:?}", e);
                                                            set_error.set(Some("Failed to add activity.".to_string()));
                                                            set_adding.set(false);
                                                        }
                                                    }
                                                });
                                            }
                                        }
                                        disabled=move || adding.get()
                                    >
                                        {move || if adding.get() { "Adding..." } else { "Add Activity" }}
                                    </button>
                                </div>
                            </div>

                            <h2 style="margin-top: 30px;">"Current Activities"</h2>
                            {move || {
                                let acts = activities.get();
                                if acts.is_empty() {
                                    return view! { <p>"No activities configured."</p> }.into_any();
                                }
                                let gid = gid_del.clone();
                                view! {
                                    <div class="activity-list">
                                        {acts.iter().map(|a| {
                                            let anime_id = a.anime_id;
                                            let name = a.name.clone();
                                            let episode = a.episode;
                                            let delay = a.delay;
                                            let gid_inner = gid.clone();
                                            let gid_reload = gid.clone();
                                            view! {
                                                <div class="activity-card">
                                                    <div class="activity-card-info">
                                                        <strong>{name}</strong>
                                                        <span>" | Episode: " {episode} " | Delay: " {delay} "s"</span>
                                                    </div>
                                                    <button
                                                        class="activity-delete-btn"
                                                        on:click=move |_| {
                                                            let gid_c = gid_inner.clone();
                                                            let gid_r = gid_reload.clone();
                                                            spawn_local(async move {
                                                                match delete_activity(get_jwt(), &gid_c, anime_id).await {
                                                                    Ok(_) => {
                                                                        set_success.set(Some("Activity deleted.".to_string()));
                                                                        if let Ok(a) = fetch_guild_activities(get_jwt(), &gid_r).await {
                                                                            set_activities.set(a);
                                                                        }
                                                                    }
                                                                    Err(e) => {
                                                                        log!("Failed to delete activity: {:?}", e);
                                                                        set_error.set(Some("Failed to delete activity.".to_string()));
                                                                    }
                                                                }
                                                            });
                                                        }
                                                    >
                                                        "Delete"
                                                    </button>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            }}
                        </div>
                    }.into_any()
                }}
            </div>
        </main>
    }
}
