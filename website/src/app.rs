use crate::api::{exchange_auth_code, fetch_user_data};
use crate::components::activities::Activities;
use crate::components::commands::Commands;
use crate::components::features::Features;
use crate::components::footer::Footer;
use crate::components::header::Header;
use crate::components::hero::Hero;
use crate::components::privacy::Privacy;
use crate::components::profile::Profile;
use crate::components::screenshots::Screenshots;
use crate::components::server_settings::ServerSettings;
use crate::components::setup::Setup;
use crate::components::terms::Terms;
use leptos::logging::log;
use leptos::prelude::document;
use leptos::prelude::Effect;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use url::Url;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{window, HashChangeEvent};

#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub enum Page {
    Home,
    Privacy,
    Terms,
    Profile,
    ServerSettings(String),
    ServerActivities(String),
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

    // Use JWT to fetch user data and store session
    let use_jwt = move |jwt: String| {
        let local_storage = window().expect("window").local_storage().unwrap().unwrap();
        let _ = local_storage.set_item("jwt", &jwt);
        spawn_local(async move {
            match fetch_user_data(Some(jwt)).await {
                Ok((user_data, guilds)) => {
                    log!("Fetched user data: {:?}", user_data);
                    set_user_session_data.set(Some(UserSessionData { user: user_data, guilds }));
                }
                Err(e) => {
                    log!("Failed to fetch user data with JWT: {:?}", e);
                    set_user_session_data.set(None);
                    let ls = window().expect("window").local_storage().unwrap().unwrap();
                    let _ = ls.remove_item("jwt");
                }
            }
        });
    };

    // Handle auth: prioritized — OAuth code beats URL JWT beats stored JWT,
    // because a fresh code implies a new login that should override stale tokens.
    let handle_auth = move |params: &ParsedParams| {
        // 1. If we have an auth code from OAuth callback, exchange it for JWT
        if let Some(ref code) = params.code {
            let code = code.clone();
            spawn_local(async move {
                match exchange_auth_code(&code).await {
                    Ok(jwt) => {
                        // Store JWT BEFORE changing hash — the hash change re-triggers
                        // handle_auth, which would otherwise fall through to the
                        // localStorage path and find nothing.
                        let local_storage = window().expect("window").local_storage().unwrap().unwrap();
                        let _ = local_storage.set_item("jwt", &jwt);
                        use_jwt(jwt);
                        // Remove the OAuth code from the URL to prevent re-exchange on reload
                        if let Some(w) = window() {
                            let _ = w.location().set_hash("#/profile");
                        }
                    }
                    Err(e) => {
                        log!("Failed to exchange auth code: {:?}", e);
                        set_user_session_data.set(None);
                    }
                }
            });
            // Early return: code exchange is async, so skip lower-priority paths
            return;
        }

        // 2. If we have a JWT directly in the URL (e.g. passed by external redirect)
        if let Some(ref jwt) = params.jwt {
            use_jwt(jwt.clone());
            return;
        }

        // 3. Try JWT from localStorage — check expiry client-side to avoid
        //    unnecessary API calls with a token the server would reject anyway
        let local_storage = window().expect("window").local_storage().unwrap().unwrap();
        if let Ok(Some(jwt_from_storage)) = local_storage.get_item("jwt") {
            if is_jwt_expired(&jwt_from_storage) {
                log!("JWT expired, clearing session");
                let _ = local_storage.remove_item("jwt");
                set_user_session_data.set(None);
            } else {
                use_jwt(jwt_from_storage);
            }
        } else {
            set_user_session_data.set(None);
        }
    };

    // Handle initial page load based on hash
    Effect::new(move |_| {
        let full_hash = window().expect("window").location().hash().unwrap_or_default();
        let params = parse_hash_and_params(&full_hash);

        set_current_page.set(params.page.clone());
        handle_auth(&params);

        // Listen for hash changes
        let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |_event: HashChangeEvent| {
            let full_hash = window().expect("window").location().hash().unwrap_or_default();
            let params = parse_hash_and_params(&full_hash);

            set_current_page.set(params.page.clone());
            handle_auth(&params);
        }) as Box<dyn FnMut(_)>);

        let _ = window().expect("window").add_event_listener_with_callback("hashchange", closure.as_ref().unchecked_ref());
        // Leak the closure so it lives for the page lifetime — dropping it would
        // invalidate the JS callback reference since WASM has no GC integration.
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
                Page::ServerSettings(ref guild_id) => {
                    let gid = guild_id.clone();
                    view! { <main id="main-content" role="main"><ServerSettings user_session_data=user_session_data guild_id=gid /></main> }.into_any()
                },
                Page::ServerActivities(ref guild_id) => {
                    let gid = guild_id.clone();
                    view! { <main id="main-content" role="main"><Activities user_session_data=user_session_data guild_id=gid /></main> }.into_any()
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

/// Decode the JWT payload (no signature verification) to check the `exp` claim.
/// Skips signature verification because the client doesn't hold the signing key —
/// the server re-validates on every API call anyway.
fn is_jwt_expired(token: &str) -> bool {
    // JWT structure: header.payload.signature
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return true;
    }
    // Only need the payload (index 1) — header and signature are irrelevant for expiry
    let payload = parts[1];
    let decoded = match base64_url_decode(payload) {
        Some(d) => d,
        None => return true,
    };
    // Parse exp from JSON
    #[derive(serde::Deserialize)]
    struct JwtPayload {
        exp: Option<f64>,
    }
    let parsed: JwtPayload = match serde_json::from_slice(&decoded) {
        Ok(p) => p,
        Err(_) => return true,
    };
    match parsed.exp {
        Some(exp) => {
            // Date::now() returns milliseconds; JWT exp is in seconds (Unix epoch)
            let now = js_sys::Date::now() / 1000.0;
            now >= exp
        }
        None => true,
    }
}

