use dioxus::prelude::*;
use crate::hooks::{SkillsState, AppView};
use crate::models::{SkillFilter, SkillSource};
use crate::components::icons::ClaudeMascot;
use crate::theme::{AppSettings, Theme, Locale, t};

#[component]
pub fn Sidebar(on_navigate: EventHandler<()>) -> Element {
    let mut state = use_context::<Signal<SkillsState>>();
    let mut settings = use_context::<Signal<AppSettings>>();
    let locale = settings.read().locale;
    let theme = settings.read().theme;
    let current_view = state.read().view.clone();
    let current_filter = state.read().filter.clone();

    let all_count = state.read().skills.len();
    let enabled_count = state.read().skills.iter().filter(|s| s.enabled).count();
    let disabled_count = all_count - enabled_count;
    let custom_count = state.read().skills.iter().filter(|s| s.source == SkillSource::Custom).count();
    let downloaded_count = state.read().skills.iter().filter(|s| s.source != SkillSource::Custom).count();
    let linked_count = state.read().skills.iter().filter(|s| s.source == SkillSource::Linked).count();
    let other_count = state.read().skills.iter().filter(|s| s.source == SkillSource::Downloaded || s.source == SkillSource::LooseFile).count();

    rsx! {
        div {
            class: "w-56 flex-shrink-0 bg-gray-100 dark:bg-gray-900 border-r border-gray-200 dark:border-gray-700 p-3 flex flex-col gap-0.5 overflow-y-auto h-full",

            // Claude mascot branding
            div {
                class: "flex items-center gap-2.5 px-2 py-3 mb-1",
                div {
                    class: "relative flex-shrink-0",
                    ClaudeMascot { class: "w-9 drop-shadow-sm" }
                }
                div {
                    class: "flex flex-col min-w-0",
                    span {
                        class: "text-[13px] font-semibold text-gray-800 dark:text-gray-100 leading-tight",
                        "SkillBox"
                    }
                    span {
                        class: "text-[10px] text-gray-400 dark:text-gray-500 leading-tight mt-0.5",
                        "for Claude Code"
                    }
                }
            }

            div { class: "mb-1 border-t border-gray-200 dark:border-gray-700" }

            // Dashboard
            SidebarItem {
                label: t("sidebar.dashboard", locale),
                count: 0,
                active: current_view == AppView::Dashboard,
                icon: "M3 3v18h18M9 17V9m4 8V5m4 12v-4",
                on_click: move |_| { state.write().view = AppView::Dashboard; on_navigate.call(()); },
            }

            // Marketplace
            SidebarItem {
                label: t("sidebar.marketplace", locale),
                count: 0,
                active: current_view == AppView::Marketplace,
                icon: "M3 3h2l.4 2M7 13h10l4-8H5.4M7 13L5.4 5M7 13l-2.293 2.293c-.63.63-.184 1.707.707 1.707H17m0 0a2 2 0 100 4 2 2 0 000-4zm-8 2a2 2 0 100 4 2 2 0 000-4z",
                on_click: move |_| { state.write().view = AppView::Marketplace; on_navigate.call(()); },
            }

            div { class: "my-2 border-t border-gray-200 dark:border-gray-700" }

            // Section: Views
            SidebarSection { label: t("sidebar.library", locale) }
            SidebarItem {
                label: t("sidebar.all_skills", locale),
                count: all_count,
                active: current_view == AppView::Skills && current_filter == SkillFilter::All,
                icon: "M4 6h16M4 12h16M4 18h16",
                on_click: move |_| { let mut s = state.write(); s.view = AppView::Skills; s.filter = SkillFilter::All; on_navigate.call(()); },
            }
            SidebarItem {
                label: t("sidebar.enabled", locale),
                count: enabled_count,
                active: current_view == AppView::Skills && current_filter == SkillFilter::Enabled,
                icon: "M5 13l4 4L19 7",
                on_click: move |_| { let mut s = state.write(); s.view = AppView::Skills; s.filter = SkillFilter::Enabled; on_navigate.call(()); },
            }
            SidebarItem {
                label: t("sidebar.disabled", locale),
                count: disabled_count,
                active: current_view == AppView::Skills && current_filter == SkillFilter::Disabled,
                icon: "M18.36 6.64A9 9 0 0 1 20.77 12 9 9 0 1 1 5.64 6.64M12 2v10",
                on_click: move |_| { let mut s = state.write(); s.view = AppView::Skills; s.filter = SkillFilter::Disabled; on_navigate.call(()); },
            }

            // Divider
            div { class: "my-2 border-t border-gray-200 dark:border-gray-700" }

            // Section: Source
            SidebarSection { label: t("sidebar.source", locale) }
            SidebarItem {
                label: t("sidebar.my_skills", locale),
                count: custom_count,
                active: current_view == AppView::Skills && current_filter == SkillFilter::Custom,
                icon: "M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z",
                on_click: move |_| { let mut s = state.write(); s.view = AppView::Skills; s.filter = SkillFilter::Custom; on_navigate.call(()); },
            }
            SidebarItem {
                label: t("sidebar.downloaded", locale),
                count: downloaded_count,
                active: current_view == AppView::Skills && current_filter == SkillFilter::Downloaded,
                icon: "M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4",
                on_click: move |_| { let mut s = state.write(); s.view = AppView::Skills; s.filter = SkillFilter::Downloaded; on_navigate.call(()); },
            }
            SidebarSubItem {
                label: t("sidebar.linked", locale),
                count: linked_count,
                active: current_view == AppView::Skills && current_filter == SkillFilter::Linked,
                on_click: move |_| { let mut s = state.write(); s.view = AppView::Skills; s.filter = SkillFilter::Linked; on_navigate.call(()); },
            }
            SidebarSubItem {
                label: t("sidebar.other", locale),
                count: other_count,
                active: current_view == AppView::Skills && current_filter == SkillFilter::Other,
                on_click: move |_| { let mut s = state.write(); s.view = AppView::Skills; s.filter = SkillFilter::Other; on_navigate.call(()); },
            }

            // Spacer
            div { class: "flex-1" }

            // Settings toggles at bottom
            div { class: "mt-2 pt-2 border-t border-gray-200 dark:border-gray-700 space-y-1" }

            // Theme toggle
            button {
                class: "w-full flex items-center gap-2.5 px-2.5 py-2 rounded-lg text-[13px] font-medium text-gray-600 dark:text-gray-300 hover:bg-gray-200/60 dark:hover:bg-gray-700/60 transition-all duration-200 ease-out",
                onclick: move |_| {
                    let new_theme = if theme == Theme::Light { Theme::Dark } else { Theme::Light };
                    settings.write().theme = new_theme;
                    if new_theme == Theme::Dark {
                        document::eval("document.documentElement.classList.add('dark');document.body.classList.add('dark');localStorage.setItem('theme','dark')");
                    } else {
                        document::eval("document.documentElement.classList.remove('dark');document.body.classList.remove('dark');localStorage.setItem('theme','light')");
                    }
                },
                svg {
                    class: "w-4 h-4 flex-shrink-0",
                    fill: "none",
                    stroke: "currentColor",
                    view_box: "0 0 24 24",
                    overflow: "visible",
                    if theme == Theme::Light {
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "1.5",
                            d: "M21.752 15.002A9.718 9.718 0 0118 15.75c-5.385 0-9.75-4.365-9.75-9.75 0-1.33.266-2.597.748-3.752A9.753 9.753 0 003 11.25C3 16.635 7.365 21 12.75 21a9.753 9.753 0 009.002-5.998z",
                        }
                    } else {
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "1.5",
                            d: "M12 3v2.25m6.364.386l-1.591 1.591M21 12h-2.25m-.386 6.364l-1.591-1.591M12 18.75V21m-4.773-4.227l-1.591 1.591M5.25 12H3m4.227-4.773L5.636 5.636M15.75 12a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0z",
                        }
                    }
                }
                span { class: "flex-1 text-left",
                    if theme == Theme::Light { {t("sidebar.theme_dark", locale)} } else { {t("sidebar.theme_light", locale)} }
                }
            }

            // Language toggle
            button {
                class: "w-full flex items-center gap-2.5 px-2.5 py-2 rounded-lg text-[13px] font-medium text-gray-600 dark:text-gray-300 hover:bg-gray-200/60 dark:hover:bg-gray-700/60 transition-all duration-200 ease-out",
                onclick: move |_| {
                    let new_locale = if locale == Locale::Zh { Locale::En } else { Locale::Zh };
                    settings.write().locale = new_locale;
                    if new_locale == Locale::En {
                        document::eval("localStorage.setItem('locale','en')");
                    } else {
                        document::eval("localStorage.setItem('locale','zh')");
                    }
                },
                svg {
                    class: "w-4 h-4 flex-shrink-0",
                    fill: "none",
                    stroke: "currentColor",
                    view_box: "0 0 24 24",
                    overflow: "visible",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "1.5",
                        d: "M10.5 21l5.25-11.25L21 21m-9-3h7.5M3 5.621a48.474 48.474 0 016-.371m0 0c1.12 0 2.233.038 3.334.114M9 5.25V3m3.334 2.364C11.176 10.658 7.69 15.08 3 17.502m9.334-12.138c.896.061 1.785.147 2.666.257m-4.589 8.495a18.023 18.023 0 01-3.827-5.802",
                    }
                }
                span { class: "flex-1 text-left",
                    if locale == Locale::Zh { "English" } else { "中文" }
                }
            }

        }
    }
}

