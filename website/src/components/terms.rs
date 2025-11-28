use leptos::*;
use leptos::prelude::*;

#[component]
pub fn Terms() -> impl IntoView {
    view! {
        <main class="legal-page">
            <div class="legal-container">
                <header>
                    <h1>"Terms of Service"</h1>
                    <p class="last-updated">"Last updated: June 07, 2025"</p>
                </header>

                <h2>"1. Introduction"</h2>
                <p>"Welcome to Kasuki (\"the Bot\"), a Discord bot created by Valgul. These Terms of Service (\"Terms\") govern your access to and use of Kasuki, including any associated services, features, content, and websites (collectively, the \"Service\"). By adding Kasuki to your Discord server or using the Service in any way, you agree to be bound by these Terms."</p>

                <h2>"2. Basic Terms"</h2>
                <p>"2.1. You must be at least 13 years old and comply with Discord's Terms of Service to use Kasuki."</p>
                <p>"2.2. You are responsible for all activity that occurs under your account or on your Discord server related to the use of Kasuki."</p>

                <h2>"3. User Content and Conduct"</h2>
                <p>"3.1. You are solely responsible for your conduct and any data, text, files, information, usernames, images, or other content that you submit, post, or display through Kasuki."</p>
                <p>"3.2. You must not use Kasuki for spamming, harassment, or any abusive behavior."</p>

                <h2>"4. Service Availability and Modifications"</h2>
                <p>"6.1. The Service is provided \"as is\" without warranties of any kind, either express or implied."</p>
                <p>"6.2. We reserve the right to modify or discontinue, temporarily or permanently, the Service (or any part thereof) with or without notice."</p>
                
                <h2>"12. Contact"</h2>
                <p>"For questions about these Terms, please contact us through the " <a href="https://github.com/ValgulNecron/kasuki/issues" target="_blank" rel="noopener noreferrer">"GitHub issues"</a> " on the Kasuki repository."</p>
            </div>
        </main>
    }
}
