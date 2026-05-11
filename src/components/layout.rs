use dioxus::prelude::*;
use crate::hooks::{SkillsState, AppView};
use crate::components::sidebar::Sidebar;
use crate::components::skill_drawer::SkillDrawer;
use crate::components::marketplace::MarketplaceView;
use crate::components::marketplace_drawer::MarketplaceDrawer;
use crate::components::dashboard::DashboardView;
use crate::components::icons::{IconTerminal, IconArrowUp};
use crate::components::upload_dialog::UploadDialog;
use crate::components::toast::ToastFrame;

#[component]
pub fn Layout() -> Element {
    let mut state = use_context::<Signal<SkillsState>>();
    let has_detail = state.read().detail_skill.is_some();
    let has_marketplace_preview = state.read().marketplace_preview.is_some();
    let show_upload = state.read().show_upload;

    let current_view = state.read().view.clone();
    let mut show_scroll_top = use_signal(|| false);
    let mut sidebar_open = use_signal(|| false);

    let page_key = use_memo(move || {
        let view = state.read().view.clone();
        let filter = state.read().filter.label();
        match view {
            AppView::Skills => format!("skills-{}", filter),
            AppView::Marketplace => "marketplace".to_string(),
            AppView::Dashboard => "dashboard".to_string(),
        }
    });

    use_effect(move || {
        let _ = page_key.read();
        document::eval(r#"
            let el = document.getElementById('page-content');
            if (el) {
                el.classList.remove('page-transition');
                void el.offsetWidth;
                el.classList.add('page-transition');
            }
        "#);
    });

    rsx! {
        style { ".scroll-top-btn {{ transition: border-color 0.2s ease, box-shadow 0.2s ease; }} .scroll-top-btn:hover {{ border-color: rgba(217,119,87,0.55); box-shadow: 0 4px 12px rgba(217,119,87,0.15); }} .scroll-top-btn .arrow-up {{ transition: transform 0.25s ease; }} .scroll-top-btn:hover .arrow-up {{ transform: translateY(-2px); }}" }
        div {
            class: "flex h-screen bg-gray-50 dark:bg-gray-900 relative",

            // Mobile sidebar overlay
            div {
                class: if *sidebar_open.read() {
                    "fixed inset-0 z-30 bg-black/30 lg:hidden opacity-100 transition-opacity duration-300 ease-in-out"
                } else {
                    "fixed inset-0 z-30 bg-black/30 lg:hidden opacity-0 pointer-events-none transition-opacity duration-300 ease-in-out"
                },
                onclick: move |_| sidebar_open.set(false),
            }

            // Sidebar — hidden on mobile, slide-in via toggle
            div {
                class: if *sidebar_open.read() {
                    "fixed inset-y-0 left-0 z-40 w-56 lg:static lg:z-auto transition-transform duration-200"
                } else {
                    "fixed inset-y-0 left-0 z-40 w-56 -translate-x-full lg:translate-x-0 lg:static lg:z-auto transition-transform duration-200"
                },
                Sidebar {
                    on_navigate: move |_| sidebar_open.set(false),
                }
            }

            div {
                class: "flex-1 flex flex-col overflow-hidden relative min-w-0",

                // Mobile hamburger
                div {
                    class: "lg:hidden sticky top-0 z-20 bg-gray-50/80 dark:bg-gray-900/80 backdrop-blur-sm border-b border-gray-200 dark:border-gray-700 px-3 py-2",
                    button {
                        class: "p-1.5 rounded-lg hover:bg-gray-200/60 dark:hover:bg-gray-700/60 transition-colors",
                        onclick: move |_| sidebar_open.toggle(),
                        svg {
                            class: "w-5 h-5 text-gray-600 dark:text-gray-300",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            overflow: "visible",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "1.5",
                                d: "M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5",
                            }
                        }
                    }
                }

                div {
                    id: "main-scroll",
                    class: "flex-1 overflow-y-auto overflow-x-hidden",
                    onscroll: move |_| {
                        spawn(async move {
                            let mut res = document::eval(r#"
                                let el = document.getElementById('main-scroll');
                                dioxus.send(el ? el.scrollTop : 0);
                            "#);
                            if let Ok(val) = res.recv::<f64>().await {
                                show_scroll_top.set(val > 200.0);
                            }
                        });
                    },
                    div {
                        id: "page-content",
                        class: "pb-24",
                        match current_view {
                            AppView::Skills => rsx! { Outlet::<crate::routes::Route> {} },
                            AppView::Marketplace => rsx! { MarketplaceView {} },
                            AppView::Dashboard => rsx! { DashboardView {} },
                        }
                    }
                }
                div {
                    id: "layout-scroll-top",
                    class: if *show_scroll_top.read() { "scroll-top-wrap scroll-top-visible" } else { "scroll-top-wrap scroll-top-hidden" },
                    style: "position: absolute; bottom: 1.5rem; left: 0; right: 0; z-index: 30; display: flex; justify-content: center;",
                    button {
                        class: "scroll-top-btn flex items-center gap-2 px-5 py-2.5 text-sm font-mono text-gray-800 dark:text-gray-200 border border-claude-light/40 rounded-full shadow-lg",
                        style: "background: rgba(255,255,255,0.72); backdrop-filter: blur(16px); -webkit-backdrop-filter: blur(16px);",
                        onclick: move |_| {
                            document::eval(r#"document.getElementById('main-scroll').scrollTo({top: 0, behavior: 'smooth'})"#);
                        },
                        IconTerminal { class: "w-4 h-4 text-green-500" }
                        "cd"
                        IconArrowUp { class: "w-4 h-4 text-claude arrow-up" }
                        "top"
                    }
                }
            }
            if has_detail {
                SkillDrawer {}
            }
            if has_marketplace_preview {
                MarketplaceDrawer {}
            }
        }
        if show_upload {
            UploadDialog {
                on_close: move |_| state.write().show_upload = false,
            }
        }
        ToastFrame {}
    }
}
