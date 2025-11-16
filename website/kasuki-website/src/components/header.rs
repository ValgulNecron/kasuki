
use leptos::*;
use leptos::prelude::*;
use crate::app::User;

#[component]
pub fn Header(
    #[prop(into)] user: Signal<Option<User>>,
    #[prop(into)] set_user: WriteSignal<Option<User>>,
) -> impl IntoView {
    let (menu_open, set_menu_open) = create_signal(false);

    let handle_login = move |_| {
        // For now, simulate Discord OAuth login with demo data
        // In production, this would redirect to Discord OAuth flow
        if let Some(window) = web_sys::window() {
            let _ = window.alert_with_message("Discord OAuth login would happen here.\nFor demo purposes, logging you in with test data.");
        }
        
        // Set demo user data
        set_user.set(Some(User {
            id: "123456789".to_string(),
            username: "DemoUser".to_string(),
            avatar_url: "https://cdn.discordapp.com/embed/avatars/0.png".to_string(),
            guilds: vec![
                crate::app::Guild {
                    id: "1".to_string(),
                    name: "Anime Lovers".to_string(),
                    icon_url: None,
                },
                crate::app::Guild {
                    id: "2".to_string(),
                    name: "Manga Readers".to_string(),
                    icon_url: None,
                },
                crate::app::Guild {
                    id: "3".to_string(),
                    name: "Kasuki Support".to_string(),
                    icon_url: None,
                },
            ],
        }));
    };

    let handle_logout = move |_| {
        set_user.set(None);
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
                                if let Some(user_data) = user.get() {
                                    view! {
                                        <div style="display: flex; align-items: center; gap: 10px;">
                                            <a href="#/profile" on:click=move |_| set_menu_open.set(false) style="display: flex; align-items: center; gap: 8px;">
                                                <img 
                                                    src={user_data.avatar_url.clone()} 
                                                    alt="Profile" 
                                                    style="width: 32px; height: 32px; border-radius: 50%;"
                                                />
                                                <span>{user_data.username.clone()}</span>
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