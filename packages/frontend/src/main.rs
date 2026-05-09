mod api;
mod app;
mod components;
mod route;

fn main() {
    register_service_worker();
    dioxus::launch(app::App);
}

fn register_service_worker() {
    let window = web_sys::window();
    if let Some(w) = window {
        let navigator = w.navigator();
        let sw_container = navigator.service_worker();
        let promise = sw_container.register("/service-worker.js");
        let _ = wasm_bindgen_futures::spawn_local(async move {
            let result = wasm_bindgen_futures::JsFuture::from(promise).await;
            if let Err(e) = result {
                web_sys::console::log_1(&format!("SW registration failed: {:?}", e).into());
            }
        });
    }
}
