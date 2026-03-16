use crate::components::app_shell::AppShell;
use crate::views::{accounts::Accounts, add::Add, settings::Settings};
use dioxus::prelude::*;

/// Top-level application routes.
#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[layout(AppShell)]
    /// The main accounts list view, served at `/`.
    #[route("/")]
    Accounts {},
    /// The add-account form view, served at `/add`.
    #[route("/add")]
    Add {},
    /// The settings view, served at `/settings`.
    #[route("/settings")]
    Settings {},
}
