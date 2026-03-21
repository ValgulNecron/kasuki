use leptos::prelude::*;
use leptos::logging::log;
use wasm_bindgen_futures::spawn_local;

use crate::api::{fetch_guild_settings, fetch_langs, update_guild_lang, update_guild_modules, LangOption};
use crate::app::UserSessionData;

#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
pub struct GuildSettings {
    pub lang: Option<String>,
    pub modules: ModuleSettings,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
pub struct ModuleSettings {
    pub ai_module: bool,
    pub anilist_module: bool,
    pub game_module: bool,
    pub anime_module: bool,
    pub vn_module: bool,
    pub level_module: bool,
    pub mini_game_module: bool,
}

fn get_jwt() -> Option<String> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item("jwt").ok().flatten())
}

#[component]
pub fn ServerSettings(
    #[prop(into)] user_session_data: Signal<Option<UserSessionData>>,
    guild_id: String,
) -> impl IntoView {
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (success, set_success) = signal(None::<String>);
    let (lang, set_lang) = signal("en".to_string());
    let (langs, set_langs) = signal(Vec::<LangOption>::new());
    let (ai_module, set_ai_module) = signal(true);
    let (anilist_module, set_anilist_module) = signal(true);
    let (game_module, set_game_module) = signal(true);
    let (anime_module, set_anime_module) = signal(true);
    let (vn_module, set_vn_module) = signal(true);
    let (level_module, set_level_module) = signal(false);
    let (mini_game_module, set_mini_game_module) = signal(true);
    let (saving, set_saving) = signal(false);

    let guild_id_clone = guild_id.clone();
    Effect::new(move |_| {
        if user_session_data.get().is_some() {
            let gid = guild_id_clone.clone();
            set_loading.set(true);
            spawn_local(async move {
                // Fetch langs and settings in parallel
                let langs_fut = fetch_langs(get_jwt());
                let settings_fut = fetch_guild_settings(get_jwt(), &gid);

                if let Ok(lang_list) = langs_fut.await {
                    set_langs.set(lang_list);
                }

                match settings_fut.await {
                    Ok(settings) => {
                        if let Some(l) = settings.lang {
                            set_lang.set(l);
                        }
                        set_ai_module.set(settings.modules.ai_module);
                        set_anilist_module.set(settings.modules.anilist_module);
                        set_game_module.set(settings.modules.game_module);
                        set_anime_module.set(settings.modules.anime_module);
                        set_vn_module.set(settings.modules.vn_module);
                        set_level_module.set(settings.modules.level_module);
                        set_mini_game_module.set(settings.modules.mini_game_module);
                        set_loading.set(false);
                    }
                    Err(e) => {
                        log!("Failed to fetch guild settings: {:?}", e);
                        set_error.set(Some("Failed to load server settings.".to_string()));
                        set_loading.set(false);
                    }
                }
            });
        }
    });

    let guild_id_for_lang = guild_id.clone();
    let guild_id_for_modules = guild_id.clone();
    let guild_id_for_activities = guild_id.clone();

    view! {
        <main class="legal-page">
            <div class="legal-container">
                <header style="text-align: center;">
                    <h1>"Server Settings"</h1>
                    <p class="last-updated">
                        <a href="#/profile" style="color: var(--secondary);">"Back to Profile"</a>
                        " | "
                        <a href={format!("#/server/{}/activities", guild_id_for_activities)} style="color: var(--secondary);">"Manage Activities"</a>
                    </p>
                </header>

                {move || {
                    if user_session_data.get().is_none() {
                        return view! { <p>"Please log in."</p> }.into_any();
                    }
                    if loading.get() {
                        return view! { <p>"Loading settings..."</p> }.into_any();
                    }

                    let gid_lang = guild_id_for_lang.clone();
                    let gid_modules = guild_id_for_modules.clone();

                    view! {
                        <div>
                            {move || error.get().map(|e| view! { <div class="settings-error">{e}</div> })}
                            {move || success.get().map(|s| view! { <div class="settings-success">{s}</div> })}

                            <h2>"Language"</h2>
                            <div class="settings-form-row">
                                <select
                                    class="settings-select"
                                    on:change=move |ev| {
                                        let target: web_sys::HtmlSelectElement = leptos::prelude::event_target(&ev);
                                        set_lang.set(target.value());
                                    }
                                >
                                    {move || langs.get().iter().map(|lo| {
                                        let code = lo.code.clone();
                                        let code_for_selected = lo.code.clone();
                                        let name = lo.name.clone();
                                        view! {
                                            <option value={code} selected=move || lang.get() == code_for_selected>
                                                {name}
                                            </option>
                                        }
                                    }).collect::<Vec<_>>()}
                                </select>
                                <button
                                    class="settings-btn"
                                    on:click={
                                        let gid = gid_lang;
                                        move |_| {
                                            let gid = gid.clone();
                                            let current_lang = lang.get_untracked();
                                            set_saving.set(true);
                                            set_error.set(None);
                                            set_success.set(None);
                                            spawn_local(async move {
                                                match update_guild_lang(get_jwt(), &gid, &current_lang).await {
                                                    Ok(_) => {
                                                        set_success.set(Some("Language updated.".to_string()));
                                                        set_saving.set(false);
                                                    }
                                                    Err(e) => {
                                                        log!("Failed to update lang: {:?}", e);
                                                        set_error.set(Some("Failed to update language.".to_string()));
                                                        set_saving.set(false);
                                                    }
                                                }
                                            });
                                        }
                                    }
                                    disabled=move || saving.get()
                                >
                                    {move || if saving.get() { "Saving..." } else { "Save Language" }}
                                </button>
                            </div>

                            <h2>"Modules"</h2>
                            <div class="settings-modules">
                                <label class="settings-toggle">
                                    <input type="checkbox"
                                        checked=move || ai_module.get()
                                        on:change=move |_| set_ai_module.update(|v| *v = !*v)
                                    />
                                    <span>"AI Module"</span>
                                </label>
                                <label class="settings-toggle">
                                    <input type="checkbox"
                                        checked=move || anilist_module.get()
                                        on:change=move |_| set_anilist_module.update(|v| *v = !*v)
                                    />
                                    <span>"AniList Module"</span>
                                </label>
                                <label class="settings-toggle">
                                    <input type="checkbox"
                                        checked=move || game_module.get()
                                        on:change=move |_| set_game_module.update(|v| *v = !*v)
                                    />
                                    <span>"Game Module"</span>
                                </label>
                                <label class="settings-toggle">
                                    <input type="checkbox"
                                        checked=move || anime_module.get()
                                        on:change=move |_| set_anime_module.update(|v| *v = !*v)
                                    />
                                    <span>"Anime Module"</span>
                                </label>
                                <label class="settings-toggle">
                                    <input type="checkbox"
                                        checked=move || vn_module.get()
                                        on:change=move |_| set_vn_module.update(|v| *v = !*v)
                                    />
                                    <span>"VN Module"</span>
                                </label>
                                <label class="settings-toggle">
                                    <input type="checkbox"
                                        checked=move || level_module.get()
                                        on:change=move |_| set_level_module.update(|v| *v = !*v)
                                    />
                                    <span>"Level Module"</span>
                                </label>
                                <label class="settings-toggle">
                                    <input type="checkbox"
                                        checked=move || mini_game_module.get()
                                        on:change=move |_| set_mini_game_module.update(|v| *v = !*v)
                                    />
                                    <span>"Mini Game Module"</span>
                                </label>
                            </div>
                            <button
                                class="settings-btn"
                                on:click={
                                    let gid = gid_modules;
                                    move |_| {
                                        let gid = gid.clone();
                                        set_saving.set(true);
                                        set_error.set(None);
                                        set_success.set(None);
                                        let modules = serde_json::json!({
                                            "ai_module": ai_module.get_untracked(),
                                            "anilist_module": anilist_module.get_untracked(),
                                            "game_module": game_module.get_untracked(),
                                            "anime_module": anime_module.get_untracked(),
                                            "vn_module": vn_module.get_untracked(),
                                            "level_module": level_module.get_untracked(),
                                            "mini_game_module": mini_game_module.get_untracked(),
                                        });
                                        spawn_local(async move {
                                            match update_guild_modules(get_jwt(), &gid, modules.to_string()).await {
                                                Ok(_) => {
                                                    set_success.set(Some("Modules updated.".to_string()));
                                                    set_saving.set(false);
                                                }
                                                Err(e) => {
                                                    log!("Failed to update modules: {:?}", e);
                                                    set_error.set(Some("Failed to update modules.".to_string()));
                                                    set_saving.set(false);
                                                }
                                            }
                                        });
                                    }
                                }
                                disabled=move || saving.get()
                                style="margin-top: 15px;"
                            >
                                {move || if saving.get() { "Saving..." } else { "Save Modules" }}
                            </button>
                        </div>
                    }.into_any()
                }}
            </div>
        </main>
    }
}
