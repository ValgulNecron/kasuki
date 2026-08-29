use leptos::prelude::*;
use leptos::logging::log;
use wasm_bindgen_futures::spawn_local;

use crate::api::fetch_admin_guilds;
use crate::app::UserSessionData;

#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
pub struct AdminGuild {
    pub id: String,
    pub name: String,
    pub icon_url: Option<String>,
}

#[component]
pub fn Dashboard(
    #[prop(into)] user_session_data: Signal<Option<UserSessionData>>,
) -> impl IntoView {
    let (guilds, set_guilds) = signal(None::<Vec<AdminGuild>>);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    Effect::new(move |_| {
        if user_session_data.get().is_some() {
            set_loading.set(true);
            spawn_local(async move {
                let jwt = web_sys::window()
                    .and_then(|w| w.local_storage().ok().flatten())
                    .and_then(|s| s.get_item("jwt").ok().flatten());
                match fetch_admin_guilds(jwt).await {
                    Ok(g) => {
                        set_guilds.set(Some(g));
                        set_loading.set(false);
                    }
                    Err(e) => {
                        log!("Failed to fetch admin guilds: {:?}", e);
                        set_error.set(Some("Failed to load your servers.".to_string()));
                        set_loading.set(false);
                    }
                }
            });
        }
    });

    view! {
        <main class="legal-page">
            <div class="legal-container">
                <header style="text-align: center;">
                    <h1>"Server Dashboard"</h1>
                    <p class="last-updated">"Manage servers where you are an admin and the bot is present."</p>
                </header>

                {move || {
                    if user_session_data.get().is_none() {
                        return view! {
                            <p>"Please log in to view your dashboard."</p>
                        }.into_any();
                    }

                    if loading.get() {
                        return view! {
                            <p class="dashboard-loading">"Loading your servers..."</p>
                        }.into_any();
                    }

                    if let Some(err) = error.get() {
                        return view! {
                            <p class="dashboard-error">{err}</p>
                        }.into_any();
                    }

                    match guilds.get() {
                        Some(guild_list) if !guild_list.is_empty() => {
                            view! {
                                <div class="dashboard-grid">
                                    {guild_list.iter().map(|guild| {
                                        let guild_id = guild.id.clone();
                                        let href = format!("#/server/{}", guild_id);
                                        view! {
                                            <a href={href} class="dashboard-card">
                                                {if let Some(ref icon_url) = guild.icon_url {
                                                    view! {
                                                        <img src={icon_url.clone()} alt="Server Icon" class="dashboard-card-icon" />
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <div class="dashboard-card-icon-placeholder">
                                                            {guild.name.chars().next().unwrap_or('S').to_string()}
                                                        </div>
                                                    }.into_any()
                                                }}
                                                <span class="dashboard-card-name">{guild.name.clone()}</span>
                                            </a>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        }
                        _ => {
                            view! {
                                <p>"No servers found where you are an admin and the bot is present."</p>
                            }.into_any()
                        }
                    }
                }}
            </div>
        </main>
    }
}
