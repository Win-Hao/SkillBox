#![allow(non_snake_case)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod components;
mod hooks;
mod models;
mod routes;
mod services;
pub mod theme;

fn load_window_size() -> (f64, f64) {
    let config_path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("SkillBox")
        .join("window.json");
    if let Ok(data) = std::fs::read_to_string(&config_path) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&data) {
            let w = val["width"].as_f64().unwrap_or(1200.0);
            let h = val["height"].as_f64().unwrap_or(780.0);
            if w >= 520.0 && h >= 400.0 {
                return (w, h);
            }
        }
    }
    (1200.0, 780.0)
}

fn main() {
    eprintln!("SkillBox v1.0 — Winhao学AI (抖音搜索同名)");
    eprintln!("本软件完全免费，不可商业化");

    let (w, h) = load_window_size();

    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            dioxus::desktop::Config::new()
                .with_background_color((249, 250, 251, 255))
                .with_custom_head(r#"<meta charset="utf-8"><style>
html,body{margin:0;background:#f9fafb;transition:background .3s ease}
html.dark,html.dark body{background:#111827}
#splash{position:fixed;inset:0;z-index:99999;display:flex;align-items:center;justify-content:center;background:#f9fafb;transition:opacity .3s ease,background .3s ease}
html.dark #splash{background:#111827}
#splash.fade-out{opacity:0;pointer-events:none}
#splash .dot-group{display:flex;gap:6px}
#splash .dot{width:8px;height:8px;border-radius:50%;background:#d97757;animation:bounce .6s infinite alternate}
#splash .dot:nth-child(2){animation-delay:.15s}
#splash .dot:nth-child(3){animation-delay:.3s}
@keyframes bounce{from{opacity:.3;transform:translateY(0)}to{opacity:1;transform:translateY(-8px)}}
#main{opacity:0;transition:opacity .2s ease .05s}
#main.ready{opacity:1}
</style>
<script>if(localStorage.getItem('theme')==='dark'){document.documentElement.classList.add('dark');document.body.classList.add('dark')}</script>
<div id="splash"><div class="dot-group"><div class="dot"></div><div class="dot"></div><div class="dot"></div></div></div>"#.into())
                .with_window(
                    dioxus::desktop::tao::window::WindowBuilder::new()
                        .with_title("SkillBox — Winhao学AI (免费软件，不可商业化)")
                        .with_inner_size(dioxus::desktop::tao::dpi::LogicalSize::new(w, h))
                        .with_min_inner_size(dioxus::desktop::tao::dpi::LogicalSize::new(520.0, 400.0))
                )
        )
        .launch(app::App);
}
