use dioxus::prelude::*;
use crate::components::layout::Layout;
use crate::components::search_bar::SearchBar;
use crate::components::skill_grid::SkillGrid;
use crate::theme::{AppSettings, t};

#[derive(Routable, Clone, Debug, PartialEq)]
pub enum Route {
    #[layout(Layout)]
        #[route("/")]
        SkillList {},
    #[end_layout]

    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}

#[component]
fn SkillList() -> Element {
    rsx! {
        div {
            SearchBar {}
            SkillGrid {}
        }
    }
}

#[component]
fn NotFound(segments: Vec<String>) -> Element {
    let settings = use_context::<Signal<AppSettings>>();
    let locale = settings.read().locale;
    rsx! {
        div {
            class: "flex items-center justify-center min-h-screen",
            p { class: "text-gray-500 dark:text-gray-400", {t("notfound.message", locale)} }
        }
    }
}
