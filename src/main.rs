use brew_vault::{
    components::totp_card::TotpCard,
    models::totp::{Algorithm, Digits, TotpEntry},
};
use dioxus::prelude::*;
use uuid::Uuid;

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let totp_entry = TotpEntry {
        id: Uuid::new_v4().into(),
        issuer: "BrewingBytes".into(),
        account: "account@brewingbytes.com".into(),
        secret: "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ".into(),
        algorithm: Algorithm::Sha1,
        digits: Digits::Six,
        period: 30,
    };

    rsx! {
        document::Stylesheet { href: MAIN_CSS }
        div {
            class: "min-h-screen bg-gray-950 flex items-center justify-center",
            TotpCard { entry: totp_entry }
        }
    }
}
