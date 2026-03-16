use crate::components::app_shell::AppShell;
use crate::views::{accounts::Accounts, add::Add, settings::Settings};
use dioxus::prelude::*;

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[layout(AppShell)]
    #[route("/")]
    Accounts {},
    #[route("/add")]
    Add {},
    #[route("/settings")]
    Settings {},
}
