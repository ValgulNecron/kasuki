
use leptos::*;
use leptos::prelude::ClassAttribute;
use leptos::prelude::ElementChild;
use leptos::prelude::create_signal;
use leptos::prelude::Update;
use leptos::prelude::Get;
use leptos::prelude::OnAttribute;
use leptos::prelude::GlobalAttributes;
use leptos::prelude::Set;
#[component]
pub fn Setup() -> impl IntoView {
    view! {
        <section class="section" id="setup">
        <div class="container">
        <div class="section-title">
            <h2>"Self-Hosting Guide"</h2>
            <p>"Follow these steps to set up your own instance of Kasuki."</p>
            </div>
            <div class="setup-steps">
        <div class="step">
        <div class="step-number">"1"</div>
            <div class="step-content">
            <h3>"Install Docker & Docker Compose"</h3>
            <p>"Ensure Docker and Docker Compose are installed on your system. They are required to run Kasuki in a containerized environment."</p>
            </div>
            </div>
            <div class="step">
        <div class="step-number">"2"</div>
            <div class="step-content">
            <h3>"Clone the Repository"</h3>
            <p>"Clone the Kasuki repository from GitHub to your local machine."</p>
            <div class="code-block">"git clone https://github.com/ValgulNecron/kasuki.git"</div>
            </div>
            </div>
            <div class="step">
        <div class="step-number">"3"</div>
            <div class="step-content">
            <h3>"Configure Settings"</h3>
            <p>"Copy the example configuration file and edit it to add your Discord bot token and other settings."</p>
            <div class="code-block">"cp config.example.toml config.toml"
            <br/>
            "# Edit config.toml with your preferred text editor"
            </div>
            </div>
            </div>
            <div class="step">
        <div class="step-number">"4"</div>
            <div class="step-content">
            <h3>"Start the Container"</h3>
            <p>"Use Docker Compose to build and start the Kasuki bot. For the latest version, add the --pull always flag."</p>
            <div class="code-block">"docker compose up -d --pull always"</div>
            </div>
            </div>
            <div class="step">
        <div class="step-number">"5"</div>
            <div class="step-content">
            <h3>"Invite Your Bot"</h3>
            <p>"Create an invite link for your bot in the Discord Developer Portal and add it to your server. Your bot is now ready to use!"</p>
            </div>
            </div>
            </div>
            </div>
            </section>
    }
}