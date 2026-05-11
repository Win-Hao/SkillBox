use std::collections::{HashMap, HashSet};
use dioxus::prelude::*;
use crate::hooks::{SkillsState, MarketplaceCache};
use crate::components::icons::*;
use crate::services::{marketplace::{self, GithubFile}, markdown};
use crate::components::toast::ToastManager;
use crate::theme::{AppSettings, t};

#[component]
pub fn MarketplaceDrawer() -> Element {
    let mut state = use_context::<Signal<SkillsState>>();
    let settings = use_context::<Signal<AppSettings>>();
    let locale = settings.read().locale;

    let Some(skill) = state.read().marketplace_preview.clone() else {
        return rsx! {};
    };

    let mut preview_content = use_signal(|| None::<String>);
    let mut preview_loading = use_signal(|| false);
    let mut preview_files = use_signal(|| None::<Vec<GithubFile>>);
    let mut expanded_dirs = use_signal(HashSet::<String>::new);
    let mut selected_file = use_signal(|| None::<String>);
    let mut file_cache = use_signal(HashMap::<String, String>::new);
    let mut drawer_show_top = use_signal(|| false);
    let mut closing = use_signal(|| false);
    let mut toast = use_context::<Signal<ToastManager>>();

    let mut mp_cache = use_context::<Signal<MarketplaceCache>>();

    let skill_id = skill.id.clone();
    use_effect(move || {
        let _ = skill_id.clone();
        expanded_dirs.set(HashSet::new());
        selected_file.set(Some("SKILL.md".to_string()));
        file_cache.set(HashMap::new());
        drawer_show_top.set(false);

        let Some(sk) = state.read().marketplace_preview.clone() else { return; };
        let sk_id = sk.id.clone();

        let cached = mp_cache.read().get_drawer(&sk_id).cloned();
        if let Some(entry) = cached {
            if let Some(content) = entry.content {
                file_cache.write().insert("SKILL.md".to_string(), content.clone());
                preview_content.set(Some(content));
                preview_loading.set(false);
            } else {
                preview_content.set(None);
                preview_loading.set(true);
                let sk_c = sk.clone();
                let id_c = sk_id.clone();
                spawn(async move {
                    match marketplace::fetch_skill_content(&sk_c).await {
                        Ok(content) => {
                            mp_cache.write().set_drawer_content(&id_c, content.clone());
                            let still_active = state.read().marketplace_preview.as_ref().map(|s| &s.id) == Some(&id_c);
                            if still_active {
                                file_cache.write().insert("SKILL.md".to_string(), content.clone());
                                preview_content.set(Some(content));
                                preview_loading.set(false);
                            }
                        }
                        Err(_) => {
                            let still_active = state.read().marketplace_preview.as_ref().map(|s| &s.id) == Some(&id_c);
                            if still_active {
                                preview_content.set(Some(t("toast.load_failed", locale).to_string()));
                                preview_loading.set(false);
                            }
                        }
                    }
                });
            }
            if let Some(files) = entry.files {
                preview_files.set(Some(files));
            } else {
                preview_files.set(None);
                let sk_c = sk.clone();
                let id_c = sk_id.clone();
                spawn(async move {
                    if let Ok(files) = marketplace::fetch_full_tree(&sk_c).await {
                        mp_cache.write().set_drawer_files(&id_c, files.clone());
                        let still_active = state.read().marketplace_preview.as_ref().map(|s| &s.id) == Some(&id_c);
                        if still_active {
                            preview_files.set(Some(files));
                        }
                    }
                });
            }
        } else {
            preview_content.set(None);
            preview_files.set(None);
            preview_loading.set(true);

            let sk2 = sk.clone();
            let id_c = sk_id.clone();
            let id_c2 = sk_id.clone();
            spawn(async move {
                match marketplace::fetch_skill_content(&sk).await {
                    Ok(content) => {
                        mp_cache.write().set_drawer_content(&id_c, content.clone());
                        let still_active = state.read().marketplace_preview.as_ref().map(|s| &s.id) == Some(&id_c);
                        if still_active {
                            file_cache.write().insert("SKILL.md".to_string(), content.clone());
                            preview_content.set(Some(content));
                            preview_loading.set(false);
                        }
                    }
                    Err(_) => {
                        let still_active = state.read().marketplace_preview.as_ref().map(|s| &s.id) == Some(&id_c);
                        if still_active {
                            preview_content.set(Some(t("toast.load_failed", locale).to_string()));
                            preview_loading.set(false);
                        }
                    }
                }
            });
            spawn(async move {
                if let Ok(files) = marketplace::fetch_full_tree(&sk2).await {
                    mp_cache.write().set_drawer_files(&id_c2, files.clone());
                    let still_active = state.read().marketplace_preview.as_ref().map(|s| &s.id) == Some(&id_c2);
                    if still_active {
                        preview_files.set(Some(files));
                    }
                }
            });
        }
    });


    let author = skill.author.clone().unwrap_or_else(|| t("marketplace.unknown_author", locale).to_string());
    let stars = skill.stars.unwrap_or(0);
    let forks = skill.forks.unwrap_or(0);
    let date_str = skill.formatted_date().unwrap_or_default();
    let desc = skill.description.clone().unwrap_or_default();
    let install_skill = skill.clone();
    let is_closing = *closing.read();

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
                        state.write().marketplace_preview = None;
                        closing.set(false);
                    });
                }
            },
        }
        // Drawer panel
        div {
            id: "skill-detail-scroll",
            class: if is_closing {
                "fixed top-0 right-0 bottom-0 z-50 w-full max-w-full sm:max-w-lg md:max-w-xl lg:max-w-2xl bg-gray-50 dark:bg-gray-900 shadow-2xl overflow-y-auto overflow-x-hidden drawer-exit"
            } else {
                "fixed top-0 right-0 bottom-0 z-50 w-full max-w-full sm:max-w-lg md:max-w-xl lg:max-w-2xl bg-gray-50 dark:bg-gray-900 shadow-2xl overflow-y-auto overflow-x-hidden drawer-enter"
            },
            onscroll: move |_| {
                spawn(async move {
                    let mut res = document::eval(r#"
                        let el = document.getElementById('skill-detail-scroll');
                        dioxus.send(el ? el.scrollTop : 0);
                    "#);
                    if let Ok(val) = res.recv::<f64>().await {
                        drawer_show_top.set(val > 200.0);
                    }
                });
            },

            // Sticky close header
            div {
                class: "sticky top-0 z-10 bg-white/90 dark:bg-gray-800/90 backdrop-blur-sm border-b border-gray-200 dark:border-gray-700 px-5 py-3.5 flex items-center justify-between",
                span { class: "text-xs font-medium text-gray-400 dark:text-gray-500 uppercase tracking-wider", {t("drawer.skill_details", locale)} }
                button {
                    class: "p-1 rounded-md text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors",
                    onclick: move |_| {
                        if !*closing.read() {
                            closing.set(true);
                            spawn(async move {
                                let mut res = document::eval(r#"setTimeout(() => dioxus.send(1), 300);"#);
                                let _ = res.recv::<f64>().await;
                                state.write().marketplace_preview = None;
                                closing.set(false);
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
                class: "px-5 py-5 space-y-5",

                // Breadcrumb
                div {
                    class: "bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 px-4 py-2 text-xs font-mono text-gray-500 dark:text-gray-400",
                    span { class: "text-green-600 dark:text-green-400", "$ pwd: " }
                    span { "~ / skills / " }
                    span { class: "text-gray-400 dark:text-gray-500", "{author}" }
                    span { " / " }
                    span { class: "text-gray-800 dark:text-gray-100 font-semibold", "{skill.name}" }
                }

                // Title + description
                div {
                    h1 {
                        class: "text-2xl font-bold text-gray-900 dark:text-gray-100 tracking-tight mb-2",
                        "{skill.name}"
                    }
                    p {
                        class: "text-[13px] text-gray-500 dark:text-gray-400 font-mono leading-relaxed",
                        "// {desc}"
                    }
                }

                // Install button
                button {
                    class: "inline-flex items-center gap-2 px-5 py-2.5 text-sm font-medium rounded-lg bg-claude text-white hover:bg-claude-dark active:scale-[0.97] transition-all duration-150",
                    onclick: move |_| {
                        let s = install_skill.clone();
                        let files = preview_files.read().clone();
                        let content = preview_content.read().clone();
                        spawn(async move {
                            match marketplace::install_skill(&s).await {
                                Ok(name) => {
                                    let compat = marketplace::check_compatibility(
                                        files.as_deref().unwrap_or(&[]),
                                        content.as_deref().unwrap_or(""),
                                    );
                                    if let Some(warning) = compat {
                                        toast.write().success(t("toast.installed", locale).replace("{}", &name));
                                        toast.write().warning(warning);
                                    } else {
                                        toast.write().success(t("toast.installed", locale).replace("{}", &name));
                                    }
                                    state.write().reload();
                                }
                                Err(e) => {
                                    toast.write().error(e);
                                }
                            }
                        });
                    },
                    {t("drawer.install_skill", locale)}
                }

                // Stats card
                div {
                    class: "bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 overflow-hidden",
                    div {
                        class: "px-4 py-2 border-b border-gray-100 dark:border-gray-700 bg-gray-50/50 dark:bg-gray-800/50 text-xs font-mono text-gray-500 dark:text-gray-400",
                        span { class: "text-green-600 dark:text-green-400", "$ " }
                        "git log --oneline --stat"
                    }
                    div {
                        class: "px-4 py-3 flex flex-wrap items-center gap-4 sm:gap-6 text-sm",
                        span {
                            class: "flex items-center gap-1.5",
                            IconStar { class: "w-4 h-4 text-amber-500 icon-animated" }
                            span { class: "text-gray-500 dark:text-gray-400", {t("drawer.stars", locale)} }
                            span { class: "font-semibold text-gray-800 dark:text-gray-100", "{stars}" }
                        }
                        span {
                            class: "flex items-center gap-1.5",
                            IconGitFork { class: "w-4 h-4 text-gray-400 dark:text-gray-500 icon-animated" }
                            span { class: "text-gray-500 dark:text-gray-400", {t("drawer.forks", locale)} }
                            span { class: "font-semibold text-gray-800 dark:text-gray-100", "{forks}" }
                        }
                        if !date_str.is_empty() {
                            span {
                                class: "flex items-center gap-1.5",
                                IconCalendar { class: "w-4 h-4 text-green-500 icon-animated" }
                                span { class: "text-gray-500 dark:text-gray-400", {t("drawer.updated", locale)} }
                                span { class: "font-semibold text-gray-800 dark:text-gray-100", "{date_str}" }
                            }
                        }
                    }
                }

                // File Explorer card
                if preview_files.read().is_none() {
                    div {
                        class: "bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 overflow-hidden",
                        div {
                            class: "flex items-center justify-between px-4 py-2 border-b border-gray-100 dark:border-gray-700 bg-gray-50/50 dark:bg-gray-800/50",
                            div { class: "flex items-center gap-2",
                                div { class: "flex items-center gap-1.5",
                                    div { class: "w-2.5 h-2.5 rounded-full bg-gray-200 dark:bg-gray-600" }
                                    div { class: "w-2.5 h-2.5 rounded-full bg-gray-200 dark:bg-gray-600" }
                                    div { class: "w-2.5 h-2.5 rounded-full bg-gray-200 dark:bg-gray-600" }
                                }
                                div { class: "skeleton h-3 w-20 ml-1" }
                            }
                            div { class: "skeleton h-3 w-12" }
                        }
                        div {
                            class: "px-4 py-3 space-y-2",
                            div { class: "flex items-center gap-2",
                                div { class: "skeleton h-3 w-3" }
                                div { class: "skeleton h-3 w-3 rounded-sm" }
                                div { class: "skeleton h-3 w-24" }
                            }
                            div { class: "flex items-center gap-2 pl-5",
                                div { class: "skeleton h-3 w-3 rounded-sm" }
                                div { class: "skeleton h-3 w-20" }
                                div { class: "skeleton h-3 w-10 ml-auto" }
                            }
                            div { class: "flex items-center gap-2 pl-5",
                                div { class: "skeleton h-3 w-3 rounded-sm" }
                                div { class: "skeleton h-3 w-28" }
                                div { class: "skeleton h-3 w-10 ml-auto" }
                            }
                            div { class: "flex items-center gap-2 pl-5",
                                div { class: "skeleton h-3 w-3 rounded-sm" }
                                div { class: "skeleton h-3 w-16" }
                                div { class: "skeleton h-3 w-10 ml-auto" }
                            }
                        }
                    }
                }
                if let Some(files) = preview_files.read().clone() {
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
                            {
                                let fc = files.iter().filter(|f| f.file_type == "file").count();
                                let files_word = t("card.files", locale);
                                rsx! { span { class: "text-[10px] text-gray-400 dark:text-gray-500", "{fc} {files_word}" } }
                            }
                        }
                        {
                            let expanded = expanded_dirs.read().clone();
                            let visible: Vec<&GithubFile> = files.iter().filter(|f| {
                                let depth = f.path.matches('/').count();
                                if depth == 0 { return true; }
                                let parent = f.path.rsplitn(2, '/').nth(1).unwrap_or("");
                                let parts: Vec<&str> = parent.split('/').collect();
                                parts.iter().enumerate().all(|(i, _)| {
                                    let ancestor = parts[..=i].join("/");
                                    expanded.contains(&ancestor)
                                })
                            }).collect();

                            rsx! {
                                div {
                                    class: "px-4 py-2",
                                    for file in visible {
                                        {
                                            let depth = file.path.matches('/').count();
                                            let indent = format!("{}rem", depth as f32 * 1.2 + 0.25);
                                            let is_dir = file.file_type == "dir";
                                            let is_skill_md = file.name == "SKILL.md";
                                            let is_selected = selected_file.read().as_ref() == Some(&file.path);
                                            let is_expanded = is_dir && expanded.contains(&file.path);
                                            let row_bg = if is_selected { "bg-orange-50 dark:bg-orange-900/20 rounded" } else { "hover:bg-gray-50 dark:hover:bg-gray-700/50 rounded" };
                                            let text_class = if is_selected { "font-mono text-xs text-orange-500 font-medium" } else { "font-mono text-xs text-gray-700 dark:text-gray-300" };
                                            let dl_url = file.download_url.clone();
                                            let file_path_str = file.path.clone();
                                            let dir_path = file.path.clone();
                                            let file_name = file.name.clone();

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
                                                        } else if let Some(ref url) = dl_url {
                                                            let fp = file_path_str.clone();
                                                            selected_file.set(Some(fp.clone()));
                                                            if !file_cache.read().contains_key(&fp) {
                                                                let url = url.clone();
                                                                spawn(async move {
                                                                    match marketplace::fetch_github_file_content(&url).await {
                                                                        Ok(c) => { file_cache.write().insert(fp, c); }
                                                                        Err(e) => { file_cache.write().insert(fp, format!("Error: {e}")); }
                                                                    }
                                                                });
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

                    {
                        let sel = selected_file.read().clone();
                        let show_other = sel.as_ref().is_some_and(|s| s != "SKILL.md");
                        if show_other {
                            let Some(sel_name) = sel else { return rsx! {} };
                            if let Some(content) = file_cache.read().get(&sel_name).cloned() {
                                let line_count = content.lines().count();
                                let is_md = sel_name.ends_with(".md");
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
                                                span { class: "text-xs font-medium text-gray-600 dark:text-gray-300 ml-1", "{sel_name}" }
                                            }
                                            span { class: "text-[10px] text-gray-400 dark:text-gray-500", {t("drawer.readonly", locale)} }
                                        }
                                        div {
                                            class: "p-4",
                                            p { class: "text-[10px] text-gray-400 dark:text-gray-500 text-right mb-2", "{line_count} {lines_word}" }
                                            if is_md {
                                                div {
                                                    class: "prose prose-sm dark:prose-invert max-w-none text-gray-700 dark:text-gray-200 overflow-x-auto break-words",
                                                    dangerous_inner_html: "{markdown::md_to_html(&content)}",
                                                }
                                            } else {
                                                pre {
                                                    class: "text-xs text-gray-700 dark:text-gray-200 whitespace-pre-wrap break-words",
                                                    code { "{content}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                rsx! {
                                    div {
                                        class: "bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 overflow-hidden",
                                        div {
                                            class: "flex items-center justify-between px-4 py-2 border-b border-gray-100 dark:border-gray-700 bg-gray-50/50 dark:bg-gray-800/50",
                                            div { class: "flex items-center gap-2",
                                                div { class: "flex items-center gap-1.5",
                                                    div { class: "w-2.5 h-2.5 rounded-full bg-gray-200 dark:bg-gray-600" }
                                                    div { class: "w-2.5 h-2.5 rounded-full bg-gray-200 dark:bg-gray-600" }
                                                    div { class: "w-2.5 h-2.5 rounded-full bg-gray-200 dark:bg-gray-600" }
                                                }
                                                div { class: "skeleton h-3 w-24" }
                                            }
                                        }
                                        div {
                                            class: "p-4 space-y-2",
                                            div { class: "skeleton h-3 w-full" }
                                            div { class: "skeleton h-3 w-5/6" }
                                            div { class: "skeleton h-3 w-4/5" }
                                            div { class: "skeleton h-3 w-2/3" }
                                        }
                                    }
                                }
                            }
                        } else {
                            rsx! {}
                        }
                    }
                }

                // SKILL.md content card
                if selected_file.read().as_deref() == Some("SKILL.md") || selected_file.read().is_none() {
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
                                span { class: "text-xs font-medium text-gray-600 dark:text-gray-300 ml-1", "SKILL.md" }
                            }
                            span { class: "text-[10px] text-gray-400 dark:text-gray-500", {t("drawer.readonly", locale)} }
                        }

                        div {
                            class: "p-4",
                            div {
                                class: "rounded-lg border border-gray-100 dark:border-gray-700 overflow-hidden mb-4",
                                div {
                                    class: "flex border-b border-gray-100 dark:border-gray-700",
                                    div { class: "w-28 px-3 py-2 bg-orange-50 dark:bg-orange-900/20 text-xs font-semibold text-orange-500 flex-shrink-0", "name" }
                                    div { class: "px-3 py-2 text-sm text-gray-700 dark:text-gray-200 flex-1", "{skill.name}" }
                                }
                                div {
                                    class: "flex",
                                    div { class: "w-28 px-3 py-2 bg-orange-50 dark:bg-orange-900/20 text-xs font-semibold text-orange-500 flex-shrink-0", "description" }
                                    div { class: "px-3 py-2 text-sm text-gray-700 dark:text-gray-200 flex-1 leading-relaxed", "{desc}" }
                                }
                            }

                            if *preview_loading.read() {
                                div {
                                    class: "space-y-3 py-2",
                                    div { class: "skeleton h-4 w-3/4" }
                                    div { class: "skeleton h-4 w-full" }
                                    div { class: "skeleton h-4 w-5/6" }
                                    div { class: "skeleton h-4 w-2/3" }
                                    div { class: "skeleton h-4 w-full mt-4" }
                                    div { class: "skeleton h-4 w-4/5" }
                                    div { class: "skeleton h-4 w-3/5" }
                                }
                            } else if let Some(content) = preview_content.read().clone() {
                                div {
                                    class: "prose prose-sm dark:prose-invert max-w-none text-gray-700 dark:text-gray-200 overflow-x-auto break-words",
                                    dangerous_inner_html: "{markdown::md_to_html(&content)}",
                                }
                            }
                        }
                    }
                }
            }

            // Back to top button
            div {
                class: if *drawer_show_top.read() { "scroll-top-wrap scroll-top-visible sticky bottom-0 w-full flex justify-center p-4" } else { "scroll-top-wrap scroll-top-hidden sticky bottom-0 w-full flex justify-center p-4" },
                button {
                    class: "scroll-top-btn flex items-center gap-2 px-5 py-2.5 text-sm font-mono text-gray-800 dark:text-gray-200 border border-claude-light/40 rounded-full shadow-lg",
                    style: "background: rgba(255,255,255,0.72); backdrop-filter: blur(16px); -webkit-backdrop-filter: blur(16px);",
                    onclick: move |_| {
                        document::eval(r#"document.getElementById('skill-detail-scroll').scrollTo({top: 0, behavior: 'smooth'})"#);
                    },
                    IconTerminal { class: "w-4 h-4 text-green-500" }
                    "cd"
                    IconArrowUp { class: "w-4 h-4 text-claude arrow-up" }
                    "top"
                }
            }
        }
    }
}
