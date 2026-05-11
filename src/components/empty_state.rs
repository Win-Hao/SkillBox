use dioxus::prelude::*;
use crate::components::icons::ClaudeMascot;
use crate::services::scanner::{self, ClaudeStatus};
use crate::theme::{AppSettings, t};

#[component]
pub fn EmptyState(message: String, is_total_empty: Option<bool>) -> Element {
    let settings = use_context::<Signal<AppSettings>>();
    let locale = settings.read().locale;
    let total_empty = is_total_empty.unwrap_or(false);

    if total_empty {
        let status = scanner::claude_install_status();
        let dir_display = scanner::skills_base_dir().display().to_string();

        return match status {
            ClaudeStatus::NotInstalled => rsx! {
                div {
                    class: "flex flex-col items-center justify-center py-20",
                    svg {
                        class: "w-14 h-14 mb-4 text-gray-300 dark:text-gray-600",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        overflow: "visible",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "1.5",
                            d: "M12 9v3.75m9-.75a9 9 0 11-18 0 9 9 0 0118 0zm-9 3.75h.008v.008H12v-.008z",
                        }
                    }
                    p { class: "text-sm font-medium text-gray-500 dark:text-gray-400 mb-1", {t("empty.not_detected", locale)} }
                    p { class: "text-xs text-gray-400 dark:text-gray-500 mb-3 max-w-xs text-center",
                        {t("empty.install_first", locale)}
                    }
                    p { class: "text-[11px] text-gray-400 dark:text-gray-500 font-mono", "{dir_display}" }
                }
            },
            ClaudeStatus::DirOnly | ClaudeStatus::Installed => rsx! {
                div {
                    class: "flex flex-col items-center justify-center py-20",
                    div {
                        class: "mb-4 opacity-40",
                        ClaudeMascot { class: "w-16 h-16" }
                    }
                    p { class: "text-sm font-medium text-gray-500 dark:text-gray-400 mb-1", {t("empty.no_skills", locale)} }
                    p { class: "text-xs text-gray-400 dark:text-gray-500 max-w-xs text-center",
                        {t("empty.browse_hint", locale)}
                    }
                    p { class: "text-[11px] text-gray-400 dark:text-gray-500 font-mono mt-3", "{dir_display}" }
                }
            },
        };
    }

    rsx! {
        div {
            class: "flex flex-col items-center justify-center py-20 text-gray-400 dark:text-gray-500",
            svg {
                class: "w-12 h-12 mb-3",
                fill: "none",
                stroke: "currentColor",
                view_box: "0 0 24 24",
                overflow: "visible",
                path {
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    stroke_width: "1.5",
                    d: "M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z",
                }
            }
            p {
                class: "text-sm font-medium",
                "{message}"
            }
        }
    }
}
