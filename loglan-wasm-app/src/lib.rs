mod app; // Import the App component

use wasm_bindgen::prelude::*;

// Called by our JS entry point to run the example
#[wasm_bindgen(start)]
pub fn run_app() -> Result<(), JsValue> {
    // Initialize logging and panic hook
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();

    log::info!("Starting Loglan WASM App");

    // Mount the Yew app
    let document = web_sys::window().unwrap().document().unwrap();
    let mount_point = document
        .get_element_by_id("yew-app")
        .expect("Should have #yew-app div");
    yew::Renderer::<app::App>::with_root(mount_point.clone()).render(); // Clone mount_point if needed later

    // Hide loading indicator and show the app container
    if let Some(loading_indicator) = document.get_element_by_id("loading-indicator") {
        loading_indicator
            .dyn_into::<web_sys::HtmlElement>()
            .map(|el| el.style().set_property("display", "none"))
            .expect("Loading indicator should be an HTMLElement with style")
            .expect("Failed to hide loading indicator");
    }

    // Show the Yew app container
    mount_point
        .dyn_into::<web_sys::HtmlElement>()
        .map(|el| el.style().remove_property("display")) // Remove display: none to show it
        .expect("Yew mount point should be an HTMLElement with style")
        .expect("Failed to show Yew app container");


    Ok(())
}
