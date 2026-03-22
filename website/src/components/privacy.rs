use leptos::prelude::*;

#[component]
pub fn Privacy() -> impl IntoView {
    view! {
        <main class="legal-page">
            <div class="legal-container">
                <header>
                    <h1>"Privacy Policy"</h1>
                    <p class="last-updated">"Last updated: July 14, 2025"</p>
                </header>

                <h2>"1. Introduction"</h2>
                <p>"This Privacy Policy explains how Kasuki (\"the Bot\", \"we\", \"us\", or \"our\") collects, uses, and shares information about you when you use our Discord bot and related services (collectively, the \"Service\"). We take your privacy seriously and are committed to protecting your personal information."</p>

                <h2>"2. Information We Collect"</h2>
                <h3>"2.1. Information You Provide"</h3>
                <p>"When using Kasuki, you may provide us with certain information, such as your Discord user ID, server ID, and the content of commands you submit."</p>
                <h3>"2.2. Information Collected Automatically"</h3>
                <p>"We automatically collect certain information when you interact with Kasuki, like command usage statistics and error logs."</p>
                
                <div class="highlight-box">
                    <p><strong>"Note:"</strong> " Self-hosted instances are managed by their owners, who are responsible for their own data privacy practices."</p>
                </div>

                <h2>"3. User Rights and Choices"</h2>
                <p>"You can remove your registered AniList account using bot commands, and server administrators can control bot features. You may request deletion of your data by contacting us. On the repository there is also a file called \"blacklist.json\" it is possible for you to ask to be added thus being excluded from the server image."</p>
                
                <h2>"4. Contact Us"</h2>
                <p>"If you have any questions about this Privacy Policy, please contact us through the " <a href="https://github.com/ValgulNecron/kasuki/issues" target="_blank" rel="noopener noreferrer">"GitHub issues"</a> " on the Kasuki repository."</p>
            </div>
        </main>
    }
}
