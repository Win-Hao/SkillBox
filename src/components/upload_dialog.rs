use dioxus::prelude::*;
use dioxus::html::HasFileData;
use std::path::PathBuf;
use crate::hooks::SkillsState;
use crate::models::SkillFrontmatter;
use crate::services::{markdown, skill_io};
use crate::components::toast::ToastManager;
use crate::theme::{AppSettings, t};

struct PreviewData {
    frontmatter: SkillFrontmatter,
    body: String,
    file_path: PathBuf,
    file_date: String,
}

#[component]
pub fn UploadDialog(on_close: EventHandler<()>) -> Element {
    let mut state = use_context::<Signal<SkillsState>>();
    let mut toast = use_context::<Signal<ToastManager>>();
    let settings = use_context::<Signal<AppSettings>>();
    let locale = settings.read().locale;
    let mut closing = use_signal(|| false);
    let mut dragging = use_signal(|| false);
    let mut preview = use_signal(|| None::<PreviewData>);
    let mut show_raw = use_signal(|| false);
    let mut entered = use_signal(|| false);

    use_effect(move || {
        spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            entered.set(true);
        });
    });

    let trigger_close = move |_| {
        if *closing.read() { return; }
        closing.set(true);
        let on_close = on_close.clone();
        spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(320)).await;
            on_close.call(());
        });
    };

    let mut load_preview = move |path: PathBuf| {
        match skill_io::preview_skill(&path) {
            Ok((fm, body)) => {
                let file_date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                preview.set(Some(PreviewData { frontmatter: fm, body, file_path: path, file_date }));
            }
            Err(e) => {
                toast.write().error(e);
            }
        }
    };

    let mut handle_install = move |path: PathBuf| {
        match skill_io::upload_skill(&path) {
            Ok(name) => {
                state.write().reload();
                preview.set(None);
                toast.write().success(t("toast.installed", locale).replace("{}", &name));
            }
            Err(e) => {
                toast.write().error(e);
            }
        }
    };

    let do_upload = move |_| {
        let file = rfd::FileDialog::new()
            .add_filter("Skill files", &["md", "zip"])
            .pick_file();

        if let Some(path) = file {
            load_preview(path);
        }
    };

    let is_closing = *closing.read();
    let is_dragging = *dragging.read();
    let has_preview = preview.read().is_some();
    let backdrop_class = if is_closing {
        "fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-exit-slow"
    } else {
        "fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-enter"
    };

    let dialog_anim = if is_closing {
        " dialog-exit"
    } else if *entered.read() {
        ""
    } else {
        " dialog-enter"
    };
    let dialog_max_w = if has_preview { "max-w-3xl" } else { "max-w-lg" };
    let dialog_flex = if has_preview { " flex flex-col" } else { "" };
    let dialog_class = format!("bg-white dark:bg-gray-800 rounded-2xl shadow-xl {dialog_max_w} w-full mx-4 overflow-hidden{dialog_anim}{dialog_flex}");
    let dialog_style = if has_preview { "max-height: 80vh" } else { "" };

    let dropzone_class = if is_dragging {
        "border-2 border-dashed border-claude bg-claude-50/50 dark:bg-claude-dark/20 rounded-xl p-10 mb-5 flex flex-col items-center justify-center cursor-pointer transition-colors"
    } else {
        "border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-xl p-10 mb-5 flex flex-col items-center justify-center cursor-pointer hover:border-claude hover:bg-claude-50/50 dark:hover:bg-claude-dark/20 transition-colors"
    };

    let icon_class = if is_dragging {
        "w-10 h-10 text-claude mb-2"
    } else {
        "w-10 h-10 text-gray-300 dark:text-gray-600 mb-2"
    };

    rsx! {
        div {
            class: backdrop_class,
            onclick: trigger_close,
            div {
                class: dialog_class,
                style: dialog_style,
                onclick: move |evt| evt.stop_propagation(),

                // Header
                div {
                    class: "flex items-center justify-between px-5 py-3 border-b border-gray-200 dark:border-gray-700 bg-gray-50/50 dark:bg-gray-800/50",
                    div {
                        class: "flex items-center gap-2",
                        if has_preview {
                            button {
                                class: "p-1 rounded-md text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors",
                                onclick: move |_| {
                                    preview.set(None);
                                    show_raw.set(false);
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
                                        d: "M15 19l-7-7 7-7",
                                    }
                                }
                            }
                        }
                        span {
                            class: "text-base font-semibold text-gray-800 dark:text-gray-100",
                            if has_preview { {t("upload.preview_title", locale)} } else { {t("upload.title", locale)} }
                        }
                    }
                    button {
                        class: "p-1 rounded-md text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors",
                        onclick: trigger_close,
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

                if has_preview {
                    // Preview mode: extract data from signal before entering rsx
                    {
                        let preview_data = {
                            let p = preview.read();
                            p.as_ref().map(|data| (
                                data.frontmatter.clone(),
                                data.body.clone(),
                                data.file_path.clone(),
                                data.file_date.clone(),
                            ))
                        };
                        if let Some((fm, body, install_path, file_date)) = preview_data {
                            let body_html = markdown::md_to_html(&body);
                        let is_raw = *show_raw.read();
                        let raw_body = body.clone();

                        rsx! {
                            // Metadata section (fixed top)
                            div {
                                class: "step-enter px-6 pt-5 pb-4 border-b border-gray-200 dark:border-gray-700 space-y-3",
                                    // Name
                                    div {
                                        class: "flex items-baseline gap-2",
                                        h2 { class: "text-lg font-semibold text-gray-900 dark:text-gray-100 tracking-tight", "{fm.name}" }
                                        if let Some(ref v) = fm.version {
                                            span { class: "text-[11px] font-mono text-gray-400 dark:text-gray-500", "v{v}" }
                                        }
                                    }
                                    // Info row: Added by / Last updated / Tier etc
                                    div {
                                        class: "flex",
                                        div {
                                            class: "pr-6",
                                            span { class: "text-[11px] font-medium text-gray-400 dark:text-gray-500 uppercase tracking-wider", {t("upload.added_by", locale)} }
                                            p { class: "text-sm font-medium text-gray-800 dark:text-gray-100 mt-0.5", {t("upload.you", locale)} }
                                        }
                                        div {
                                            class: "px-6",
                                            span { class: "text-[11px] font-medium text-gray-400 dark:text-gray-500 uppercase tracking-wider", {t("upload.last_updated", locale)} }
                                            p { class: "text-sm font-medium text-gray-800 dark:text-gray-100 mt-0.5", "{file_date}" }
                                        }
                                        if let Some(ref tier) = fm.preamble_tier {
                                            div {
                                                class: "px-6",
                                                span { class: "text-[11px] font-medium text-gray-400 dark:text-gray-500 uppercase tracking-wider", {t("upload.tier", locale)} }
                                                p { class: "text-sm font-medium text-gray-800 dark:text-gray-100 mt-0.5", "{tier}" }
                                            }
                                        }
                                        if let Some(ref license) = fm.license {
                                            div {
                                                class: "px-6",
                                                span { class: "text-[11px] font-medium text-gray-400 dark:text-gray-500 uppercase tracking-wider", {t("upload.license", locale)} }
                                                p { class: "text-sm font-medium text-gray-800 dark:text-gray-100 mt-0.5", "{license}" }
                                            }
                                        }
                                    }
                                    // Description
                                    div {
                                        span { class: "text-[11px] font-medium text-gray-400 dark:text-gray-500 uppercase tracking-wider", {t("upload.description", locale)} }
                                        p { class: "text-sm text-gray-600 dark:text-gray-300 leading-relaxed mt-0.5", "{fm.description}" }
                                    }
                                }

                            // Content area (scrollable)
                            div {
                                class: "step-enter flex-1 overflow-y-auto px-6 py-4",
                                div {
                                    class: "rounded-xl border border-gray-200 dark:border-gray-700 overflow-hidden",

                                    // Mac-style card header with view toggle
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
                                        div {
                                            class: "flex items-center border border-gray-200 dark:border-gray-600 rounded-lg overflow-hidden",
                                            button {
                                                class: if !is_raw { "px-2.5 py-1 text-xs font-medium bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-200" } else { "px-2.5 py-1 text-xs font-medium text-gray-400 dark:text-gray-500 hover:text-gray-600 dark:hover:text-gray-300" },
                                                onclick: move |_| show_raw.set(false),
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
                                                        d: "M15 12a3 3 0 11-6 0 3 3 0 016 0z",
                                                    }
                                                    path {
                                                        stroke_linecap: "round",
                                                        stroke_linejoin: "round",
                                                        stroke_width: "2",
                                                        d: "M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z",
                                                    }
                                                }
                                            }
                                            button {
                                                class: if is_raw { "px-2.5 py-1 text-xs font-medium bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-200" } else { "px-2.5 py-1 text-xs font-medium text-gray-400 dark:text-gray-500 hover:text-gray-600 dark:hover:text-gray-300" },
                                                onclick: move |_| show_raw.set(true),
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
                                                        d: "M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4",
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    if is_raw {
                                        div {
                                            class: "px-5 py-4 overflow-x-auto",
                                            pre {
                                                class: "text-[13px] font-mono text-gray-700 dark:text-gray-200 leading-relaxed",
                                                code { "{raw_body}" }
                                            }
                                        }
                                    } else {
                                        div {
                                            class: "px-5 py-4",
                                            if !body.is_empty() {
                                                div {
                                                    class: "prose prose-sm dark:prose-invert max-w-none text-gray-600 dark:text-gray-300 leading-relaxed overflow-x-auto break-words",
                                                    dangerous_inner_html: "{body_html}",
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Install footer
                            div {
                                class: "step-enter px-6 py-4 border-t border-gray-200 dark:border-gray-700 flex items-center justify-end",

                                button {
                                    class: "px-5 py-2 text-sm font-medium rounded-lg bg-claude text-white hover:bg-claude-dark active:scale-[0.97] transition-colors",
                                    onclick: move |_| {
                                        handle_install(install_path.clone());
                                    },
                                    {t("upload.install", locale)}
                                }
                            }
                        }
                        } else {
                            rsx! {}
                        }
                    }
                } else {
                    // Upload / drop zone mode
                    div {
                        class: "step-enter p-6",
                        div {
                            class: dropzone_class,
                            onclick: do_upload,
                            ondragover: move |evt: DragEvent| {
                                evt.prevent_default();
                                dragging.set(true);
                            },
                            ondragleave: move |_| {
                                dragging.set(false);
                            },
                            ondrop: move |evt: DragEvent| {
                                evt.prevent_default();
                                dragging.set(false);
                                let files = evt.data().files();
                                if let Some(file) = files.first() {
                                    let path: std::path::PathBuf = file.path();
                                    let ext = path.extension()
                                        .and_then(|e: &std::ffi::OsStr| e.to_str())
                                        .unwrap_or("")
                                        .to_lowercase();
                                    if ext == "md" || ext == "zip" {
                                        load_preview(path);
                                    } else {
                                        toast.write().warning(t("toast.unsupported_file", locale));
                                    }
                                }
                            },
                            svg {
                                class: icon_class,
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                overflow: "visible",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "1.5",
                                    d: "M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12",
                                }
                            }
                            p {
                                class: if is_dragging { "text-sm text-claude font-medium" } else { "text-sm text-gray-400 dark:text-gray-500" },
                                if is_dragging { {t("upload.drop_release", locale)} } else { {t("upload.drop_hint", locale)} }
                            }
                        }

                        div {
                            class: "text-xs text-gray-400 dark:text-gray-500",
                            p { class: "font-medium mb-1.5", {t("upload.requirements", locale)} }
                            ul {
                                class: "list-disc list-inside space-y-0.5",
                                li { {t("upload.req_md", locale)} }
                                li { {t("upload.req_zip", locale)} }
                            }
                        }
                    }
                }
            }
        }
    }
}
