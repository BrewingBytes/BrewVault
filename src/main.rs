use brew_vault::components::bottom_nav::BottomNav;
use brew_vault::views::{accounts::Accounts, add::Add, settings::Settings};
use dioxus::prelude::*;

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[layout(AppShell)]
    #[route("/")]
    Accounts {},
    #[route("/add")]
    Add {},
    #[route("/settings")]
    Settings {},
}

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: MAIN_CSS }
        Router::<Route> {}
    }
}

#[component]
fn AppShell() -> Element {
    let route = use_route::<Route>();
    let show_nav = route != Route::Add {};
    rsx! {
        div {
            class: "h-screen bg-base flex flex-col overflow-hidden",
            div { class: "flex-1 overflow-hidden", Outlet::<Route> {} }
            if show_nav {
                BottomNav {}
            }
        }
    }
}
