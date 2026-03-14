use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default)]
pub enum AppPage {
    #[default]
    Home,
    AddEntry,
    Settings,
}
