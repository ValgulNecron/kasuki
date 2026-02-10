use crate::api::fetch_user_data;
use crate::components::commands::Commands;
use crate::components::features::Features;
use crate::components::footer::Footer;
use crate::components::header::Header;
use crate::components::hero::Hero;
use crate::components::privacy::Privacy;
use crate::components::profile::Profile;
use crate::components::screenshots::Screenshots;
use crate::components::setup::Setup;
use crate::components::terms::Terms;
use leptos::logging::log;
use leptos::prelude::document;
use leptos::prelude::Effect;
use leptos::prelude::*;
// Corrected import
use serde::{Deserialize, Serialize};
// Added window and Storage
use url::Url;
// Corrected import
use wasm_bindgen::JsCast;
// Import the new API function
use wasm_bindgen_futures::spawn_local;
use web_sys::{window, HashChangeEvent};
// Added

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Page {
    Home,
    Privacy,
    Terms,
    Profile,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub avatar: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Guild {
    pub id: String,
    pub name: String,
    pub icon_url: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UserSessionData {
    pub user: User,
    pub guilds: Vec<Guild>,
}

#[component]
pub fn App() -> impl IntoView {
    let (is_dark, set_is_dark) = signal(false);
    let (current_page, set_current_page) = signal(Page::Home);
    let (user_session_data, set_user_session_data) = signal(None::<UserSessionData>);

    // Function to handle JWT and user state
    let handle_jwt_and_user_state = move |jwt_option: Option<String>| {
        let local_storage = window().expect("window to be available").local_storage().unwrap().unwrap();
        let set_user_session_data_clone = set_user_session_data;

        if let Some(jwt) = jwt_option {
            // Store JWT in localStorage
            let _ = local_storage.set_item("jwt", &jwt);

            // Fetch user data using JWT
            spawn_local(async move {
                match fetch_user_data(Some(jwt)).await {
                    Ok((user_data, guilds)) => {
                        log!("Fetched user data: {:?}", user_data);
                        set_user_session_data_clone.set(Some(UserSessionData { user: user_data, guilds }));
                    }
                    Err(e) => {
                        log!("Failed to fetch user data with JWT: {:?}", e);
                        set_user_session_data_clone.set(None); // Clear user on error
                        let _ = local_storage.remove_item("jwt"); // Clear invalid JWT
                    }
                }
            });
        } else {
            // No JWT from URL, check localStorage
            if let Ok(Some(jwt_from_storage)) = local_storage.get_item("jwt") {
                spawn_local(async move {
                    match fetch_user_data(Some(jwt_from_storage)).await {
                        Ok((user_data, guilds)) => {
                            log!("Fetched user data from stored JWT: {:?}", user_data);
                            set_user_session_data_clone.set(Some(UserSessionData { user: user_data, guilds }));
                        }
                        Err(e) => {
                            log!("Failed to fetch user data with stored JWT: {:?}", e);
                            set_user_session_data_clone.set(None); // Clear user on error
                            let _ = local_storage.remove_item("jwt"); // Clear invalid JWT
                        }
                    }
                });
            } else {
                set_user_session_data.set(None); // No JWT, not logged in
            }
        }
    };

    // Handle initial page load based on hash
    Effect::new(move |_| {
        let full_hash = window().expect("window to be available").location().hash().unwrap_or_default();
        let (page, jwt_from_url) = parse_hash_and_params(&full_hash);

        set_current_page.set(page);
        handle_jwt_and_user_state(jwt_from_url);

        // Listen for hash changes
        let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |_event: HashChangeEvent| {
            let full_hash = window().expect("window to be available").location().hash().unwrap_or_default();
            let (page, jwt_from_url) = parse_hash_and_params(&full_hash);

            set_current_page.set(page);
            handle_jwt_and_user_state(jwt_from_url);
        }) as Box<dyn FnMut(_)>);

        let _ = window().expect("window to be available").add_event_listener_with_callback("hashchange", closure.as_ref().unchecked_ref());
        closure.forget();
    });

    // Load theme from local storage on startup
    Effect::new(move |_| {
        if let Some(storage) = window().and_then(|w| w.local_storage().ok().flatten()) {
            if let Ok(Some(value)) = storage.get_item("theme") {
                set_is_dark.set(value == "dark");
            }
        }
    });

    // Effect to update body attribute and save to local storage
    Effect::new(move |_| {
        if let Some(body) = document().body() {
            let is_dark_now = is_dark.get();
            if is_dark_now {
                let _ = body.set_attribute("data-theme", "dark");
            } else {
                let _ = body.remove_attribute("data-theme");
            }

            if let Some(storage) = window().and_then(|w| w.local_storage().ok().flatten()) {
                let _ = storage.set_item("theme", if is_dark_now { "dark" } else { "light" });
            }
        }
    });

    view! {
        <div id="app">
            <a href="#main-content" class="skip-link">"Skip to main content"</a>
            <Header
                user_session_data=user_session_data
                set_user_session_data=set_user_session_data
            />
            {move || match current_page.get() {
                Page::Home => view! {
                    <main id="main-content" role="main">
                        <Hero />
                        <Features />
                        <Commands />
                        <Screenshots />
                        <Setup />
                    </main>
                }.into_any(),
                Page::Privacy => view! { <main id="main-content" role="main"><Privacy /></main> }.into_any(),
                Page::Terms => view! { <main id="main-content" role="main"><Terms /></main> }.into_any(),
                Page::Profile => {
                    view! { <main id="main-content" role="main"><Profile user_session_data=user_session_data /></main> }.into_any()
                },
            }}
            <Footer />
            <button 
                class="theme-toggle" 
                on:click=move |_| set_is_dark.update(|val| *val = !*val)
                aria-label={move || if is_dark.get() { "Switch to light mode" } else { "Switch to dark mode" }}
                title={move || if is_dark.get() { "Switch to light mode" } else { "Switch to dark mode" }}
            >
                <i class={move || if is_dark.get() { "fas fa-sun" } else { "fas fa-moon" }} aria-hidden="true"></i>
            </button>
        </div>
    }
}

// Helper function to parse hash and query parameters
fn parse_hash_and_params(full_hash: &str) -> (Page, Option<String>) {
    let parts: Vec<&str> = full_hash.splitn(2, '?').collect();
    let hash_path = parts[0];
    let page = match hash_path {
        "#/privacy" => Page::Privacy,
        "#/terms" => Page::Terms,
        "#/profile" => Page::Profile,
        _ => Page::Home,
    };

    let mut jwt: Option<String> = None;

    if parts.len() > 1 {
        let query_string = parts[1];
        if let Ok(url) = Url::parse(&format!("http://example.com?{}", query_string)) {
            for (key, value) in url.query_pairs() {
                if key == "jwt" {
                    jwt = Some(value.to_string());
                }
            }
        }
    }
    (page, jwt)
}
