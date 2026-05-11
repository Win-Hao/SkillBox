use std::collections::BTreeMap;
use dioxus::prelude::*;

static ID_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);

fn next_id() -> usize {
    ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastType {
    Success,
    Warning,
    Error,
    Info,
}

#[derive(Debug, Clone)]
struct ToastItem {
    id: usize,
    message: String,
    toast_type: ToastType,
    expire_at: i64,
    dismissing: bool,
}

const VISIBLE_TOASTS: usize = 3;
const GAP: i32 = 14;
const TOAST_WIDTH: u32 = 356;

#[derive(Debug, Clone)]
pub struct ToastManager {
    list: BTreeMap<usize, ToastItem>,
}

impl ToastManager {
    pub fn new() -> Self {
        Self { list: BTreeMap::new() }
    }

    fn push(&mut self, message: impl Into<String>, toast_type: ToastType, duration_secs: u64) {
        let id = next_id();
        let expire_at = chrono::Local::now().timestamp() + duration_secs as i64;

        if self.list.len() >= VISIBLE_TOASTS {
            if let Some(&oldest_id) = self.list.keys().next() {
                self.list.remove(&oldest_id);
            }
        }

        self.list.insert(id, ToastItem {
            id,
            message: message.into(),
            toast_type,
            expire_at,
            dismissing: false,
        });
    }

    pub fn success(&mut self, message: impl Into<String>) {
        self.push(message, ToastType::Success, 3);
    }

    pub fn warning(&mut self, message: impl Into<String>) {
        self.push(message, ToastType::Warning, 3);
    }

    pub fn error(&mut self, message: impl Into<String>) {
        self.push(message, ToastType::Error, 4);
    }

    pub fn info(&mut self, message: impl Into<String>) {
        self.push(message, ToastType::Info, 3);
    }

    pub fn dismiss(&mut self, id: usize) {
        if let Some(item) = self.list.get_mut(&id) {
            item.dismissing = true;
            item.expire_at = chrono::Local::now().timestamp();
        }
    }
}

#[component]
pub fn ToastFrame() -> Element {
    let mut manager = use_context::<Signal<ToastManager>>();
    let mut hovered = use_signal(|| false);

    let _ = use_resource(move || async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            if manager.read().list.is_empty() {
                continue;
            }
            let now = chrono::Local::now().timestamp();
            let mut mgr = manager.write();
            let expired: Vec<usize> = mgr.list.iter()
                .filter(|(_, item)| !item.dismissing && now >= item.expire_at)
                .map(|(&id, _)| id)
                .collect();
            for id in expired {
                if let Some(item) = mgr.list.get_mut(&id) {
                    item.dismissing = true;
                }
            }
            mgr.list.retain(|_, item| {
                if item.dismissing { now < item.expire_at + 1 } else { true }
            });
        }
    });

    let items: Vec<ToastItem> = manager.read().list.values().rev().cloned().collect();
    let count = items.len();

    if count == 0 {
        return rsx! {};
    }

    let is_hovered = *hovered.read();

    rsx! {
        div {
            class: "sonner-container",
            style: "position:fixed;top:24px;left:50%;transform:translateX(-50%);z-index:999999;width:{TOAST_WIDTH}px;pointer-events:none;",
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),
            for (i, item) in items.iter().enumerate() {
                {render_sonner_item(item.clone(), i, count, is_hovered, manager)}
            }
        }
    }
}

fn render_sonner_item(
    item: ToastItem,
    index: usize,
    _count: usize,
    expanded: bool,
    mut manager: Signal<ToastManager>,
) -> Element {
    let id = item.id;
    let is_front = index == 0;

    let top_px = if expanded {
        index as i32 * (48 + GAP)
    } else if is_front {
        0
    } else {
        index.min(2) as i32 * 4
    };

    let scale_val = if expanded || is_front {
        1.0
    } else {
        1.0 - (index.min(2) as f64 * 0.05)
    };

    let anim_class = if item.dismissing {
        "sonner-toast sonner-exit"
    } else {
        "sonner-toast sonner-enter"
    };

    let (bg, border, text) = match item.toast_type {
        ToastType::Success => ("hsl(143,85%,96%)", "hsl(145,92%,87%)", "hsl(140,100%,27%)"),
        ToastType::Warning => ("hsl(49,100%,97%)", "hsl(49,91%,84%)", "hsl(31,92%,45%)"),
        ToastType::Error   => ("hsl(359,100%,97%)", "hsl(359,100%,94%)", "hsl(360,100%,45%)"),
        ToastType::Info    => ("hsl(208,100%,97%)", "hsl(221,91%,93%)", "hsl(210,92%,45%)"),
    };

    let icon_svg = match item.toast_type {
        ToastType::Success => rsx! {
            svg { class: "sonner-icon", width: "20", height: "20", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2.5", stroke_linecap: "round", stroke_linejoin: "round", overflow: "visible",
                path { d: "M20 6 9 17l-5-5" }
            }
        },
        ToastType::Error => rsx! {
            svg { class: "sonner-icon", width: "20", height: "20", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2.5", stroke_linecap: "round", stroke_linejoin: "round", overflow: "visible",
                circle { cx: "12", cy: "12", r: "10" }
                path { d: "m15 9-6 6M9 9l6 6" }
            }
        },
        ToastType::Warning => rsx! {
            svg { class: "sonner-icon", width: "20", height: "20", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2.5", stroke_linecap: "round", stroke_linejoin: "round", overflow: "visible",
                path { d: "M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" }
                path { d: "M12 9v4M12 17h.01" }
            }
        },
        ToastType::Info => rsx! {
            svg { class: "sonner-icon", width: "20", height: "20", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2.5", stroke_linecap: "round", stroke_linejoin: "round", overflow: "visible",
                circle { cx: "12", cy: "12", r: "10" }
                path { d: "M12 16v-4M12 8h.01" }
            }
        },
    };

    rsx! {
        div {
            class: "{anim_class}",
            style: "position:absolute;left:0;right:0;\
                    top:{top_px}px;transform:scale({scale_val});\
                    padding:14px 16px;border-radius:12px;display:flex;align-items:center;gap:8px;font-size:13px;pointer-events:auto;box-sizing:border-box;\
                    background:{bg};border:1px solid {border};color:{text};\
                    box-shadow:0px 4px 12px rgba(0,0,0,0.1);\
                    transition:top 300ms cubic-bezier(0.4,0,0.2,1),transform 300ms cubic-bezier(0.4,0,0.2,1);\
                    z-index:{1000 - index};",
            {icon_svg}
            span {
                style: "flex:1;min-width:0;overflow-wrap:anywhere;font-weight:500;",
                "{item.message}"
            }
            button {
                class: "sonner-close",
                style: "position:absolute;top:0;left:0;width:20px;height:20px;border-radius:50%;border:1px solid {border};background:{bg};color:{text};display:flex;align-items:center;justify-content:center;cursor:pointer;transform:translate(-35%,-35%);padding:0;opacity:0;transition:opacity 100ms;",
                onclick: move |_| manager.write().dismiss(id),
                svg { width: "12", height: "12", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2.5", stroke_linecap: "round", stroke_linejoin: "round", overflow: "visible",
                    path { d: "M18 6 6 18M6 6l12 12" }
                }
            }
        }
    }
}
