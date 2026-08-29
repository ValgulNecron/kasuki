mod app;
mod config;
mod api;
mod components;
use crate::app::App;
use leptos::mount::mount_to_body;
use leptos::view;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| {
        view! {
            <App/>
        }
    })
}
