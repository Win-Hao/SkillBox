use dioxus::prelude::*;
use crate::hooks::{SkillsState, MarketplaceCache};
use crate::components::toast::ToastManager;
use crate::components::icons::ClaudeMascot;
use crate::routes::Route;
use crate::services::scanner::skills_base_dir;
use crate::theme::{AppSettings, Theme, Locale, t};

pub fn App() -> Element {
    let mut state = use_context_provider(|| Signal::new(SkillsState::new()));
    use_context_provider(|| Signal::new(MarketplaceCache::new()));
    use_context_provider(|| Signal::new(ToastManager::new()));
    let mut settings = use_context_provider(|| Signal::new(AppSettings::new()));
    let mut show_welcome = use_signal(|| false);

    use_effect(move || {
        spawn(async move {
            let mut res = document::eval(r#"
                var t = localStorage.getItem('theme') || 'light';
                var l = localStorage.getItem('locale') || 'zh';
                var w = localStorage.getItem('wm_welcome') || '';
                dioxus.send(t + ',' + l + ',' + w);
            "#);
            if let Ok(val) = res.recv::<String>().await {
                let parts: Vec<&str> = val.split(',').collect();
                if parts.len() >= 2 {
                    let theme = if parts[0] == "dark" { Theme::Dark } else { Theme::Light };
                    let locale = if parts[1] == "en" { Locale::En } else { Locale::Zh };
                    let mut s = settings.write();
                    s.theme = theme;
                    s.locale = locale;
                }
                if parts.len() >= 3 && parts[2] != "1" {
                    show_welcome.set(true);
                }
            }
        });

        document::eval(r#"
            requestAnimationFrame(() => {
                let splash = document.getElementById('splash');
                let main = document.getElementById('main');
                if (main) main.classList.add('ready');
                if (splash) {
                    splash.classList.add('fade-out');
                    setTimeout(() => splash.remove(), 350);
                }
            });
        "#);
    });

    use_effect(move || {
        spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                let mut res = document::eval(
                    "dioxus.send(window.innerWidth + ',' + window.innerHeight)"
                );
                if let Ok(val) = res.recv::<String>().await {
                    let parts: Vec<&str> = val.split(',').collect();
                    if let (Some(w), Some(h)) = (
                        parts.first().and_then(|s| s.parse::<f64>().ok()),
                        parts.get(1).and_then(|s| s.parse::<f64>().ok()),
                    ) {
                        if w >= 520.0 && h >= 400.0 {
                            let dir = dirs::config_dir()
                                .unwrap_or_else(|| std::path::PathBuf::from("."))
                                .join("SkillBox");
                            let _ = std::fs::create_dir_all(&dir);
                            let json = format!("{{\"width\":{w},\"height\":{h}}}");
                            let _ = std::fs::write(dir.join("window.json"), json);
                        }
                    }
                }
            }
        });
    });

    let mut watcher_active = use_signal(|| false);

    use_effect(move || {
        if *watcher_active.peek() {
            return;
        }
        watcher_active.set(true);

        spawn(async move {
            let path = skills_base_dir();
            let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(1);

            std::thread::spawn(move || {
                use notify::{Watcher, RecursiveMode, recommended_watcher};
                let tx = tx.clone();
                let Ok(mut watcher) = recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
                    if let Ok(event) = res {
                        use notify::EventKind::*;
                        match event.kind {
                            Create(_) | Remove(_) | Modify(_) => {
                                let _ = tx.blocking_send(());
                            }
                            _ => {}
                        }
                    }
                }) else {
                    return;
                };
                let _ = watcher.watch(&path, RecursiveMode::NonRecursive);
                let disabled = path.join(".disabled");
                if disabled.exists() {
                    let _ = watcher.watch(&disabled, RecursiveMode::NonRecursive);
                }
                std::thread::park();
            });

            while let Some(()) = rx.recv().await {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                while rx.try_recv().is_ok() {}
                state.write().reload();
            }
        });
    });

    let locale = settings.read().locale;

    rsx! {
        document::Link {
            rel: "stylesheet",
            href: asset!("/assets/main.css"),
        }
        Router::<Route> {}
        if *show_welcome.read() {
            div {
                class: "fixed inset-0 z-[999999] flex items-center justify-center",
                div {
                    class: "absolute inset-0 bg-black/40",
                    style: "backdrop-filter: blur(4px); -webkit-backdrop-filter: blur(4px);",
                }
                div {
                    class: "relative bg-white dark:bg-gray-800 rounded-2xl shadow-2xl max-w-sm mx-4 overflow-hidden",
                    div {
                        class: "px-6 py-5 flex flex-col items-center gap-2",
                        style: "background: linear-gradient(135deg, #D97757, #C4684B);",
                        ClaudeMascot { class: "w-14 h-14 drop-shadow-md" }
                        div {
                            class: "text-center",
                            p { class: "text-white text-lg font-semibold", "SkillBox" }
                            p { class: "text-white/70 text-sm mt-0.5", "for Claude Code · by Winhao学AI" }
                        }
                    }
                    div {
                        class: "px-6 py-5 space-y-3",
                        p {
                            class: "text-sm text-gray-600 dark:text-gray-300 leading-relaxed",
                            {t("welcome.free_notice", locale)}
                        }
                        div {
                            class: "px-3 py-2.5 bg-amber-50 dark:bg-amber-900/20 rounded-lg border border-amber-200/60 dark:border-amber-700/30",
                            p {
                                class: "text-xs text-amber-700 dark:text-amber-300 leading-relaxed",
                                {t("welcome.scam_warning", locale)}
                            }
                        }
                    }
                    div {
                        class: "px-6 py-4 border-t border-gray-100 dark:border-gray-700 flex justify-between items-center",
                        p {
                            class: "text-xs text-gray-400 dark:text-gray-500",
                            {t("welcome.douyin", locale)}
                        }
                        button {
                            class: "px-5 py-2 text-white text-sm font-medium rounded-lg hover:opacity-90 transition-opacity",
                            style: "background: #D97757;",
                            onclick: move |_| {
                                show_welcome.set(false);
                                document::eval("localStorage.setItem('wm_welcome','1')");
                            },
                            {t("welcome.dismiss", locale)}
                        }
                    }
                }
            }
        }
    }
}
