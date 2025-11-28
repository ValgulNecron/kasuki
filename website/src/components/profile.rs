use leptos::*;
use leptos::prelude::*;
use crate::app::{UserSessionData};

#[component]
pub fn Profile(
    #[prop(into)] user_session_data: Signal<Option<UserSessionData>>,
) -> impl IntoView {
    view! {
        <main class="legal-page">
            <div class="legal-container">
                {move || {
                    if let Some(data) = user_session_data.get() {
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
                                                src={"https://cdn.discordapp.com/embed/avatars/0.png"} // Generic default Discord avatar
                                                alt="Default Profile Avatar"
                                                style="width: 100px; height: 100px; border-radius: 50%; margin-bottom: 20px;"
                                            />
                                        }.into_any()
                                    }}
                                    <h1>"You are logged in"</h1>
                                    <p class="last-updated">"Welcome, " {data.user.username.clone()}</p>
                                </header>

                                <h2>"Your Discord Servers"</h2>
                                {if data.guilds.is_empty() {
                                    view! {
                                        <p>"You are not in any servers with this bot."</p>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 20px; margin-top: 20px;">
                                            {data.guilds.iter().map(|guild| {
                                                let guild_clone = guild.clone();
                                                view! {
                                                    <div style="background: var(--light); padding: 20px; border-radius: var(--border-radius); display: flex; align-items: center; gap: 15px;">
                                                        {if let Some(icon_url) = &guild_clone.icon_url {
                                                            view! {
                                                                <img
                                                                    src={icon_url.clone()}
                                                                    alt="Server Icon"
                                                                    style="width: 50px; height: 50px; border-radius: 50%;"
                                                                />
                                                            }.into_any()
                                                        } else {
                                                            view! {
                                                                <div style="width: 50px; height: 50px; border-radius: 50%; background: var(--primary); display: flex; align-items: center; justify-content: center; color: white; font-weight: bold;">
                                                                    {guild_clone.name.chars().next().unwrap_or('S').to_string()}
                                                                </div>
                                                            }.into_any()
                                                        }}
                                                        <div>
                                                            <h3 style="margin: 0; font-size: 1rem;">{guild_clone.name}</h3>
                                                        </div>
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }.into_any()
                                }}
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
