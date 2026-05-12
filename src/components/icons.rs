// Lucide Icons (https://lucide.dev) — MIT License
use dioxus::prelude::*;

#[component]
pub fn ClaudeMascot(#[props(default = "w-8 h-8")] class: &'static str) -> Element {
    let c = "#D97757";
    let e = "#000000";
    rsx! {
        svg {
            class: "{class}",
            width: "36",
            height: "36",
            view_box: "0 5 15 11",
            shape_rendering: "crispEdges",
            overflow: "visible",

            // Legs (static, not affected by body breathing)
            g { fill: "{c}",
                rect { x: "3",  y: "13", width: "1", height: "2" }
                rect { x: "5",  y: "13", width: "1", height: "2" }
                rect { x: "9",  y: "13", width: "1", height: "2" }
                rect { x: "11", y: "13", width: "1", height: "2" }
            }

            // Body group (sway + breathe)
            g { class: "clawd-action",
                g { class: "clawd-breathe",
                    // Torso
                    rect { x: "2", y: "6", width: "11", height: "7", fill: "{c}" }
                    // Left arm (waves)
                    g { class: "clawd-arm-l",
                        rect { x: "0", y: "9", width: "2", height: "2", fill: "{c}" }
                    }
                    // Right arm
                    g { class: "clawd-arm-r",
                        rect { x: "13", y: "9", width: "2", height: "2", fill: "{c}" }
                    }
                    // Eyes (track + blink)
                    g { class: "clawd-eye-track", fill: "{e}",
                        g { class: "clawd-eye-blink",
                            rect { x: "4",  y: "8", width: "1", height: "2" }
                            rect { x: "10", y: "8", width: "1", height: "2" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn IconFolder(#[props(default = "w-4 h-4")] class: &'static str) -> Element {
    rsx! {
        svg {
            class: "{class}",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            view_box: "0 0 24 24",
            overflow: "visible",
            path { d: "M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z" }
        }
    }
}

#[component]
pub fn IconFile(#[props(default = "w-4 h-4")] class: &'static str) -> Element {
    rsx! {
        svg {
            class: "{class}",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            view_box: "0 0 24 24",
            overflow: "visible",
            path { d: "M6 22a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h8a2.4 2.4 0 0 1 1.704.706l3.588 3.588A2.4 2.4 0 0 1 20 8v12a2 2 0 0 1-2 2z" }
            path { d: "M14 2v5a1 1 0 0 0 1 1h5" }
        }
    }
}

#[component]
pub fn IconFileText(#[props(default = "w-4 h-4")] class: &'static str) -> Element {
    rsx! {
        svg {
            class: "{class}",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            view_box: "0 0 24 24",
            overflow: "visible",
            path { d: "M6 22a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h8a2.4 2.4 0 0 1 1.704.706l3.588 3.588A2.4 2.4 0 0 1 20 8v12a2 2 0 0 1-2 2z" }
            path { d: "M14 2v5a1 1 0 0 0 1 1h5" }
            path { d: "M10 9H8" }
            path { d: "M16 13H8" }
            path { d: "M16 17H8" }
        }
    }
}

#[component]
pub fn IconPackage(#[props(default = "w-4 h-4")] class: &'static str) -> Element {
    rsx! {
        svg {
            class: "{class}",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            view_box: "0 0 24 24",
            overflow: "visible",
            path { d: "M11 21.73a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73z" }
            path { d: "M12 22V12" }
            polyline { points: "3.29 7 12 12 20.71 7" }
            path { d: "m7.5 4.27 9 5.15" }
        }
    }
}

#[component]
pub fn IconHardDrive(#[props(default = "w-4 h-4")] class: &'static str) -> Element {
    rsx! {
        svg {
            class: "{class}",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            view_box: "0 0 24 24",
            overflow: "visible",
            path { d: "M10 16h.01" }
            path { d: "M2.212 11.577a2 2 0 0 0-.212.896V18a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-5.527a2 2 0 0 0-.212-.896L18.55 5.11A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z" }
            path { d: "M21.946 12.013H2.054" }
            path { d: "M6 16h.01" }
        }
    }
}

#[component]
pub fn IconZap(#[props(default = "w-4 h-4")] class: &'static str) -> Element {
    rsx! {
        svg {
            class: "{class}",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            view_box: "0 0 24 24",
            overflow: "visible",
            path { d: "M4 14a1 1 0 0 1-.78-1.63l9.9-10.2a.5.5 0 0 1 .86.46l-1.92 6.02A1 1 0 0 0 13 10h7a1 1 0 0 1 .78 1.63l-9.9 10.2a.5.5 0 0 1-.86-.46l1.92-6.02A1 1 0 0 0 11 14z" }
        }
    }
}

#[component]
pub fn IconStar(#[props(default = "w-4 h-4")] class: &'static str) -> Element {
    rsx! {
        svg {
            class: "{class}",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            view_box: "0 0 24 24",
            overflow: "visible",
            path { d: "M11.525 2.295a.53.53 0 0 1 .95 0l2.31 4.679a2.123 2.123 0 0 0 1.595 1.16l5.166.756a.53.53 0 0 1 .294.904l-3.736 3.638a2.123 2.123 0 0 0-.611 1.878l.882 5.14a.53.53 0 0 1-.771.56l-4.618-2.428a2.122 2.122 0 0 0-1.973 0L6.396 21.01a.53.53 0 0 1-.77-.56l.881-5.139a2.122 2.122 0 0 0-.611-1.879L2.16 9.795a.53.53 0 0 1 .294-.906l5.165-.755a2.122 2.122 0 0 0 1.597-1.16z" }
        }
    }
}

#[component]
pub fn IconGitFork(#[props(default = "w-4 h-4")] class: &'static str) -> Element {
    rsx! {
        svg {
            class: "{class}",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            view_box: "0 0 24 24",
            overflow: "visible",
            circle { cx: "12", cy: "18", r: "3" }
            circle { cx: "6", cy: "6", r: "3" }
            circle { cx: "18", cy: "6", r: "3" }
            path { d: "M18 9v2c0 .6-.4 1-1 1H7c-.6 0-1-.4-1-1V9" }
            path { d: "M12 12v3" }
        }
    }
}

#[component]
pub fn IconCalendar(#[props(default = "w-4 h-4")] class: &'static str) -> Element {
    rsx! {
        svg {
            class: "{class}",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            view_box: "0 0 24 24",
            overflow: "visible",
            path { d: "M8 2v4" }
            path { d: "M16 2v4" }
            rect { width: "18", height: "18", x: "3", y: "4", rx: "2" }
            path { d: "M3 10h18" }
        }
    }
}

#[component]
pub fn IconPencil(#[props(default = "w-4 h-4")] class: &'static str) -> Element {
    rsx! {
        svg {
            class: "{class}",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            view_box: "0 0 24 24",
            overflow: "visible",
            path { d: "M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z" }
            path { d: "m15 5 4 4" }
        }
    }
}

#[component]
pub fn IconTerminal(#[props(default = "w-4 h-4")] class: &'static str) -> Element {
    rsx! {
        svg {
            class: "{class}",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            view_box: "0 0 24 24",
            overflow: "visible",
            path { d: "m4 17 6-6-6-6" }
            path { d: "M12 19h8" }
        }
    }
}

#[component]
pub fn IconArrowUp(#[props(default = "w-4 h-4")] class: &'static str) -> Element {
    rsx! {
        svg {
            class: "{class}",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            view_box: "0 0 24 24",
            overflow: "visible",
            path { d: "m5 12 7-7 7 7" }
            path { d: "M12 19V5" }
        }
    }
}
