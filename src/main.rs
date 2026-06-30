mod app;
mod components;
mod serde_helpers;
mod state;
mod sudoku_engine;
mod utils;

use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| view! { <app::App /> });
}
