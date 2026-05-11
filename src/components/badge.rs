use dioxus::prelude::*;
use crate::models::SkillSource;
use crate::theme::{AppSettings, t};

#[component]
pub fn SourceBadge(source: SkillSource, #[props(default)] linked_from: Option<String>) -> Element {
    let settings = use_context::<Signal<AppSettings>>();
    let locale = settings.read().locale;

    let (label, bg, text) = match source {
        SkillSource::Custom => (t("badge.my_skill", locale).to_string(), "bg-blue-100", "text-blue-700"),
        SkillSource::Downloaded => (t("badge.downloaded", locale).to_string(), "bg-amber-100", "text-amber-700"),
        SkillSource::Linked => {
            let base = t("badge.linked", locale);
            let label = match &linked_from {
                Some(from) => format!("{base} · {from}"),
                None => base.to_string(),
            };
            (label, "bg-purple-100", "text-purple-700")
        }
        SkillSource::LooseFile => (t("badge.file", locale).to_string(), "bg-gray-100", "text-gray-600"),
    };

    rsx! {
        span {
            class: "inline-flex items-center px-2 py-0.5 rounded text-xs font-medium {bg} {text}",
            "{label}"
        }
    }
}

#[component]
pub fn StatusBadge(enabled: bool) -> Element {
    let settings = use_context::<Signal<AppSettings>>();
    let locale = settings.read().locale;

    let (label, bg, text) = if enabled {
        (t("badge.enabled", locale), "bg-green-100", "text-green-700")
    } else {
        (t("badge.disabled", locale), "bg-red-100", "text-red-600")
    };

    rsx! {
        span {
            class: "inline-flex items-center px-2 py-0.5 rounded text-xs font-medium {bg} {text}",
            "{label}"
        }
    }
}
