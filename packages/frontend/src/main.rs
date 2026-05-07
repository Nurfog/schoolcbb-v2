mod api;
mod app;
mod components;
mod route;

fn main() {
    dioxus::launch(app::App);
}
