use dioxus::prelude::*;
use crate::hooks::SkillsState;
use crate::components::badge::{SourceBadge, StatusBadge};
use crate::components::confirm_dialog::ConfirmDialog;
use crate::components::icons::*;
use crate::routes::Route;
use crate::services::{markdown, skill_io};
use crate::models::SkillSource;
use crate::theme::{AppSettings, t};

#[component]
pub fn SkillDetail(name: String) -> Element {
    let mut state = use_context::<Signal<SkillsState>>();
    let settings = use_context::<Signal<AppSettings>>();
    let locale = settings.read().locale;
    let mut show_delete_confirm = use_signal(|| false);

    let skill = state.read().find_skill(&name).cloned();

    let Some(skill) = skill else {
        return rsx! {
            div {
                class: "p-6",
                p { class: "text-gray-500", "{t(\"detail.not_found\", locale)}: {name}" }
                Link {
                    to: Route::SkillList {},
                    class: "text-blue-600 hover:underline text-sm mt-2 inline-block",
                    {t("detail.back_to_list", locale)}
                }
            }
        };
    };

    let body_html = markdown::md_to_html(&skill.body);
    let skill_name = skill.dir_name.clone();
    let is_enabled = skill.enabled;
    let is_symlink = skill.source == SkillSource::Linked;
    let is_custom = skill.source == SkillSource::Custom;
    let can_toggle_source = !skill.source_locked;

    rsx! {
        div {
            class: "max-w-4xl mx-auto p-6",

            // Back nav
            Link {
                to: Route::SkillList {},
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
                {t("detail.back", locale)}
            }

            // Header
            div {
                class: "mb-6",
                div {
                    class: "flex items-center gap-3 mb-2 flex-wrap",
                    h1 {
                        class: "text-2xl font-bold text-gray-900",
                        "{skill.frontmatter.name}"
                    }
                    SourceBadge { source: skill.source.clone(), linked_from: skill.linked_from.clone() }
                    StatusBadge { enabled: skill.enabled }
                    if let Some(ref v) = skill.frontmatter.version {
                        span {
                            class: "text-sm text-gray-400",
                            "v{v}"
                        }
                    }
                }
                p {
                    class: "text-sm text-gray-500 font-mono",
                    "{skill.path.display()}"
                }
            }

            // Actions
            div {
                class: "flex gap-3 mb-6",
                button {
                    class: if is_enabled { "px-4 py-2 text-sm font-medium rounded-lg border border-gray-300 text-gray-700 hover:bg-gray-100 transition-colors" } else { "px-4 py-2 text-sm font-medium rounded-lg bg-green-600 text-white hover:bg-green-700 transition-colors" },
                    onclick: {
                        let name = skill_name.clone();
                        move |_| {
                            let s = state.read().find_skill(&name).cloned();
                            if let Some(ref sk) = s {
                                let result = if sk.enabled {
                                    skill_io::disable_skill(sk)
                                } else {
                                    skill_io::enable_skill(sk)
                                };
                                if result.is_ok() {
                                    state.write().reload();
                                }
                            }
                        }
                    },
                    if is_enabled { {t("detail.disable", locale)} } else { {t("detail.enable", locale)} }
                }

                Link {
                    to: Route::SkillEdit { name: skill_name.clone() },
                    class: "px-4 py-2 text-sm font-medium rounded-lg bg-blue-600 text-white hover:bg-blue-700 transition-colors",
                    {t("detail.edit", locale)}
                }

                if can_toggle_source {
                    button {
                        class: "px-4 py-2 text-sm font-medium rounded-lg border border-gray-300 text-gray-700 hover:bg-gray-100 transition-colors",
                        onclick: {
                            let name = skill_name.clone();
                            move |_| {
                                let s = state.read().find_skill(&name).cloned();
                                if let Some(ref sk) = s {
                                    if skill_io::toggle_custom_mark(&sk.dir_name).is_ok() {
                                        state.write().reload();
                                    }
                                }
                            }
                        },
                        if is_custom { {t("detail.mark_downloaded", locale)} } else { {t("detail.mark_custom", locale)} }
                    }
                }

                button {
                    class: "px-4 py-2 text-sm font-medium rounded-lg border border-red-300 text-red-600 hover:bg-red-50 transition-colors",
                    onclick: move |_| show_delete_confirm.set(true),
                    if is_symlink { {t("detail.remove_link", locale)} } else { {t("detail.delete", locale)} }
                }
            }

            // Metadata
            div {
                class: "bg-white rounded-xl border border-gray-200 p-5 mb-6",
                h2 {
                    class: "text-sm font-semibold text-gray-400 uppercase tracking-wider mb-3",
                    {t("detail.metadata", locale)}
                }
                div {
                    class: "grid grid-cols-2 gap-3 text-sm",
                    MetaField { label: t("detail.name", locale).to_string(), value: skill.frontmatter.name.clone() }
                    MetaField { label: t("detail.files", locale).to_string(), value: format!("{}", skill.file_count) }
                    MetaField { label: t("detail.size", locale).to_string(), value: skill.human_size() }
                    if let Some(ref tier) = skill.frontmatter.preamble_tier {
                        MetaField { label: t("detail.preamble_tier", locale).to_string(), value: format!("{}", tier) }
                    }
                    if let Some(ref license) = skill.frontmatter.license {
                        MetaField { label: t("detail.license", locale).to_string(), value: license.clone() }
                    }
                    if let Some(ref hint) = skill.frontmatter.argument_hint {
                        MetaField { label: t("detail.argument_hint", locale).to_string(), value: hint.clone() }
                    }
                }
                if let Some(ref tools) = skill.frontmatter.allowed_tools {
                    div {
                        class: "mt-3 pt-3 border-t border-gray-100",
                        p {
                            class: "text-xs text-gray-400 mb-1",
                            {t("detail.allowed_tools", locale)}
                        }
                        div {
                            class: "flex flex-wrap gap-1",
                            for tool in tools {
                                span {
                                    class: "inline-block px-2 py-0.5 bg-gray-100 rounded text-xs text-gray-600 font-mono",
                                    "{tool}"
                                }
                            }
                        }
                    }
                }
            }

            // Description
            div {
                class: "bg-white rounded-xl border border-gray-200 p-5 mb-6",
                h2 {
                    class: "text-sm font-semibold text-gray-400 uppercase tracking-wider mb-3",
                    {t("detail.description", locale)}
                }
                p {
                    class: "text-sm text-gray-700 whitespace-pre-wrap",
                    "{skill.frontmatter.description}"
                }
            }

            // Body (rendered markdown)
            if !skill.body.is_empty() {
                div {
                    class: "bg-white rounded-xl border border-gray-200 p-5 mb-6",
                    h2 {
                        class: "text-sm font-semibold text-gray-400 uppercase tracking-wider mb-3",
                        {t("detail.content", locale)}
                    }
                    div {
                        class: "prose prose-sm max-w-none text-gray-700",
                        dangerous_inner_html: "{body_html}",
                    }
                }
            }

            // Files
            if !skill.files.is_empty() {
                div {
                    class: "bg-white rounded-xl border border-gray-200 p-5",
                    h2 {
                        class: "text-sm font-semibold text-gray-400 uppercase tracking-wider mb-3",
                        "{t(\"detail.files_count\", locale)} ({skill.file_count})"
                    }
                    div {
                        class: "space-y-1",
                        for file in &skill.files {
                            div {
                                class: "flex items-center gap-2 text-sm py-1",
                                if file.is_dir {
                                    IconFolder { class: "w-4 h-4 text-amber-500" }
                                } else {
                                    IconFile { class: "w-4 h-4 text-gray-400" }
                                }
                                span {
                                    class: "font-mono text-gray-700 flex-1",
                                    "{file.relative_path}"
                                }
                                if !file.is_dir {
                                    span {
                                        class: "text-xs text-gray-400",
                                        "{bytesize::ByteSize(file.size)}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Delete confirmation
            if *show_delete_confirm.read() {
                ConfirmDialog {
                    title: if is_symlink { t("dialog.delete_linked", locale).to_string() } else { t("dialog.delete_skill", locale).to_string() },
                    message: if is_symlink {
                        t("dialog.symlink_confirm", locale).replace("{}", &skill_name)
                    } else {
                        t("dialog.delete_confirm", locale).replace("{}", &skill_name)
                    },
                    confirm_label: if is_symlink { t("dialog.remove_link_only", locale).to_string() } else { t("detail.delete", locale).to_string() },
                    on_confirm: {
                        let name = skill_name.clone();
                        move |_| {
                            let s = state.read().find_skill(&name).cloned();
                            if let Some(ref sk) = s {
                                if skill_io::delete_skill(sk).is_ok() {
                                    state.write().reload();
                                    navigator().push(Route::SkillList {});
                                }
                            }
                            show_delete_confirm.set(false);
                        }
                    },
                    on_cancel: move |_| show_delete_confirm.set(false),
                    secondary_label: if is_symlink { Some(t("dialog.delete_completely", locale).to_string()) } else { None },
                    on_secondary: {
                        let name = skill_name.clone();
                        if is_symlink { Some(EventHandler::new(move |_| {
                            let s = state.read().find_skill(&name).cloned();
                            if let Some(ref sk) = s {
                                if skill_io::delete_skill_completely(sk).is_ok() {
                                    state.write().reload();
                                    navigator().push(Route::SkillList {});
                                }
                            }
                            show_delete_confirm.set(false);
                        })) } else { None }
                    },
                }
            }
        }
    }
}

#[component]
fn MetaField(label: String, value: String) -> Element {
    rsx! {
        div {
            p {
                class: "text-xs text-gray-400",
                "{label}"
            }
            p {
                class: "text-gray-700",
                "{value}"
            }
        }
    }
}
