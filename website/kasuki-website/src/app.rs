use leptos::*;
use crate::components::header::Header;
use crate::components::footer::Footer;
use crate::components::hero::Hero;
use crate::components::features::Features;
use crate::components::commands::Commands;
use crate::components::screenshots::Screenshots;
use crate::components::setup::Setup;
use crate::components::privacy::Privacy;
use crate::components::terms::Terms;
use leptos::prelude::ClassAttribute;
use leptos::prelude::ElementChild;
use leptos::prelude::create_signal;
use leptos::prelude::Update;
use leptos::prelude::Get;
use leptos::prelude::Set;
use leptos::prelude::OnAttribute;
use leptos::prelude::GlobalAttributes;
use leptos::prelude::CustomAttribute;
use leptos::prelude::document;
use leptos::prelude::create_effect;
use leptos::prelude::IntoAny;
use wasm_bindgen::JsCast;
use web_sys::HashChangeEvent;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Page {
    Home,
    Privacy,
    Terms,
}

#[component]
pub fn App() -> impl IntoView {
    let (is_dark, set_is_dark) = create_signal(false);
    let (current_page, set_current_page) = create_signal(Page::Home);

    // Handle initial page load based on hash
    create_effect(move |_| {
        if let Some(window) = web_sys::window() {
            let hash = window.location().hash().unwrap_or_default();
            let page = match hash.as_str() {
                "#/privacy" => Page::Privacy,
                "#/terms" => Page::Terms,
                _ => Page::Home,
            };
            set_current_page.set(page);

            // Listen for hash changes
            let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |_event: HashChangeEvent| {
                if let Some(window) = web_sys::window() {
                    let hash = window.location().hash().unwrap_or_default();
                    let page = match hash.as_str() {
                        "#/privacy" => Page::Privacy,
                        "#/terms" => Page::Terms,
                        _ => Page::Home,
                    };
                    set_current_page.set(page);
                }
            }) as Box<dyn FnMut(_)>);
            
            let _ = window.add_event_listener_with_callback("hashchange", closure.as_ref().unchecked_ref());
            closure.forget();
        }
    });

    create_effect(move |_| {
        if let Some(body) = document().body() {
            let is_dark_now = is_dark.get();
            if is_dark_now {
                let _ = body.set_attribute("data-theme", "dark");
            } else {
                let _ = body.remove_attribute("data-theme");
            }
        }
    });

    view! {
        <div id="app">
            <Header />
            {move || match current_page.get() {
                Page::Home => view! {
                    <main>
                        <Hero />
                        <Features />
                        <Commands />
                        <Screenshots />
                        <Setup />
                    </main>
                }.into_any(),
                Page::Privacy => view! { <Privacy /> }.into_any(),
                Page::Terms => view! { <Terms /> }.into_any(),
            }}
            <Footer />
            <button class="theme-toggle" on:click=move |_| set_is_dark.update(|val| *val = !*val)>
                <i class={move || if is_dark.get() { "fas fa-sun" } else { "fas fa-moon" }}></i>
            </button>
        </div>
    }
}