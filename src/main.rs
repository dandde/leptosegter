pub mod app;
pub mod backend {
    pub mod pli_segmenter;
    pub mod types;
}
pub mod components {
    pub mod input_ui;
    pub mod result_ui;
}

use app::App;
use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> })
}
