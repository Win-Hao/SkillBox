use dioxus::prelude::*;
use crate::hooks::SkillsState;
use crate::theme::{AppSettings, t};

#[component]
pub fn SearchBar() -> Element {
    let mut state = use_context::<Signal<SkillsState>>();
    let settings = use_context::<Signal<AppSettings>>();
    let locale = settings.read().locale;

    let filter_label = t(state.read().filter.i18n_key(), locale);
    let filtered_count = state.read().filtered_skills().len();
    let skills_word = t("search.skills_count", locale);

    rsx! {
        div {
            class: "sticky top-0 z-10 bg-gray-50/80 dark:bg-gray-900/80 backdrop-blur-sm border-b border-gray-200 dark:border-gray-700",
            div {
                class: "max-w-6xl mx-auto px-4 py-3.5 sm:px-6 sm:py-4",
                div {
                    class: "bg-claude-50/50 dark:bg-gray-800/50 rounded-xl border-2 border-claude-light/50 dark:border-claude-dark/50 hover:border-claude-light/70 dark:hover:border-claude/70 focus-within:border-claude/70 transition-all duration-300 ease-in-out overflow-hidden",
                    // Title bar
                    div {
                        class: "flex items-center justify-between px-3 py-2 border-b border-claude-light/30 dark:border-claude-dark/30 bg-claude-50/50 dark:bg-gray-800/50",
                        div {
                            class: "flex items-center gap-1.5",
                            div { class: "w-2.5 h-2.5 rounded-full bg-red-400" }
                            div { class: "w-2.5 h-2.5 rounded-full bg-amber-400" }
                            div { class: "w-2.5 h-2.5 rounded-full bg-green-400" }
                            span {
                                class: "text-xs font-medium text-gray-600 dark:text-gray-300 ml-2",
                                "{filter_label} · {filtered_count} {skills_word}"
                            }
                        }
                        button {
                            class: "px-3 py-1 text-xs font-medium rounded-lg bg-claude text-white hover:bg-claude-dark active:scale-[0.95] transition-all duration-150",
                            onclick: move |_| state.write().show_upload = true,
                            {t("search.upload", locale)}
                        }
                    }
                    // Input row
                    div {
                        class: "flex items-center gap-2 px-3 py-2.5 bg-white dark:bg-gray-800",
                        span { class: "text-sm text-green-600 dark:text-green-400 font-mono flex-shrink-0", "$ ls" }
                        svg {
                            class: "w-3 h-3 text-gray-300 dark:text-gray-500 flex-shrink-0",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            overflow: "visible",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607z",
                            }
                        }
                        input {
                            class: "flex-1 py-0.5 bg-transparent text-sm font-mono focus:outline-none placeholder-gray-400 dark:placeholder-gray-500 text-gray-800 dark:text-gray-200",
                            r#type: "text",
                            placeholder: t("search.placeholder", locale),
                            value: "{state.read().search_query}",
                            oninput: move |evt| {
                                state.write().search_query = evt.value().to_string();
                            },
                        }
                    }
                }
            }
        }

    }
}
