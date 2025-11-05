
use leptos::*;
use leptos::prelude::ClassAttribute;
use leptos::prelude::ElementChild;
use leptos::prelude::create_signal;
use leptos::prelude::Update;
use leptos::prelude::Get;
use leptos::prelude::OnAttribute;
use leptos::prelude::GlobalAttributes;
#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer>
            <div class="container">
                <div class="footer-content">
                    <div class="footer-column">
                        <h3>"Kasuki"</h3>
                        <p>"A Discord bot for anime and manga lovers, packed with features and AI integration."</p>
                        <div class="social-links">
                            <a href="https://github.com/ValgulNecron/kasuki" target="_blank" rel="noopener noreferrer" aria-label="GitHub"><i class="fab fa-github"></i></a>
                            <a href="https://discord.gg/your-server-invite" target="_blank" rel="noopener noreferrer" aria-label="Discord"><i class="fab fa-discord"></i></a>
                        </div>
                    </div>
                    <div class="footer-column">
                        <h3>"Quick Links"</h3>
                        <ul class="footer-links">
                            <li><a href="#features">"Features"</a></li>
                            <li><a href="#commands">"Commands"</a></li>
                            <li><a href="#screenshots">"Screenshots"</a></li>
                            <li><a href="#setup">"Setup Guide"</a></li>
                        </ul>
                    </div>
                    <div class="footer-column">
                        <h3>"Resources"</h3>
                        <ul class="footer-links">
                            <li><a href="https://github.com/ValgulNecron/kasuki" target="_blank" rel="noopener noreferrer">"GitHub Repository"</a></li>
                            <li><a href="https://github.com/ValgulNecron/kasuki/issues" target="_blank" rel="noopener noreferrer">"Report an Issue"</a></li>
                            <li><a href="terms.html">"Terms of Service"</a></li>
                            <li><a href="privacy.html">"Privacy Policy"</a></li>
                        </ul>
                    </div>
                </div>
                <div class="footer-bottom">
                    <p>"© " <span class="year">"2025"</span> " Kasuki Bot. All rights reserved. Created with " <i class="fas fa-heart" style="color: var(--primary);"></i> " for the anime community."</p>
                </div>
            </div>
        </footer>
    }
}