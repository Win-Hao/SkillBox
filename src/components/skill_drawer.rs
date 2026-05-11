use dioxus::prelude::*;
use std::collections::HashSet;
use std::fs;
use crate::hooks::SkillsState;
use crate::components::confirm_dialog::ConfirmDialog;
use crate::components::icons::*;
use crate::services::{markdown, skill_io};
use crate::models::SkillSource;
use crate::theme::{AppSettings, t};

#[component]
pub fn SkillDrawer() -> Element {
    let mut state = use_context::<Signal<SkillsState>>();
    let settings = use_context::<Signal<AppSettings>>();
    let locale = settings.read().locale;
    let mut show_delete_confirm = use_signal(|| false);
    let mut file_content = use_signal(|| None::<String>);
    let mut expanded_dirs = use_signal(HashSet::<String>::new);

    let skill_name = state.read().detail_skill.clone();
    let Some(ref name) = skill_name else {
        return rsx! {};
    };

    let skill = state.read().find_skill(name).cloned();
    let Some(skill) = skill else {
        return rsx! {};
    };

    let is_loose_file = skill.path.is_file();
    let main_file_name = if is_loose_file {
        skill.path.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or("SKILL.md".to_string())
    } else {
        "SKILL.md".to_string()
    };
    let mut selected_file = use_signal(|| None::<String>);
    let mut show_scroll_top = use_signal(|| false);
    let mut closing = use_signal(|| false);
    let mut prev_skill = use_signal(String::new);
    if *prev_skill.read() != skill.dir_name {
        prev_skill.set(skill.dir_name.clone());
        selected_file.set(Some(main_file_name.clone()));
        file_content.set(None);
        expanded_dirs.set(HashSet::new());
        closing.set(false);
    }

    let is_closing = *closing.read();
    let body_html = markdown::md_to_html(&skill.body);
    let skill_name = skill.dir_name.clone();
    let is_enabled = skill.enabled;
    let is_symlink = skill.source == SkillSource::Linked;
    let is_custom = skill.source == SkillSource::Custom;
    let can_toggle_source = !skill.source_locked;
    let is_main_selected = selected_file.read().as_ref() == Some(&main_file_name);

    let files_label = t("drawer.files_label", locale);
    let files_count_label = format!("{} {}", skill.file_count, t("card.files", locale));

    rsx! {
        // Backdrop
        div {
            class: if is_closing { "fixed inset-0 z-40 bg-black/30 backdrop-exit" } else { "fixed inset-0 z-40 bg-black/30 backdrop-enter" },
            onclick: move |_| {
                if !*closing.read() {
                    closing.set(true);
                    spawn(async move {
                        let mut res = document::eval(r#"setTimeout(() => dioxus.send(1), 300);"#);
                        let _ = res.recv::<f64>().await;
                        state.write().detail_skill = None;
                    });
                }
            },
        }
        // Drawer panel
        div {
            id: "drawer-scroll",
            class: if is_closing {
                "fixed top-0 right-0 bottom-0 z-50 w-full max-w-full sm:max-w-lg md:max-w-xl lg:max-w-2xl bg-gray-50 dark:bg-gray-900 shadow-2xl overflow-y-auto overflow-x-hidden drawer-exit"
            } else {
                "fixed top-0 right-0 bottom-0 z-50 w-full max-w-full sm:max-w-lg md:max-w-xl lg:max-w-2xl bg-gray-50 dark:bg-gray-900 shadow-2xl overflow-y-auto overflow-x-hidden drawer-enter"
            },
            onscroll: move |_| {
                spawn(async move {
                    let mut res = document::eval(r#"
                        let el = document.getElementById('drawer-scroll');
                        dioxus.send(el ? el.scrollTop : 0);
                    "#);
                    if let Ok(val) = res.recv::<f64>().await {
                        show_scroll_top.set(val > 200.0);
                    }
                });
            },
            // Header
            div {
                class: "sticky top-0 bg-white/90 dark:bg-gray-800/90 backdrop-blur-sm border-b border-gray-200 dark:border-gray-700 px-5 py-3.5",
                div {
                    class: "flex items-center justify-between mb-2.5",
                    div {
                        class: "flex items-center gap-2.5",
                        span { class: "text-xs font-medium text-gray-400 dark:text-gray-500 uppercase tracking-wider", {t("drawer.skill_details", locale)} }
                        if !is_enabled {
                            span { class: "text-xs text-red-400 font-medium", {t("drawer.disabled_label", locale)} }
                        }
                        if let Some(ref v) = skill.frontmatter.version {
                            span { class: "text-xs text-gray-400 dark:text-gray-500", "v{v}" }
                        }
                    }
                    button {
                        class: "p-1 rounded-md text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors",
                        onclick: move |_| {
                            if !*closing.read() {
                                closing.set(true);
                                spawn(async move {
                                    let mut res = document::eval(r#"setTimeout(() => dioxus.send(1), 300);"#);
                                    let _ = res.recv::<f64>().await;
                                    state.write().detail_skill = None;
                                });
                            }
                        },
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
                                d: "M6 18L18 6M6 6l12 12",
                            }
                        }
                    }
                }
                div {
                    class: "flex items-center gap-2",
                    h2 {
                        class: "text-xl font-bold text-gray-900 dark:text-gray-100 truncate tracking-tight",
                        "{skill.frontmatter.name}"
                    }
                    {match &skill.source {
                        SkillSource::Custom => rsx! {
                            span { class: "inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 dark:bg-blue-900/40 text-blue-700 dark:text-blue-300 shrink-0", {t("card.my_skill", locale)} }
                        },
                        SkillSource::Linked => rsx! {
                            span { class: "inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-purple-100 dark:bg-purple-900/40 text-purple-700 dark:text-purple-300 shrink-0",
                                if let Some(ref from) = skill.linked_from {
                                    "{t(\"card.linked\", locale)} · {from}"
                                } else {
                                    {t("card.linked", locale)}
                                }
                            }
                        },
                        SkillSource::Downloaded => rsx! {
                            span { class: "inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-amber-100 dark:bg-amber-900/40 text-amber-700 dark:text-amber-300 shrink-0", {t("card.downloaded", locale)} }
                        },
                        SkillSource::LooseFile => rsx! {
                            span { class: "inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 shrink-0", {t("card.file", locale)} }
                        },
                    }}
                }
                p {
                    class: "text-xs text-gray-400 dark:text-gray-500 font-mono truncate mt-0.5",
                    "{skill.path.display()}"
                }
            }

            div {
                class: "px-5 py-5 space-y-5 bg-gray-50 dark:bg-gray-900",

                // Actions
                div {
                    class: "flex flex-wrap gap-2",
                    button {
                        class: if is_enabled {
                            "px-4 py-2 text-xs font-medium rounded-lg border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 active:scale-[0.97] transition-all duration-150"
                        } else {
                            "px-4 py-2 text-xs font-medium rounded-lg bg-green-600 text-white hover:bg-green-700 active:scale-[0.97] transition-all duration-150"
                        },
                        onclick: {
                            let name = skill_name.clone();
                            move |_| {
                                let s = state.read().find_skill(&name).cloned();
                                if let Some(ref sk) = s {
                                    let result = if sk.enabled { skill_io::disable_skill(sk) } else { skill_io::enable_skill(sk) };
                                    if result.is_ok() {
                                        let mut s = state.write();
                                        s.detail_skill = None;
                                        s.reload();
                                    }
                                }
                            }
                        },
                        if is_enabled { {t("drawer.disable", locale)} } else { {t("drawer.enable", locale)} }
                    }
                    if can_toggle_source {
                        button {
                            class: "px-4 py-2 text-xs font-medium rounded-lg border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 active:scale-[0.97] transition-all duration-150",
                            onclick: {
                                let name = skill_name.clone();
                                move |_| {
                                    if skill_io::toggle_custom_mark(&name).is_ok() { state.write().reload(); }
                                }
                            },
                            if is_custom { {t("drawer.mark_downloaded", locale)} } else { {t("drawer.mark_custom", locale)} }
                        }
                    }
                    button {
                        class: "px-4 py-2 text-xs font-medium rounded-lg border border-red-300 dark:border-red-700 text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/30 active:scale-[0.97] transition-all duration-150",
                        onclick: move |_| show_delete_confirm.set(true),
                        if is_symlink { {t("drawer.remove_link", locale)} } else { {t("drawer.delete", locale)} }
                    }
                }

                // Stats card
                div {
                    class: "bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 overflow-hidden",
                    div {
                        class: "px-4 py-2 border-b border-gray-100 dark:border-gray-700 bg-gray-50/50 dark:bg-gray-800/50 text-xs font-mono text-gray-500 dark:text-gray-400",
                        span { class: "text-green-600 dark:text-green-400", "$ " }
                        {t("drawer.stat_cmd", locale)}
                    }
                    div {
                        class: "px-4 py-3 flex flex-wrap items-center gap-4 sm:gap-6 text-sm",
                        span {
                            class: "flex items-center gap-1.5",
                            IconPackage { class: "w-4 h-4 text-gray-400 dark:text-gray-500 icon-animated" }
                            span { class: "text-gray-500 dark:text-gray-400", {files_label} }
                            span { class: "font-semibold text-gray-800 dark:text-gray-100", "{skill.file_count}" }
                        }
                        span {
                            class: "flex items-center gap-1.5",
                            IconHardDrive { class: "w-4 h-4 text-gray-400 dark:text-gray-500 icon-animated" }
                            span { class: "text-gray-500 dark:text-gray-400", {t("drawer.size", locale)} }
                            span { class: "font-semibold text-gray-800 dark:text-gray-100", "{skill.human_size()}" }
                        }
                        if let Some(ref tier) = skill.frontmatter.preamble_tier {
                            span {
                                class: "flex items-center gap-1.5",
                                IconZap { class: "w-4 h-4 text-gray-400 dark:text-gray-500 icon-animated" }
                                span { class: "text-gray-500 dark:text-gray-400", {t("drawer.tier", locale)} }
                                span { class: "font-semibold text-gray-800 dark:text-gray-100", "{tier}" }
                            }
                        }
                    }
                }

                // File Explorer card
                if !skill.files.is_empty() {
                    div {
                        class: "bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 overflow-hidden",
                        div {
                            class: "flex items-center justify-between px-4 py-2 border-b border-gray-100 dark:border-gray-700 bg-gray-50/50 dark:bg-gray-800/50",
                            div {
                                class: "flex items-center gap-2",
                                div {
                                    class: "flex items-center gap-1.5",
                                    div { class: "w-2.5 h-2.5 rounded-full bg-red-400" }
                                    div { class: "w-2.5 h-2.5 rounded-full bg-amber-400" }
                                    div { class: "w-2.5 h-2.5 rounded-full bg-green-400" }
                                }
                                span { class: "text-xs font-medium text-gray-600 dark:text-gray-300 ml-1", {t("drawer.file_explorer", locale)} }
                            }
                            span { class: "text-[10px] text-gray-400 dark:text-gray-500", "{files_count_label}" }
                        }
                        {
                            let visible_files: Vec<_> = skill.files.iter().filter(|file| {
                                let depth = file.relative_path.matches('/').count();
                                if depth == 0 { return true; }
                                let parent = file.relative_path.rsplitn(2, '/').nth(1).unwrap_or("");
                                let parts: Vec<&str> = parent.split('/').collect();
                                parts.iter().enumerate().all(|(i, _)| {
                                    let ancestor = parts[..=i].join("/");
                                    expanded_dirs.read().contains(&ancestor)
                                })
                            }).collect();

                            let main_fn = main_file_name.clone();
                            rsx! {
                                div {
                                    class: "px-4 py-2",
                                    for file in visible_files {
                                        {
                                            let depth = file.relative_path.matches('/').count();
                                            let indent = format!("{}rem", depth as f32 * 1.2 + 0.25);
                                            let is_dir = file.is_dir;
                                            let file_name = file.relative_path.rsplit('/').next().unwrap_or(&file.relative_path).to_string();
                                            let is_skill_md = file_name == "SKILL.md";
                                            let main_file_cmp = main_fn.clone();
                                            let is_selected_file = selected_file.read().as_ref() == Some(&file.relative_path);
                                            let row_bg = if is_selected_file { "bg-orange-50 dark:bg-orange-900/20 rounded" } else { "hover:bg-gray-50 dark:hover:bg-gray-700/50 rounded" };
                                            let text_class = if is_selected_file {
                                                "font-mono text-xs text-orange-500 font-medium"
                                            } else {
                                                "font-mono text-xs text-gray-700 dark:text-gray-300"
                                            };
                                            let click_path = file.relative_path.clone();
                                            let dir_path = file.relative_path.clone();
                                            let skill_path = skill.path.clone();
                                            let is_expanded = is_dir && expanded_dirs.read().contains(&file.relative_path);

                                            rsx! {
                                                div {
                                                    class: "flex items-center gap-1 py-1 cursor-pointer transition-colors {row_bg}",
                                                    style: "padding-left: {indent}",
                                                    onclick: move |_| {
                                                        if is_dir {
                                                            let mut dirs = expanded_dirs.write();
                                                            if dirs.contains(&dir_path) {
                                                                dirs.remove(&dir_path);
                                                            } else {
                                                                dirs.insert(dir_path.clone());
                                                            }
                                                        } else {
                                                            selected_file.set(Some(click_path.clone()));
                                                            if click_path == main_file_cmp {
                                                                file_content.set(None);
                                                            } else {
                                                                let full_path = skill_path.join(&click_path);
                                                                let too_large = fs::metadata(&full_path)
                                                                    .map(|m| m.len() > 1_048_576)
                                                                    .unwrap_or(false);
                                                                let content = if too_large {
                                                                    t("toast.file_too_large", locale).to_string()
                                                                } else {
                                                                    fs::read_to_string(&full_path)
                                                                        .unwrap_or_else(|_| t("toast.binary_file", locale).to_string())
                                                                };
                                                                file_content.set(Some(content));
                                                            }
                                                        }
                                                    },
                                                    if is_dir {
                                                        svg {
                                                            class: "w-3 h-3 text-gray-400 dark:text-gray-500 flex-shrink-0 transition-transform duration-150",
                                                            style: if is_expanded { "transform: rotate(90deg);" } else { "" },
                                                            fill: "none",
                                                            stroke: "currentColor",
                                                            view_box: "0 0 24 24",
                                                            overflow: "visible",
                                                            path {
                                                                stroke_linecap: "round",
                                                                stroke_linejoin: "round",
                                                                stroke_width: "2",
                                                                d: "M9 6l6 6-6 6",
                                                            }
                                                        }
                                                        IconFolder { class: "w-4 h-4 text-amber-500 flex-shrink-0" }
                                                    } else {
                                                        span { class: "w-3 flex-shrink-0" }
                                                        if is_skill_md {
                                                            IconFileText { class: "w-4 h-4 text-gray-400 dark:text-gray-500 flex-shrink-0" }
                                                        } else {
                                                            IconFile { class: "w-4 h-4 text-gray-400 dark:text-gray-500 flex-shrink-0" }
                                                        }
                                                    }
                                                    span { class: "{text_class} flex-1", "{file_name}" }
                                                    if !is_dir {
                                                        span { class: "text-[10px] text-gray-400 dark:text-gray-500 flex-shrink-0", "{bytesize::ByteSize(file.size)}" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // File content viewer (non-main files only)
                    {
                        let sel = selected_file.read().clone();
                        let show_other = sel.as_ref().is_some_and(|s| s != &main_file_name);
                        if show_other {
                            if let Some(ref content) = *file_content.read() {
                                let Some(sel_path) = sel else { return rsx! {} };
                                let line_count = content.lines().count();
                                let is_md = sel_path.ends_with(".md");
                                let lines_word = t("drawer.lines", locale);
                                rsx! {
                                    div {
                                        class: "bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 overflow-hidden",
                                        div {
                                            class: "flex items-center justify-between px-4 py-2 border-b border-gray-100 dark:border-gray-700 bg-gray-50/50 dark:bg-gray-800/50",
                                            div {
                                                class: "flex items-center gap-2",
                                                div {
                                                    class: "flex items-center gap-1.5",
                                                    div { class: "w-2.5 h-2.5 rounded-full bg-red-400" }
                                                    div { class: "w-2.5 h-2.5 rounded-full bg-amber-400" }
                                                    div { class: "w-2.5 h-2.5 rounded-full bg-green-400" }
                                                }
                                                span { class: "text-xs font-medium text-gray-600 dark:text-gray-300 ml-1", "{sel_path}" }
                                            }
                                            span { class: "text-[10px] text-gray-400 dark:text-gray-500", {t("drawer.readonly", locale)} }
                                        }
                                        div {
                                            class: "p-4",
                                            p { class: "text-[10px] text-gray-400 dark:text-gray-500 text-right mb-2", "{line_count} {lines_word}" }
                                            if is_md {
                                                div {
                                                    class: "prose prose-sm dark:prose-invert max-w-none text-gray-700 dark:text-gray-200 overflow-x-auto break-words",
                                                    dangerous_inner_html: "{markdown::md_to_html(content)}",
                                                }
                                            } else {
                                                pre {
                                                    class: "text-xs text-gray-700 dark:text-gray-200 overflow-x-auto",
                                                    code { "{content}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                rsx! {}
                            }
                        } else {
                            rsx! {}
                        }
                    }
                }

                // Main file content card (shown when main file is selected)
                if is_main_selected {
                div {
                    class: "bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 overflow-hidden",
                    div {
                        class: "flex items-center justify-between px-4 py-2 border-b border-gray-100 dark:border-gray-700 bg-gray-50/50 dark:bg-gray-800/50",
                        div {
                            class: "flex items-center gap-2",
                            div {
                                class: "flex items-center gap-1.5",
                                div { class: "w-2.5 h-2.5 rounded-full bg-red-400" }
                                div { class: "w-2.5 h-2.5 rounded-full bg-amber-400" }
                                div { class: "w-2.5 h-2.5 rounded-full bg-green-400" }
                            }
                            span { class: "text-xs font-medium text-gray-600 dark:text-gray-300 ml-1", "{main_file_name}" }
                        }
                        span { class: "text-[10px] text-gray-400 dark:text-gray-500", {t("drawer.readonly", locale)} }
                    }
                    div {
                        class: "p-4",
                        // Frontmatter table
                        div {
                            class: "rounded-lg border border-gray-100 dark:border-gray-700 overflow-hidden mb-4",
                            div {
                                class: "flex border-b border-gray-100 dark:border-gray-700",
                                div { class: "w-28 px-3 py-2 bg-orange-50 dark:bg-orange-900/20 text-xs font-semibold text-orange-500 flex-shrink-0", "name" }
                                div { class: "px-3 py-2 text-sm text-gray-700 dark:text-gray-200 flex-1", "{skill.frontmatter.name}" }
                            }
                            div {
                                class: "flex",
                                div { class: "w-28 px-3 py-2 bg-orange-50 dark:bg-orange-900/20 text-xs font-semibold text-orange-500 flex-shrink-0", "description" }
                                div { class: "px-3 py-2 text-sm text-gray-700 dark:text-gray-200 flex-1 leading-relaxed", "{skill.frontmatter.description}" }
                            }
                        }

                        // Rendered body
                        if !skill.body.is_empty() {
                            div {
                                class: "prose prose-sm dark:prose-invert max-w-none text-gray-700 dark:text-gray-200 overflow-x-auto break-words",
                                dangerous_inner_html: "{body_html}",
                            }
                        }
                    }
                }
                }
            }

            // Back to top button
            div {
                class: if *show_scroll_top.read() { "scroll-top-wrap scroll-top-visible sticky bottom-0 w-full flex justify-center p-4" } else { "scroll-top-wrap scroll-top-hidden sticky bottom-0 w-full flex justify-center p-4" },
                button {
                    class: "scroll-top-btn flex items-center gap-2 px-5 py-2.5 text-sm font-mono text-gray-800 dark:text-gray-200 border border-claude-light/40 rounded-full shadow-lg",
                    style: "background: rgba(255,255,255,0.72); backdrop-filter: blur(16px); -webkit-backdrop-filter: blur(16px);",
                    onclick: move |_| {
                        document::eval(r#"document.getElementById('drawer-scroll').scrollTo({top: 0, behavior: 'smooth'})"#);
                    },
                    IconTerminal { class: "w-4 h-4 text-green-500" }
                    "cd"
                    IconArrowUp { class: "w-4 h-4 text-claude arrow-up" }
                    "top"
                }
            }
        }

        // Delete confirm
        if *show_delete_confirm.read() {
            ConfirmDialog {
                title: if is_symlink { t("dialog.delete_linked", locale).to_string() } else { t("dialog.delete_skill", locale).to_string() },
                message: if is_symlink {
                    t("dialog.is_symlink", locale).replace("{}", &skill_name)
                } else {
                    t("dialog.move_to_trash", locale).replace("{}", &skill_name)
                },
                confirm_label: if is_symlink { t("dialog.remove_link_only", locale).to_string() } else { t("drawer.delete", locale).to_string() },
                on_confirm: {
                    let name = skill_name.clone();
                    move |_| {
                        let s = state.read().find_skill(&name).cloned();
                        if let Some(ref sk) = s {
                            if skill_io::delete_skill(sk).is_ok() {
                                let mut sw = state.write();
                                sw.detail_skill = None;
                                sw.reload();
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
                                let mut sw = state.write();
                                sw.detail_skill = None;
                                sw.reload();
                            }
                        }
                        show_delete_confirm.set(false);
                    })) } else { None }
                },
            }
        }
    }
}
