use leptos::*;
use crate::components::header::Header;
use crate::components::footer::Footer;
use crate::components::hero::Hero;
use crate::components::features::Features;
use crate::components::commands::Commands;
use crate::components::screenshots::Screenshots;
use crate::components::setup::Setup;
use leptos::prelude::ClassAttribute;
use leptos::prelude::ElementChild;
use leptos::prelude::create_signal;
use leptos::prelude::Update;
use leptos::prelude::Get;
use leptos::prelude::OnAttribute;
use leptos::prelude::GlobalAttributes;
use leptos::prelude::CustomAttribute;
use leptos::prelude::document;
use leptos::prelude::create_effect;

#[component]
pub fn App() -> impl IntoView {
    let (is_dark, set_is_dark) = create_signal(false);

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
            <main>
                <Hero />
                <Features />
                <Commands />
                <Screenshots />
                <Setup />
            </main>
            <Footer />
            <button class="theme-toggle" on:click=move |_| set_is_dark.update(|val| *val = !*val)>
                <i class={move || if is_dark.get() { "fas fa-sun" } else { "fas fa-moon" }}></i>
            </button>
        </div>
    }
}