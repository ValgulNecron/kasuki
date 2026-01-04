use crate::app::UserSessionData;
use crate::config::Config;
use leptos::prelude::*;

#[component]
pub fn Header(
    #[prop(into)] user_session_data: Signal<Option<UserSessionData>>,
    #[prop(into)] set_user_session_data: WriteSignal<Option<UserSessionData>>,
) -> impl IntoView {
    let (menu_open, set_menu_open) = signal(false);

    let handle_login = move |_| {
        // Redirect to the Discord OAuth login endpoint
        if let Some(window) = web_sys::window() {
            let login_url = Config::oauth_login_url();
            let _ = window.location().set_href(&login_url);
        }
    };

    let handle_logout = move |_| {
        set_user_session_data.set(None);
        // Navigate to home
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_hash("");
        }
    };

    view! {
        <header>
            <div class="container">
                <nav class="navbar">
                    <a href="index.html" class="logo">
                        <img src="/public/assets/icon.png" alt="Kasuki Logo"/>
                        <h1>"Kasuki"</h1>
                    </a>
                    <ul class="nav-links" class:active=move || menu_open.get()>
                        <li><a href="#features" on:click=move |_| set_menu_open.set(false)>"Features"</a></li>
                        <li><a href="#commands" on:click=move |_| set_menu_open.set(false)>"Commands"</a></li>
                        <li><a href="#screenshots" on:click=move |_| set_menu_open.set(false)>"Screenshots"</a></li>
                        <li><a href="#setup" on:click=move |_| set_menu_open.set(false)>"Setup"</a></li>
                        <li><a class="add-btn" href="https://github.com/ValgulNecron/kasuki" target="_blank" rel="noopener noreferrer">
                            <i class="fab fa-github"></i>" GitHub"
                        </a></li>
                        <li>
                            {move || {
                                if let Some(user_data) = user_session_data.get() {
                                    view! {
                                        <div style="display: flex; align-items: center; gap: 10px;">
                                            <a href="#/profile" on:click=move |_| set_menu_open.set(false) style="display: flex; align-items: center; gap: 8px;">
                                                {if let Some(avatar_hash) = &user_data.user.avatar {
                                                    view! {
                                                        <img 
                                                            src={format!("https://cdn.discordapp.com/avatars/{}/{}.png", user_data.user.id, avatar_hash)} 
                                                            alt="Profile" 
                                                            style="width: 32px; height: 32px; border-radius: 50%;"
                                                        />
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <img 
                                                            src={"https://cdn.discordapp.com/embed/avatars/0.png"} // Generic default Discord avatar
                                                            alt="Default Profile" 
                                                            style="width: 32px; height: 32px; border-radius: 50%;"
                                                        />
                                                    }.into_any()
                                                }}
                                                <span>{user_data.user.username.clone()}</span>
                                            </a>
                                            <button 
                                                on:click=handle_logout 
                                                style="background: none; border: 1px solid var(--primary); color: var(--primary); padding: 5px 10px; border-radius: 5px; cursor: pointer; font-size: 0.9rem;"
                                            >
                                                "Logout"
                                            </button>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <button class="add-btn" on:click=handle_login style="cursor: pointer;">
                                            <i class="fab fa-discord"></i>" Login with Discord"
                                        </button>
                                    }.into_any()
                                }
                            }}
                        </li>
                    </ul>
                    <button class="menu-toggle" on:click=move |_| set_menu_open.update(|val| *val = !*val) aria-label="Toggle navigation menu">
                        <span></span>
                        <span></span>
                        <span></span>
                    </button>
                </nav>
            </div>
        </header>
    }
}