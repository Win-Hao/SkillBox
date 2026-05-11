use dioxus::prelude::*;
use crate::theme::{AppSettings, t};

#[component]
pub fn ConfirmDialog(
    title: String,
    message: String,
    confirm_label: String,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
    #[props(default)] secondary_label: Option<String>,
    #[props(default)] on_secondary: Option<EventHandler<()>>,
) -> Element {
    let settings = use_context::<Signal<AppSettings>>();
    let locale = settings.read().locale;

    let secondary_btn = match (secondary_label.as_ref(), on_secondary.as_ref()) {
        (Some(label), Some(handler)) => {
            let label = label.clone();
            let handler = handler.clone();
            rsx! {
                button {
                    class: "px-4 py-1.5 text-xs font-medium text-white bg-red-800 rounded-lg hover:bg-red-900 transition-colors",
                    onclick: move |_| handler.call(()),
                    "{label}"
                }
            }
        }
        _ => rsx! {},
    };

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-enter",
            onclick: move |_| on_cancel.call(()),
            div {
                class: "bg-white dark:bg-gray-800 rounded-2xl shadow-xl max-w-sm w-full mx-4 overflow-hidden dialog-enter",
                onclick: move |evt| evt.stop_propagation(),
                div {
                    class: "flex items-center justify-between px-5 py-3 border-b border-gray-200 dark:border-gray-700 bg-gray-50/50 dark:bg-gray-800/50",
                    span { class: "text-base font-semibold text-gray-800 dark:text-gray-100", "{title}" }
                    button {
                        class: "p-1 rounded-md text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors",
                        onclick: move |_| on_cancel.call(()),
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
                    class: "p-5",
                    p {
                        class: "text-sm text-gray-500 dark:text-gray-400 mb-5",
                        "{message}"
                    }
                    div {
                        class: "flex justify-end gap-2",
                        button {
                            class: "px-4 py-1.5 text-xs font-medium text-gray-700 dark:text-gray-200 bg-gray-100 dark:bg-gray-700 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors",
                            onclick: move |_| on_cancel.call(()),
                            {t("dialog.cancel", locale)}
                        }
                        button {
                            class: "px-4 py-1.5 text-xs font-medium text-white bg-red-600 rounded-lg hover:bg-red-700 transition-colors",
                            onclick: move |_| on_confirm.call(()),
                            "{confirm_label}"
                        }
                        {secondary_btn}
                    }
                }
            }
        }
    }
}
