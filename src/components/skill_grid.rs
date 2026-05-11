use dioxus::prelude::*;
use crate::hooks::SkillsState;
use crate::models::skill::Skill;
use crate::components::skill_card::SkillCard;
use crate::components::empty_state::EmptyState;
use crate::components::confirm_dialog::ConfirmDialog;
use crate::services::skill_io;
use crate::theme::{AppSettings, t};

fn estimate_skill_height(skill: &Skill) -> u32 {
    let base = 110u32;
    let desc_len = skill.frontmatter.description.len() as u32;
    let desc_lines = (desc_len / 45).min(4);
    base + desc_lines * 18
}

fn distribute_to_columns<'a>(skills: &'a [Skill], col_count: usize) -> Vec<Vec<&'a Skill>> {
    let mut columns: Vec<Vec<&Skill>> = (0..col_count).map(|_| Vec::new()).collect();
    let mut col_heights = vec![0u32; col_count];
    for skill in skills {
        let shortest = col_heights.iter().enumerate().min_by_key(|(_, h)| *h).map(|(i, _)| i).unwrap_or(0);
        col_heights[shortest] += estimate_skill_height(skill);
        columns[shortest].push(skill);
    }
    columns
}

#[component]
pub fn SkillGrid() -> Element {
    let mut state = use_context::<Signal<SkillsState>>();
    let settings = use_context::<Signal<AppSettings>>();
    let locale = settings.read().locale;
    let mut show_delete_confirm = use_signal(|| false);
    let filtered = state.read().filtered_skills().into_iter().cloned().collect::<Vec<_>>();
    let select_mode = state.read().select_mode;
    let selected_count = state.read().selected_count();
    let total_filtered = filtered.len();

    let mut col_count = use_signal(|| 3usize);

    use_future(move || async move {
        let mut eval = document::eval(r#"
            (function() {
                function calc() {
                    var el = document.getElementById('main-scroll');
                    var w = el ? el.clientWidth : window.innerWidth;
                    if (w < 580) return 1;
                    if (w < 900) return 2;
                    return 3;
                }
                dioxus.send(calc());
                var ro = new ResizeObserver(function() { dioxus.send(calc()); });
                var el = document.getElementById('main-scroll');
                if (el) ro.observe(el);
            })();
        "#);
        while let Ok(count) = eval.recv::<usize>().await {
            if count != *col_count.peek() {
                col_count.set(count);
            }
        }
    });

    let cols = *col_count.read();

    rsx! {
        div {
            class: if select_mode { "px-4 py-5 sm:px-6 sm:py-6 pb-24 max-w-6xl mx-auto" } else { "px-4 py-5 sm:px-6 sm:py-6 max-w-6xl mx-auto" },
            if filtered.is_empty() {
                {
                    let all_empty = state.read().skills.is_empty();
                    let search_active = !state.read().search_query.is_empty();
                    if search_active {
                        rsx! { EmptyState { message: t("empty.no_match", locale).to_string() } }
                    } else {
                        rsx! { EmptyState { message: t("empty.no_found", locale).to_string(), is_total_empty: all_empty } }
                    }
                }
            } else {
                {
                    let columns = distribute_to_columns(&filtered, cols);
                    rsx! {
                        div {
                            class: "flex gap-4",
                            for col in columns.iter() {
                                div {
                                    class: "flex-1 flex flex-col gap-4 min-w-0",
                                    for skill in col.iter() {
                                        SkillCard {
                                            key: "{skill.dir_name}",
                                            skill: (*skill).clone(),
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Floating action bar at bottom
        if select_mode {
            div {
                class: "fixed bottom-0 left-0 right-0 z-50",
                div {
                    class: "max-w-4xl mx-auto px-4 pb-4 sm:px-6 sm:pb-6",
                    div {
                        class: "bg-gray-900 text-white rounded-xl shadow-2xl px-6 py-3 flex items-center justify-between",
                        // Left: count + select all
                        div {
                            class: "flex items-center gap-4",
                            span {
                                class: "text-sm font-medium",
                                "{selected_count} {t(\"grid.selected\", locale)}"
                            }
                            if selected_count < total_filtered {
                                button {
                                    class: "text-sm text-blue-400 hover:text-blue-300 transition-colors",
                                    onclick: {
                                        let names: Vec<String> = filtered.iter().map(|s| s.dir_name.clone()).collect();
                                        move |_| {
                                            state.write().select_all(&names);
                                        }
                                    },
                                    {t("grid.select_all", locale)}
                                }
                            }
                        }
                        // Right: actions
                        div {
                            class: "flex items-center gap-3",
                            button {
                                class: "px-4 py-1.5 text-sm font-medium rounded-lg bg-red-600 hover:bg-red-500 transition-colors",
                                onclick: move |_| show_delete_confirm.set(true),
                                {t("grid.delete", locale)}
                            }
                            button {
                                class: "text-sm text-gray-400 hover:text-white transition-colors",
                                onclick: move |_| {
                                    state.write().clear_selection();
                                },
                                svg {
                                    class: "w-5 h-5",
                                    fill: "none",
                                    stroke: "currentColor",
                                    view_box: "0 0 24 24",
                                    overflow: "visible",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        stroke_width: "2",
                                        d: "M6 18L18 6M6 6l12 12",
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if *show_delete_confirm.read() {
            ConfirmDialog {
                title: format!("{} {} {}", t("grid.delete", locale), selected_count, t("grid.selected", locale)),
                message: t("empty.no_found", locale).to_string(),
                confirm_label: t("grid.delete", locale).to_string(),
                on_confirm: move |_| {
                    let names: Vec<String> = state.read().selected.iter().cloned().collect();
                    for name in &names {
                        if let Some(skill) = state.read().find_skill(name).cloned() {
                            let _ = skill_io::delete_skill(&skill);
                        }
                    }
                    state.write().clear_selection();
                    state.write().reload();
                    show_delete_confirm.set(false);
                },
                on_cancel: move |_| show_delete_confirm.set(false),
            }
        }
    }
}
