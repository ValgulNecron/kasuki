use leptos::prelude::ClassAttribute;
use leptos::prelude::ElementChild;
use leptos::*;
#[component]
pub fn Hero() -> impl IntoView {
    view! {
        <section class="hero">
            <div class="container">
                <div class="hero-content">
                    <h2>"Enhance Your Discord Server with Anime & Manga"</h2>
                    <p>"Kasuki is a powerful Discord bot that provides seamless access to AniList data, AI features, and more. Get instant anime and manga information, notifications, and user stats right in your server!"</p>
                    <div class="hero-buttons">
                        <a class="btn btn-primary" href="https://discord.com/api/oauth2/authorize?client_id=923286536445894697&permissions=533113194560&scope=bot" target="_blank" rel="noopener noreferrer">
                            <i class="fab fa-discord"></i>" Add to Discord"
                        </a>
                        <a class="btn btn-secondary" href="#setup">
                            <i class="fas fa-server"></i>" Self-Host Guide"
                        </a>
                    </div>
                </div>
            </div>
        </section>
    }
}