mod app; // Import the App component

use wasm_bindgen::prelude::*;

// Called by our JS entry point to run the example
#[wasm_bindgen(start)]
pub fn run_app() -> Result<(), JsValue> {
    // Initialize logging and panic hook
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();

    log::info!("Starting Loglan WASM App");

    // Mount the Yew app into the designated div
    let document = web_sys::window().unwrap().document().unwrap();
    let mount_point = document.get_element_by_id("yew-app").unwrap();
    yew::Renderer::<app::App>::with_root(mount_point).render();

    Ok(())
}
