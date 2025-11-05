
use leptos::*;
use leptos::prelude::ClassAttribute;
use leptos::prelude::ElementChild;
use leptos::prelude::create_signal;
use leptos::prelude::Update;
use leptos::prelude::Get;
use leptos::prelude::OnAttribute;
use leptos::prelude::GlobalAttributes;
#[component]
pub fn Features() -> impl IntoView {
    view! {
        <section class="section" id="features">
            <div class="container">
                <div class="section-title">
                    <h2>"Powerful Features"</h2>
                    <p>"Kasuki comes packed with features to enhance your Discord anime experience."</p>
                </div>
                <div class="features">
                    <div class="feature-card">
                        <div class="feature-icon"><i class="fas fa-tv"></i></div>
                        <h3>"AniList Integration"</h3>
                        <p>"Access anime, manga, characters, staff, and studio information directly from AniList. Get detailed information and statistics with simple commands."</p>
                    </div>
                    <div class="feature-card">
                        <div class="feature-icon"><i class="fas fa-bell"></i></div>
                        <h3>"Anime Airing Notifications"</h3>
                        <p>"Get notified when your favorite anime episodes are about to air. Never miss an episode again!"</p>
                    </div>
                    <div class="feature-card">
                        <div class="feature-icon"><i class="fas fa-user-friends"></i></div>
                        <h3>"User Profiles & Comparison"</h3>
                        <p>"Link your AniList account to Discord and view profiles, compare anime tastes with friends, and show off your anime stats."</p>
                    </div>
                    <div class="feature-card">
                        <div class="feature-icon"><i class="fas fa-microphone-alt"></i></div>
                        <h3>"Seiyuu Image Generation"</h3>
                        <p>"Generate custom images showing voice actors alongside characters they've voiced for easy reference."</p>
                    </div>
                    <div class="feature-card">
                        <div class="feature-icon"><i class="fas fa-robot"></i></div>
                        <h3>"AI Features"</h3>
                        <p>"Generate images, get answers to questions, and access video transcription and translation using advanced AI technology."</p>
                    </div>
                    <div class="feature-card">
                        <div class="feature-icon"><i class="fas fa-language"></i></div>
                        <h3>"Multi-Language Support"</h3>
                        <p>"Kasuki supports multiple languages for both commands and responses, making it accessible to users worldwide."</p>
                    </div>
                </div>
            </div>
        </section>
    }
}