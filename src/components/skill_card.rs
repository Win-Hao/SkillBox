use dioxus::prelude::*;
use crate::models::{Skill, SkillFilter, SkillSource};
use crate::hooks::SkillsState;
use crate::components::icons::IconHardDrive;
use crate::services::skill_io;
use crate::theme::{AppSettings, t};

#[component]
pub fn SkillCard(skill: Skill) -> Element {
    let mut state = use_context::<Signal<SkillsState>>();
    let settings = use_context::<Signal<AppSettings>>();
    let locale = settings.read().locale;
    let name = skill.dir_name.clone();
    let display_name = skill.frontmatter.name.clone();
    let excerpt = skill.description_excerpt(300);
    let source = skill.source.clone();
    let enabled = skill.enabled;
    let file_count = skill.file_count;
    let size = skill.human_size();
    let installed_date = skill.formatted_installed_date();
    let is_linked = source == SkillSource::Linked;
    let is_custom = source == SkillSource::Custom;
    let linked_from = skill.linked_from.clone();
    let toggle_name = name.clone();
    let check_name = name.clone();
    let open_name = name.clone();

    let mut toggle_guard = use_signal(|| false);
    let mut dismissing = use_signal(|| false);

    let select_mode = state.read().select_mode;
    let is_selected = state.read().is_selected(&name);
    let drawer_open = state.read().detail_skill.is_some();

    let (dot_r, dot_a, dot_g) = if !enabled {
        ("bg-gray-300 dark:bg-gray-600", "bg-gray-300 dark:bg-gray-600", "bg-gray-300 dark:bg-gray-600")
    } else {
        ("bg-red-400", "bg-amber-400", "bg-green-400")
    };

    let source_label = match &source {
        SkillSource::Custom => t("card.my_skill", locale).to_string(),
        SkillSource::Linked => match &linked_from {
            Some(from) => format!("{} · {from}", t("card.linked", locale)),
            None => t("card.linked", locale).to_string(),
        },
        SkillSource::Downloaded => t("card.downloaded", locale).to_string(),
        SkillSource::LooseFile => t("card.file", locale).to_string(),
    };

    let card_border = if is_selected {
        "border-2 border-blue-500"
    } else if !enabled {
        "border border-gray-300 dark:border-gray-600 opacity-60"
    } else {
        "border border-gray-300 dark:border-gray-600"
    };

    let is_dismissing = *dismissing.read();

    let wrap_class = if is_dismissing { "card-dismissing" } else { "" };

    let inner_class = if drawer_open {
        format!("bg-white dark:bg-gray-800 rounded-xl {card_border} flex flex-col cursor-pointer overflow-hidden")
    } else {
        format!("bg-white dark:bg-gray-800 rounded-xl {card_border} flex flex-col cursor-pointer overflow-hidden group card-interactive")
    };

    let files_word = t("card.files", locale);

    rsx! {
      div {
        class: wrap_class,
        div {
            class: inner_class,
            onclick: {
                let check_name = check_name.clone();
                let open_name = open_name.clone();
                move |_| {
                    if *toggle_guard.read() {
                        toggle_guard.set(false);
                        return;
                    }
                    if select_mode {
                        state.write().toggle_selected(&check_name);
                    } else {
                        state.write().detail_skill = Some(open_name.clone());
                    }
                }
            },

            // Title bar
            div {
                class: "flex items-center justify-between px-4 py-2.5 border-b border-gray-200 dark:border-gray-700 bg-gray-50/50 dark:bg-gray-800/50",
                div {
                    class: "flex items-center gap-2 min-w-0",
                    if select_mode {
                        div {
                            class: if is_selected {
                                "w-4 h-4 rounded border-2 border-blue-500 bg-blue-500 flex items-center justify-center flex-shrink-0"
                            } else {
                                "w-4 h-4 rounded border-2 border-gray-300 dark:border-gray-500 bg-white dark:bg-gray-700 flex-shrink-0"
                            },
                            if is_selected {
                                svg {
                                    class: "w-2.5 h-2.5 text-white",
                                    fill: "none",
                                    stroke: "currentColor",
                                    view_box: "0 0 24 24",
                                    overflow: "visible",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        stroke_width: "3",
                                        d: "M5 13l4 4L19 7",
                                    }
                                }
                            }
                        }
                    } else {
                        div {
                            class: "flex items-center gap-1.5 flex-shrink-0 opacity-50 transition-opacity duration-200 group-hover:opacity-100",
                            div { class: "w-2.5 h-2.5 rounded-full {dot_r}" }
                            div { class: "w-2.5 h-2.5 rounded-full {dot_a}" }
                            div { class: "w-2.5 h-2.5 rounded-full {dot_g}" }
                        }
                    }
                    span {
                        class: "text-sm font-semibold text-gray-800 dark:text-gray-100 truncate ml-1",
                        "{display_name}"
                    }
                }
                div {
                    class: "flex items-center gap-2 flex-shrink-0",
                    IconHardDrive { class: "w-3 h-3 text-gray-400 dark:text-gray-500" }
                    span { class: "text-xs text-gray-500 dark:text-gray-400 font-medium", "{size}" }
                }
            }

            // Body
            div {
                class: "px-4 py-3.5 flex-1 flex flex-col",
                // Source tag
                p {
                    class: "text-xs mb-2.5",
                    span { class: "text-gray-400 dark:text-gray-500", "{t(\"card.from\", locale)} " }
                    span {
                        class: if is_custom { "text-blue-600 dark:text-blue-400 font-medium" } else if is_linked { "text-purple-600 dark:text-purple-400 font-medium" } else { "text-amber-600 dark:text-amber-400 font-medium" },
                        "\"{source_label}\""
                    }
                }
                // Description
                p {
                    class: "text-[13px] text-gray-600 dark:text-gray-300 flex-1 mb-3.5 leading-relaxed",
                    "{excerpt}"
                }
                // Footer
                div {
                    class: "flex items-center justify-between text-xs text-gray-400 dark:text-gray-500 pt-2.5 border-t border-gray-200 dark:border-gray-700",
                    if let Some(ref date) = installed_date {
                        span { "{date}" }
                    } else {
                        span { "{file_count} {files_word}" }
                    }
                    div {
                        class: "flex items-center gap-2",
                        if display_name != name {
                            span { class: "font-mono", "/{name}" }
                        }
                        button {
                            class: "card-toggle relative inline-flex items-center rounded-full transition-colors duration-200 flex-shrink-0",
                            style: if enabled {
                                "width: 28px; height: 16px; background-color: #D97757;"
                            } else {
                                "width: 28px; height: 16px; background-color: #d1d5db;"
                            },
                            title: if enabled { t("card.disable", locale) } else { t("card.enable", locale) },
                            onclick: {
                                let toggle_name = toggle_name.clone();
                                move |_| {
                                    toggle_guard.set(true);
                                    let sk = state.read().find_skill(&toggle_name).cloned();
                                    let filter = state.read().filter.clone();
                                    if let Some(sk) = sk {
                                        let should_animate = matches!(
                                            (&filter, sk.enabled),
                                            (SkillFilter::Enabled, true) | (SkillFilter::Disabled, false)
                                        );
                                        if should_animate {
                                            dismissing.set(true);
                                        }
                                        let is_enabled = sk.enabled;
                                        spawn(async move {
                                            if should_animate {
                                                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                                            }
                                            if is_enabled {
                                                let _ = skill_io::disable_skill(&sk);
                                            } else {
                                                let _ = skill_io::enable_skill(&sk);
                                            }
                                            state.write().reload();
                                            dismissing.set(false);
                                        });
                                    }
                                }
                            },
                            span {
                                class: "absolute bg-white rounded-full shadow transition-all duration-200",
                                style: if enabled {
                                    "width: 12px; height: 12px; top: 2px; left: 14px;"
                                } else {
                                    "width: 12px; height: 12px; top: 2px; left: 2px;"
                                },
                            }
                        }
                    }
                }
            }
        }
      }
    }
}
