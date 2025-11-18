use leptos::*;
use leptos::prelude::*;
use crate::app::{User, Guild};

#[component]
pub fn Profile(
    #[prop(into)] user: Signal<Option<User>>,
) -> impl IntoView {
    view! {
        <main class="legal-page">
            <div class="legal-container">
                {move || {
                    if let Some(user_data) = user.get() {
                        view! {
                            <div>
                                <header style="text-align: center;">
                                    <img 
                                        src={user_data.avatar_url.clone()} 
                                        alt="Profile Avatar" 
                                        style="width: 100px; height: 100px; border-radius: 50%; margin-bottom: 20px;"
                                    />
                                    <h1>"You are logged in"</h1>
                                    <p class="last-updated">"Welcome, " {user_data.username.clone()}</p>
                                </header>

                                <h2>"Your Discord Servers"</h2>
                                {if user_data.guilds.is_empty() {
                                    view! {
                                        <p>"You are not in any servers with this bot."</p>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(250px, 1fr)); gap: 20px; margin-top: 20px;">
                                            {user_data.guilds.iter().map(|guild| {
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
