
use leptos::*;
use leptos::prelude::ClassAttribute;
use leptos::prelude::ElementChild;
use leptos::prelude::create_signal;
use leptos::prelude::Update;
use leptos::prelude::Get;
use leptos::prelude::OnAttribute;
use leptos::prelude::GlobalAttributes;
use leptos::prelude::Set;
use leptos::prelude::AriaAttributes;
#[component]
pub fn Header() -> impl IntoView {
    let (menu_open, set_menu_open) = create_signal(false);

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