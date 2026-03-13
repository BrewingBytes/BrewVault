use dioxus::prelude::*;

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: MAIN_CSS }
        div {
            class: "min-h-screen bg-gray-950 flex items-center justify-center",
            h1 { class: "text-5xl font-bold text-white", "Hello, BrewVault!" }
        }
    }
}
