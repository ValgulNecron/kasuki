
use leptos::*;
use leptos::prelude::*;
use crate::app::User;
use crate::config::Config;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

#[component]
pub fn Header(
    #[prop(into)] user: Signal<Option<User>>,
    #[prop(into)] set_user: WriteSignal<Option<User>>,
) -> impl IntoView {
    let (menu_open, set_menu_open) = create_signal(false);

    let handle_login = move |_| {
        // Redirect to the Discord OAuth login endpoint
        if let Some(window) = web_sys::window() {
            let login_url = Config::oauth_login_url();
            let _ = window.location().set_href(&login_url);
        }
    };

    let handle_logout = move |_| {
        // Call logout endpoint
        wasm_bindgen_futures::spawn_local(async move {
            let _ = call_logout().await;
            set_user.set(None);
            // Navigate to home
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_hash("");
            }
        });
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

/// Call the logout endpoint
async fn call_logout() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or(JsValue::from_str("No window"))?;
    
    let logout_url = format!("{}/api/session/logout", Config::api_url());
    
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);
    opts.credentials(web_sys::RequestCredentials::Include); // Important: include cookies
    
    let request = Request::new_with_str_and_init(&logout_url, &opts)?;
    
    let _ = JsFuture::from(window.fetch_with_request(&request)).await?;
    
    Ok(())
}