fn base64_url_decode(input: &str) -> Option<Vec<u8>> {
    // base64url uses - and _ instead of + and / to be URL-safe (RFC 4648 sec 5)
    let mut s = input.replace('-', "+").replace('_', "/");
    // JWTs strip padding, but atob() requires it — pad to a multiple of 4
    while s.len() % 4 != 0 {
        s.push('=');
    }
    // Use the browser's built-in atob to avoid pulling in a base64 crate for WASM
    let window = web_sys::window()?;
    let decoded_str = window.atob(&s).ok()?;
    // atob returns a "binary string" where each char is one byte
    Some(decoded_str.bytes().collect())
}

struct ParsedParams {
    page: Page,
    jwt: Option<String>,
    code: Option<String>,
}

// Helper function to parse hash and query parameters
fn parse_hash_and_params(full_hash: &str) -> ParsedParams {
    let parts: Vec<&str> = full_hash.splitn(2, '?').collect();
    let hash_path = parts[0];
    let page = match hash_path {
        "#/privacy" => Page::Privacy,
        "#/terms" => Page::Terms,
        "#/profile" => Page::Profile,
        _ => {
            // Match dynamic routes: #/server/{id} and #/server/{id}/activities
            if let Some(rest) = hash_path.strip_prefix("#/server/") {
                if let Some(guild_id) = rest.strip_suffix("/activities") {
                    Page::ServerActivities(guild_id.to_string())
                } else {
                    Page::ServerSettings(rest.to_string())
                }
            } else {
                Page::Home
            }
        }
    };

    let mut jwt: Option<String> = None;
    let mut code: Option<String> = None;

    if parts.len() > 1 {
        let query_string = parts[1];
        // Url::parse needs a full URL — use a dummy base so we can leverage its query parser
        if let Ok(url) = Url::parse(&format!("http://example.com?{}", query_string)) {
            for (key, value) in url.query_pairs() {
                match key.as_ref() {
                    "jwt" => jwt = Some(value.to_string()),
                    "code" => code = Some(value.to_string()),
                    _ => {}
                }
            }
        }
    }
    ParsedParams { page, jwt, code }
}