#[component]
fn SidebarSection(label: &'static str) -> Element {
    rsx! {
        p {
            class: "text-[10px] font-semibold text-gray-400 dark:text-gray-500 uppercase tracking-widest px-2.5 pt-3 pb-1.5",
            "{label}"
        }
    }
}

#[component]
fn SidebarItem(
    label: &'static str,
    count: usize,
    active: bool,
    icon: &'static str,
    on_click: EventHandler<()>,
) -> Element {
    let bg = if active {
        "bg-white dark:bg-gray-800 shadow-sm text-gray-900 dark:text-gray-100"
    } else {
        "text-gray-600 dark:text-gray-300 hover:bg-gray-200/60 dark:hover:bg-gray-700/60"
    };
    rsx! {
        button {
            class: "w-full flex items-center gap-2.5 px-2.5 py-2 rounded-lg text-[13px] font-medium transition-all duration-200 ease-out {bg}",
            onclick: move |_| on_click.call(()),
            svg {
                class: "w-4 h-4 flex-shrink-0",
                fill: "none",
                stroke: "currentColor",
                view_box: "0 0 24 24",
                overflow: "visible",
                path {
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    stroke_width: "1.5",
                    d: "{icon}",
                }
            }
            span { class: "flex-1 text-left", "{label}" }
            if count > 0 {
                span {
                    class: "text-xs text-gray-400 dark:text-gray-500 tabular-nums",
                    "{count}"
                }
            }
        }
    }
}

#[component]
fn SidebarSubItem(
    label: &'static str,
    count: usize,
    active: bool,
    on_click: EventHandler<()>,
) -> Element {
    let bg = if active {
        "bg-white dark:bg-gray-800 shadow-sm text-gray-900 dark:text-gray-100"
    } else {
        "text-gray-500 dark:text-gray-400 hover:bg-gray-200/60 dark:hover:bg-gray-700/60"
    };
    rsx! {
        button {
            class: "w-full flex items-center gap-2 pl-9 pr-2.5 py-1.5 rounded-lg text-[13px] transition-all duration-200 ease-out {bg}",
            onclick: move |_| on_click.call(()),
            span { class: "flex-1 text-left", "{label}" }
            span {
                class: "text-xs text-gray-400 dark:text-gray-500 tabular-nums",
                "{count}"
            }
        }
    }
}
