use crate::api::{fetch_admin_guilds, request_blacklist};
use crate::app::UserSessionData;
use leptos::logging::log;
use leptos::prelude::*;
use leptos::*;
use wasm_bindgen_futures::spawn_local;

fn get_jwt() -> Option<String> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item("jwt").ok().flatten())
}

#[component]
pub fn Profile(
    #[prop(into)] user_session_data: Signal<Option<UserSessionData>>,
) -> impl IntoView {
    let (blacklist_loading, set_blacklist_loading) = signal(false);
    let (blacklist_status, set_blacklist_status) = signal(None::<String>);
    let (blacklist_error, set_blacklist_error) = signal(None::<String>);
    let (admin_guild_ids, set_admin_guild_ids) = signal(Vec::<String>::new());

    // Fetch admin guilds when user is logged in
    Effect::new(move |_| {
        if user_session_data.get().is_some() {
            spawn_local(async move {
                match fetch_admin_guilds(get_jwt()).await {
                    Ok(guilds) => {
                        let ids: Vec<String> = guilds.iter().map(|g| g.id.clone()).collect();
                        set_admin_guild_ids.set(ids);
                    }
                    Err(e) => {
                        log!("Failed to fetch admin guilds: {:?}", e);
                    }
                }
            });
        }
    });

    let handle_blacklist = move |_| {
        set_blacklist_loading.set(true);
        set_blacklist_status.set(None);
        set_blacklist_error.set(None);
        spawn_local(async move {
            match request_blacklist(get_jwt()).await {
                Ok(_) => {
                    set_blacklist_status.set(Some("Blacklist request sent successfully.".to_string()));
                    set_blacklist_loading.set(false);
                }
                Err(e) => {
                    log!("Blacklist request failed: {:?}", e);
                    set_blacklist_error.set(Some("Failed to send blacklist request.".to_string()));
                    set_blacklist_loading.set(false);
                }
            }
        });
    };

    view! {
        <main class="legal-page">
            <div class="legal-container">
                {move || {
                    if let Some(data) = user_session_data.get() {
                        let admin_ids = admin_guild_ids.get();

                        // Sort: admin guilds first
                        let mut manageable = Vec::new();
                        let mut other = Vec::new();
                        for guild in &data.guilds {
                            if admin_ids.contains(&guild.id) {
                                manageable.push(guild.clone());
                            } else {
                                other.push(guild.clone());
                            }
                        }

                        view! {
                            <div>
                                <header style="text-align: center;">
                                    {if let Some(avatar_hash) = &data.user.avatar {
                                        view! {
                                            <img
                                                src={format!("https://cdn.discordapp.com/avatars/{}/{}.png", data.user.id, avatar_hash)}
                                                alt="Profile Avatar"
                                                style="width: 100px; height: 100px; border-radius: 50%; margin-bottom: 20px;"
                                            />
                                        }.into_any()
                                    } else {
                                        view! {
                                            <img
                                                src={"https://cdn.discordapp.com/embed/avatars/0.png"}
                                                alt="Default Profile Avatar"
                                                style="width: 100px; height: 100px; border-radius: 50%; margin-bottom: 20px;"
                                            />
                                        }.into_any()
                                    }}
                                    <h1>"You are logged in"</h1>
                                    <p class="last-updated">"Welcome, " {data.user.username.clone()}</p>
                                </header>

                                <h2>"Your Discord Servers"</h2>
                                <div class="guild-list">
                                    {manageable.iter().map(|guild| {
                                        let href = format!("#/server/{}", guild.id);
                                        view! {
                                            <a href={href} class="guild-card guild-card-manageable">
                                                {if let Some(ref icon_url) = guild.icon_url {
                                                    view! {
                                                        <img src={icon_url.clone()} alt="Server Icon" class="guild-card-icon" />
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <div class="guild-card-icon-placeholder">
                                                            {guild.name.chars().next().unwrap_or('S').to_string()}
                                                        </div>
                                                    }.into_any()
                                                }}
                                                <div class="guild-card-info">
                                                    <h3>{guild.name.clone()}</h3>
                                                    <span class="guild-card-badge">"Manage"</span>
                                                </div>
                                            </a>
                                        }
                                    }).collect::<Vec<_>>()}

                                    {other.iter().map(|guild| {
                                        view! {
                                            <div class="guild-card guild-card-inactive">
                                                {if let Some(ref icon_url) = guild.icon_url {
                                                    view! {
                                                        <img src={icon_url.clone()} alt="Server Icon" class="guild-card-icon guild-card-icon-gray" />
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <div class="guild-card-icon-placeholder guild-card-icon-gray">
                                                            {guild.name.chars().next().unwrap_or('S').to_string()}
                                                        </div>
                                                    }.into_any()
                                                }}
                                                <div class="guild-card-info">
                                                    <h3>{guild.name.clone()}</h3>
                                                </div>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>

                                <h2 style="margin-top: 30px;">"Blacklist"</h2>
                                <p style="margin-bottom: 15px;">"Request to be added to the bot's blacklist. This will prevent the bot from processing your commands."</p>
                                {move || blacklist_status.get().map(|s| view! { <div class="settings-success">{s}</div> })}
                                {move || blacklist_error.get().map(|e| view! { <div class="settings-error">{e}</div> })}
                                <button
                                    class="blacklist-btn"
                                    on:click=handle_blacklist
                                    disabled=move || blacklist_loading.get()
                                >
                                    {move || if blacklist_loading.get() { "Sending..." } else { "Request Blacklist Addition" }}
                                </button>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div>
                                <header>
                                    <h1>"Not Logged In"</h1>
                                    <p class="last-updated">"Please log in to view your profile."</p>
                                </header>
                                <p>"Click the Login button in the header to connect with Discord."</p>
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </main>
    }
}
