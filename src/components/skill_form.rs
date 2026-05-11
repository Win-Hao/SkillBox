use dioxus::prelude::*;
use crate::hooks::SkillsState;
use crate::models::{parse_skill_md, serialize_skill_md};
use crate::components::toast::ToastManager;
use crate::routes::Route;
use crate::theme::{AppSettings, t};
use std::fs;

#[component]
pub fn SkillEdit(name: String) -> Element {
    let mut state = use_context::<Signal<SkillsState>>();
    let settings = use_context::<Signal<AppSettings>>();
    let locale = settings.read().locale;
    let skill = state.read().find_skill(&name).cloned();

    let Some(skill) = skill else {
        return rsx! {
            div { class: "p-6", {t("form.not_found", locale)} }
        };
    };

    let mut form_name = use_signal(|| skill.frontmatter.name.clone());
    let mut form_desc = use_signal(|| skill.frontmatter.description.clone());
    let mut form_version = use_signal(|| skill.frontmatter.version.clone().unwrap_or_default());
    let mut form_body = use_signal(|| skill.body.clone());
    let mut toast = use_context::<Signal<ToastManager>>();

    let skill_path = skill.path.clone();
    let nav_name = name.clone();

    rsx! {
        div {
            class: "max-w-3xl mx-auto p-6",
            Link {
                to: Route::SkillDetail { name: nav_name.clone() },
                class: "inline-flex items-center gap-1 text-sm text-gray-500 hover:text-gray-700 mb-6",
                svg {
                    class: "w-4 h-4",
                    fill: "none",
                    stroke: "currentColor",
                    view_box: "0 0 24 24",
                    overflow: "visible",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M15 19l-7-7 7-7",
                    }
                }
                {t("form.back", locale)}
            }

            h1 {
                class: "text-2xl font-bold text-gray-900 mb-6",
                "Edit: {name}"
            }

            div {
                class: "space-y-5",
                FormField {
                    label: t("form.name", locale).to_string(),
                    value: form_name(),
                    on_change: move |v: String| form_name.set(v),
                }
                div {
                    label {
                        class: "block text-sm font-medium text-gray-700 mb-1",
                        {t("form.description", locale)}
                    }
                    textarea {
                        class: "w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 resize-y",
                        rows: 4,
                        value: "{form_desc}",
                        oninput: move |evt| form_desc.set(evt.value().to_string()),
                    }
                }
                FormField {
                    label: t("form.version", locale).to_string(),
                    value: form_version(),
                    on_change: move |v: String| form_version.set(v),
                }
                div {
                    label {
                        class: "block text-sm font-medium text-gray-700 mb-1",
                        {t("form.body", locale)}
                    }
                    textarea {
                        class: "w-full px-3 py-2 border border-gray-300 rounded-lg text-sm font-mono focus:outline-none focus:ring-2 focus:ring-blue-500 resize-y",
                        rows: 16,
                        value: "{form_body}",
                        oninput: move |evt| form_body.set(evt.value().to_string()),
                    }
                }

                div {
                    class: "flex gap-3 pt-4",
                    button {
                        class: "px-5 py-2 text-sm font-medium rounded-lg bg-blue-600 text-white hover:bg-blue-700 transition-colors",
                        onclick: {
                            let skill_path = skill_path.clone();
                            let nav_name = nav_name.clone();
                            move |_| {
                                let skill_md_path = skill_path.join("SKILL.md");
                                // Re-read to preserve fields not shown in the form
                                // (e.g., allowed-tools, hooks, metadata).
                                let content = fs::read_to_string(&skill_md_path).unwrap_or_default();
                                let (mut fm, _) = parse_skill_md(&content).unwrap_or_default();

                                fm.name = form_name.read().clone();
                                fm.description = form_desc.read().clone();
                                let ver = form_version.read().clone();
                                fm.version = if ver.is_empty() { None } else { Some(ver) };

                                match serialize_skill_md(&fm, &form_body.read()) {
                                    Ok(output) => {
                                        match fs::write(&skill_md_path, &output) {
                                            Ok(_) => {
                                                state.write().reload();
                                                navigator().push(Route::SkillDetail { name: nav_name.clone() });
                                            }
                                            Err(e) => {
                                                toast.write().error(format!("Failed to save: {e}"));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        toast.write().error(format!("Failed to serialize: {e}"));
                                    }
                                }
                            }
                        },
                        {t("form.save", locale)}
                    }
                    Link {
                        to: Route::SkillDetail { name: nav_name.clone() },
                        class: "px-5 py-2 text-sm font-medium rounded-lg border border-gray-300 text-gray-700 hover:bg-gray-100 transition-colors",
                        {t("form.cancel", locale)}
                    }
                }
            }
        }
    }
}

#[component]
fn FormField(label: String, value: String, on_change: EventHandler<String>) -> Element {
    rsx! {
        div {
            label {
                class: "block text-sm font-medium text-gray-700 mb-1",
                "{label}"
            }
            input {
                class: "w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500",
                r#type: "text",
                value: "{value}",
                oninput: move |evt| on_change.call(evt.value().to_string()),
            }
        }
    }
}
