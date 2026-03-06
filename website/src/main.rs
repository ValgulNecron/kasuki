mod app;
mod config;
mod api; // Declare the api module
mod components; // Explicitly import mount_to_body
use crate::app::App;
// Explicitly import view!
use leptos::mount::mount_to_body;
// Declare the components module
use leptos::view;
// Explicitly import App

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| {
        view! {
            <App/>
        }
    })
}
