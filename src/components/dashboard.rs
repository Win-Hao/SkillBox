use dioxus::prelude::*;
use crate::hooks::SkillsState;
use crate::models::SkillSource;
use crate::components::icons::ClaudeMascot;
use crate::theme::{AppSettings, Locale, Theme, t};
use std::collections::BTreeMap;

#[component]
pub fn DashboardView() -> Element {
    let state = use_context::<Signal<SkillsState>>();
    let settings = use_context::<Signal<AppSettings>>();
    let locale = settings.read().locale;
    let skills = &state.read().skills;

    let total = skills.len();
    let enabled = skills.iter().filter(|s| s.enabled).count();
    let disabled = total - enabled;
    let custom = skills.iter().filter(|s| s.source == SkillSource::Custom).count();
    let downloaded = skills.iter().filter(|s| s.source == SkillSource::Downloaded).count();
    let linked = skills.iter().filter(|s| s.source == SkillSource::Linked).count();
    let loose = skills.iter().filter(|s| s.source == SkillSource::LooseFile).count();
    let total_size: u64 = skills.iter().map(|s| s.total_size).sum();
    let total_files: usize = skills.iter().map(|s| s.file_count).sum();
    let size_str = bytesize::ByteSize(total_size).to_string();

    let enabled_pct = if total > 0 { (enabled as f64 / total as f64 * 100.0) as u32 } else { 0 };
    let disabled_pct = if total > 0 { (disabled as f64 / total as f64 * 100.0) as u32 } else { 0 };

    let source_segments: Vec<(&str, usize, &str)> = vec![
        (t("dashboard.my_skills", locale), custom, "#3b82f6"),
        (t("dashboard.downloaded", locale), downloaded, "#f59e0b"),
        (t("dashboard.linked", locale), linked, "#a855f7"),
        (t("dashboard.file", locale), loose, "#6b7280"),
    ];

    let mut sized_skills: Vec<_> = skills.iter().collect();
    sized_skills.sort_by(|a, b| b.total_size.cmp(&a.total_size));
    let top_skills: Vec<_> = sized_skills.into_iter().take(8).collect();
    let max_size = top_skills.first().map(|s| s.total_size).unwrap_or(1).max(1);

    let timeline_data = build_timeline(skills.iter().filter_map(|s| s.installed_at).collect());

    let mut recent_skills: Vec<_> = skills.iter().filter(|s| s.installed_at.is_some()).collect();
    recent_skills.sort_by(|a, b| b.installed_at.cmp(&a.installed_at));
    let recent_skills: Vec<_> = recent_skills.into_iter().take(8).collect();

    let files_label = format!("{total_files} {}", t("dashboard.files", locale));

    rsx! {
        div {
            class: "px-4 py-5 sm:px-6 sm:py-6 max-w-5xl mx-auto",

            h1 {
                class: "text-lg font-semibold text-gray-800 dark:text-gray-100 mb-6",
                {t("dashboard.title", locale)}
            }

            // Row 1: Stat cards
            div {
                class: "grid grid-cols-2 lg:grid-cols-4 gap-3 sm:gap-4 mb-5 db-stat-grid",
                StatCard { value: "{total}", label: t("dashboard.total_skills", locale).to_string(), icon_bg: "bg-gray-100 dark:bg-gray-700", icon_color: "text-gray-500 dark:text-gray-400", icon: "M4 6h16M4 12h16M4 18h16" }
                StatCard { value: "{enabled}", label: t("dashboard.enabled", locale).to_string(), icon_bg: "bg-green-50 dark:bg-green-900/30", icon_color: "text-green-500", icon: "M5 13l4 4L19 7" }
                StatCard { value: "{disabled}", label: t("dashboard.disabled", locale).to_string(), icon_bg: "bg-red-50 dark:bg-red-900/30", icon_color: "text-red-400", icon: "M18.36 6.64A9 9 0 0 1 20.77 12 9 9 0 1 1 5.64 6.64M12 2v10" }
                StatCard { value: "{size_str}", label: files_label, icon_bg: "bg-blue-50 dark:bg-blue-900/30", icon_color: "text-blue-500", icon: "M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4" }
            }

            // Row 2: Growth chart (full width)
            MacPanel { title: t("dashboard.panel_growth", locale).to_string(),
                div {
                    class: "p-5",
                    if timeline_data.points.len() >= 2 {
                        AreaChart { data: timeline_data, locale: locale }
                    } else {
                        p {
                            class: "text-sm text-gray-400 dark:text-gray-500 py-8 text-center",
                            {t("dashboard.no_chart_data", locale)}
                        }
                    }
                    p {
                        class: "text-xs text-gray-400 dark:text-gray-500 text-center mt-2",
                        {t("dashboard.chart_note", locale)}
                    }
                }
            }

            // Row 3: Three columns
            div {
                class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3 sm:gap-4 mt-5 db-panel-grid",

                // Col 1: Source breakdown
                MacPanel { title: t("dashboard.panel_source", locale).to_string(),
                    div {
                        class: "p-5",
                        if total > 0 {
                            div {
                                class: "flex h-3 rounded-full overflow-hidden mb-4",
                                for (_label, count, color) in source_segments.iter() {
                                    if *count > 0 {
                                        {
                                            let pct = (*count as f64 / total as f64 * 100.0).max(2.0);
                                            rsx! {
                                                div {
                                                    style: "width: {pct}%; background-color: {color};",
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        div {
                            class: "space-y-2.5",
                            for (label, count, color) in source_segments.iter() {
                                div {
                                    class: "flex items-center justify-between",
                                    div {
                                        class: "flex items-center gap-2.5",
                                        div {
                                            class: "w-2.5 h-2.5 rounded-full flex-shrink-0",
                                            style: "background-color: {color};",
                                        }
                                        span { class: "text-sm text-gray-600 dark:text-gray-300", "{label}" }
                                    }
                                    div {
                                        class: "flex items-center gap-2",
                                        span { class: "text-sm font-medium text-gray-800 dark:text-gray-100 tabular-nums", "{count}" }
                                        if total > 0 {
                                            {
                                                let pct = (*count as f64 / total as f64 * 100.0) as u32;
                                                rsx! {
                                                    span {
                                                        class: "text-xs text-gray-400 dark:text-gray-500 tabular-nums",
                                                        "{pct}%"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        div {
                            class: "mt-4 pt-3 border-t border-gray-100 dark:border-gray-700 space-y-2.5",
                            div {
                                div {
                                    class: "flex items-center justify-between mb-1.5",
                                    span { class: "text-xs text-gray-400 dark:text-gray-500", {t("dashboard.enabled", locale)} }
                                    span { class: "text-xs font-medium text-gray-600 dark:text-gray-300", "{enabled_pct}%" }
                                }
                                div {
                                    class: "h-1.5 rounded-full bg-gray-100 dark:bg-gray-700 overflow-hidden",
                                    div {
                                        class: "h-full rounded-full",
                                        style: "width: {enabled_pct}%; background-color: #D97757;",
                                    }
                                }
                            }
                            div {
                                div {
                                    class: "flex items-center justify-between mb-1.5",
                                    span { class: "text-xs text-gray-400 dark:text-gray-500", {t("dashboard.disabled", locale)} }
                                    span { class: "text-xs font-medium text-gray-600 dark:text-gray-300", "{disabled_pct}%" }
                                }
                                div {
                                    class: "h-1.5 rounded-full bg-gray-100 dark:bg-gray-700 overflow-hidden",
                                    div {
                                        class: "h-full rounded-full",
                                        style: "width: {disabled_pct}%; background-color: #6b7280;",
                                    }
                                }
                            }
                        }
                    }
                }

                // Col 2: Size leaderboard
                MacPanel { title: t("dashboard.panel_largest", locale).to_string(),
                    div {
                        class: "p-5",
                        div {
                            class: "space-y-2",
                            for (i, skill) in top_skills.iter().enumerate() {
                                {
                                    let bar_pct = (skill.total_size as f64 / max_size as f64 * 100.0) as u32;
                                    let size_label = bytesize::ByteSize(skill.total_size).to_string();
                                    let rank = i + 1;
                                    let bar_color = if i == 0 { "#D97757" } else if i < 3 { "#F0B3A0" } else { "#e5e7eb" };
                                    let text_color = if i < 3 { "text-gray-800 dark:text-gray-100 font-medium" } else { "text-gray-600 dark:text-gray-300" };
                                    rsx! {
                                        div {
                                            class: "relative",
                                            div {
                                                class: "absolute inset-0 rounded-md opacity-20",
                                                style: "width: {bar_pct}%; background-color: {bar_color};",
                                            }
                                            div {
                                                class: "relative flex items-center gap-2 py-1.5 px-2",
                                                span {
                                                    class: "text-xs text-gray-400 dark:text-gray-500 w-4 text-right tabular-nums flex-shrink-0",
                                                    "{rank}"
                                                }
                                                span {
                                                    class: "text-sm {text_color} flex-1 truncate",
                                                    "{skill.frontmatter.name}"
                                                }
                                                span {
                                                    class: "text-xs text-gray-400 dark:text-gray-500 tabular-nums flex-shrink-0",
                                                    "{size_label}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Col 3: Recent activity
                MacPanel { title: t("dashboard.panel_recent", locale).to_string(),
                    div {
                        class: "p-5",
                        if recent_skills.is_empty() {
                            p {
                                class: "text-sm text-gray-400 dark:text-gray-500 py-4 text-center",
                                {t("dashboard.no_dates", locale)}
                            }
                        } else {
                            div {
                                class: "space-y-0.5",
                                for skill in recent_skills.iter() {
                                    {
                                        let date = skill.formatted_installed_date().unwrap_or_default();
                                        let src_color = match skill.source {
                                            SkillSource::Custom => "bg-blue-400",
                                            SkillSource::Downloaded => "bg-amber-400",
                                            SkillSource::Linked => "bg-purple-400",
                                            SkillSource::LooseFile => "bg-gray-400",
                                        };
                                        rsx! {
                                            div {
                                                class: "flex items-center gap-2.5 py-1.5 px-2 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors",
                                                div {
                                                    class: "w-1.5 h-1.5 rounded-full flex-shrink-0 {src_color}",
                                                }
                                                span {
                                                    class: "text-sm text-gray-700 dark:text-gray-200 flex-1 truncate",
                                                    "{skill.frontmatter.name}"
                                                }
                                                span {
                                                    class: "text-xs text-gray-400 dark:text-gray-500 tabular-nums flex-shrink-0",
                                                    "{date}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            div {
                class: "mt-5",
                div {
                    class: "bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 overflow-hidden",
                    div {
                        class: "px-5 py-3.5 flex items-center gap-4",
                        div {
                            style: "width:36px;height:36px;min-width:36px;min-height:36px;",
                            ClaudeMascot { class: "w-9 h-9" }
                        }
                        div {
                            class: "flex-1 min-w-0",
                            p {
                                class: "text-sm font-medium text-gray-700 dark:text-gray-200",
                                "Winhao学AI"
                            }
                            p {
                                class: "text-xs text-gray-400 dark:text-gray-500 mt-0.5",
                                {t("about.tagline", locale)}
                            }
                        }
                        span {
                            class: "text-[10px] text-gray-400/50 dark:text-gray-500/40 flex-shrink-0 tabular-nums",
                            "v1.0.0"
                        }
                    }
                }
            }
        }
    }
}

// --- Mac-style panel with traffic light title bar ---

#[component]
fn MacPanel(title: String, children: Element) -> Element {
    rsx! {
        div {
            class: "bg-white dark:bg-gray-800 rounded-xl border border-gray-300 dark:border-gray-600 overflow-hidden",
            div {
                class: "flex items-center gap-2 px-4 py-2.5 border-b border-gray-200 dark:border-gray-700 bg-gray-50/50 dark:bg-gray-800/50",
                div {
                    class: "flex items-center gap-1.5 flex-shrink-0",
                    div { class: "w-2.5 h-2.5 rounded-full bg-red-400" }
                    div { class: "w-2.5 h-2.5 rounded-full bg-amber-400" }
                    div { class: "w-2.5 h-2.5 rounded-full bg-green-400" }
                }
                span {
                    class: "text-xs font-medium text-gray-500 dark:text-gray-400 ml-1",
                    "{title}"
                }
            }
            {children}
        }
    }
}

// --- Stat card with mac border ---

#[component]
fn StatCard(
    value: String,
    label: String,
    icon_bg: &'static str,
    icon_color: &'static str,
    icon: &'static str,
) -> Element {
    rsx! {
        div {
            class: "bg-white dark:bg-gray-800 rounded-xl border border-gray-300 dark:border-gray-600 overflow-hidden",
            div {
                class: "flex items-center gap-1.5 px-3 py-1.5 border-b border-gray-200 dark:border-gray-700 bg-gray-50/50 dark:bg-gray-800/50",
                div { class: "w-2 h-2 rounded-full bg-red-400" }
                div { class: "w-2 h-2 rounded-full bg-amber-400" }
                div { class: "w-2 h-2 rounded-full bg-green-400" }
            }
            div {
                class: "p-4 flex items-start gap-3",
                div {
                    class: "rounded-lg p-2 {icon_bg}",
                    svg {
                        class: "w-4 h-4 {icon_color}",
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
                }
                div {
                    p {
                        class: "text-xl font-semibold text-gray-800 dark:text-gray-100 tabular-nums",
                        "{value}"
                    }
                    p {
                        class: "text-xs text-gray-400 dark:text-gray-500 mt-0.5",
                        "{label}"
                    }
                }
            }
        }
    }
}

// --- Timeline helpers ---

#[derive(Clone, PartialEq)]
struct TimelineData {
    points: Vec<(String, usize)>,
    max_val: usize,
}

fn build_timeline(mut timestamps: Vec<i64>) -> TimelineData {
    if timestamps.is_empty() {
        return TimelineData { points: vec![], max_val: 0 };
    }

    timestamps.sort();

    let mut weekly: BTreeMap<String, usize> = BTreeMap::new();
    for ts in &timestamps {
        if let Some(dt) = chrono::DateTime::from_timestamp(*ts, 0) {
            let week_start = dt.format("%m/%d").to_string();
            *weekly.entry(week_start).or_insert(0) += 1;
        }
    }

    let mut cumulative = 0usize;
    let mut points = Vec::new();
    for (label, count) in &weekly {
        cumulative += count;
        points.push((label.clone(), cumulative));
    }

    if points.len() > 15 {
        let step = points.len() as f64 / 14.0;
        let mut sampled = Vec::new();
        let mut i = 0.0;
        while (i as usize) < points.len() {
            sampled.push(points[i as usize].clone());
            i += step;
        }
        if sampled.last() != points.last() {
            sampled.push(points.last().unwrap().clone());
        }
        points = sampled;
    }

    let max_val = points.last().map(|(_, v)| *v).unwrap_or(1);
    TimelineData { points, max_val }
}

#[component]
fn AreaChart(data: TimelineData, locale: Locale) -> Element {
    let settings = use_context::<Signal<AppSettings>>();
    let is_dark = settings.read().theme == Theme::Dark;
    let w = 600.0_f64;
    let h = 180.0_f64;
    let pad_left = 40.0_f64;
    let pad_right = 16.0_f64;
    let pad_top = 12.0_f64;
    let pad_bottom = 28.0_f64;
    let chart_w = w - pad_left - pad_right;
    let chart_h = h - pad_top - pad_bottom;
    let n = data.points.len();
    let max_v = data.max_val as f64;

    let mut hover_idx: Signal<Option<usize>> = use_signal(|| None);
    let mut mouse_pos: Signal<(f64, f64)> = use_signal(|| (0.0, 0.0));

    let coords: Vec<(f64, f64)> = data.points.iter().enumerate().map(|(i, (_, v))| {
        let x = pad_left + (i as f64 / (n - 1).max(1) as f64) * chart_w;
        let y = pad_top + chart_h - (*v as f64 / max_v * chart_h);
        (x, y)
    }).collect();

    let mut line_path = String::new();
    for (i, (x, y)) in coords.iter().enumerate() {
        if i == 0 {
            line_path.push_str(&format!("M{x:.1},{y:.1}"));
        } else {
            let (px, py) = coords[i - 1];
            let cx = (px + x) / 2.0;
            line_path.push_str(&format!(" C{cx:.1},{py:.1} {cx:.1},{y:.1} {x:.1},{y:.1}"));
        }
    }

    let area_path = if let (Some(first), Some(last)) = (coords.first(), coords.last()) {
        let bottom = pad_top + chart_h;
        format!("{line_path} L{:.1},{bottom:.1} L{:.1},{bottom:.1} Z", last.0, first.0)
    } else {
        String::new()
    };

    let y_ticks: Vec<(f64, String)> = (0..=4).map(|i| {
        let v = (max_v * i as f64 / 4.0) as usize;
        let y = pad_top + chart_h - (i as f64 / 4.0 * chart_h);
        (y, format!("{v}"))
    }).collect();

    let label_step = if n <= 8 { 1 } else { (n / 6).max(1) };
    let x_labels: Vec<(f64, String)> = data.points.iter().enumerate()
        .filter(|(i, _)| *i % label_step == 0 || *i == n - 1)
        .map(|(i, (label, _))| {
            let x = pad_left + (i as f64 / (n - 1).max(1) as f64) * chart_w;
            (x, label.clone())
        }).collect();

    let chinese_labels: Vec<String> = data.points.iter().map(|(label, _)| {
        if let Some((m, d)) = label.split_once('/') {
            let m = m.trim_start_matches('0');
            let d = d.trim_start_matches('0');
            format!("{m}月{d}日")
        } else {
            label.clone()
        }
    }).collect();

    let current_hover = hover_idx.read().clone();
    let (mx, my) = *mouse_pos.read();

    let tooltip_opacity = if current_hover.is_some() { 1.0 } else { 0.0 };
    let skills_label = t("dashboard.skills_label", locale);
    let (tip_label, tip_count) = current_hover.map(|idx| {
        (
            chinese_labels.get(idx).cloned().unwrap_or_default(),
            data.points.get(idx).map(|(_, c)| *c).unwrap_or(0),
        )
    }).unwrap_or((String::new(), 0));

    rsx! {
        div {
            style: "position: relative;",
            onmousemove: move |evt| {
                let pos = evt.element_coordinates();
                mouse_pos.set((pos.x, pos.y));
            },
            onmouseleave: move |_| hover_idx.set(None),

            svg {
                view_box: "0 0 {w} {h}",
                class: "w-full",
                style: "max-height: 200px;",

                for (y, _label) in y_ticks.iter() {
                    line {
                        x1: "{pad_left}",
                        y1: "{y}",
                        x2: "{w - pad_right}",
                        y2: "{y}",
                        stroke: "#f3f4f6",
                        stroke_width: "1",
                    }
                }

                path {
                    d: "{area_path}",
                    fill: "url(#area-gradient)",
                }

                path {
                    d: "{line_path}",
                    fill: "none",
                    stroke: "#D97757",
                    stroke_width: "2",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                }

                if let Some(hi) = current_hover {
                    if let Some((hx, hy)) = coords.get(hi) {
                        line {
                            x1: "{hx}",
                            y1: "{hy}",
                            x2: "{hx}",
                            y2: "{pad_top + chart_h}",
                            stroke: "#D97757",
                            stroke_width: "1",
                            stroke_dasharray: "3,2",
                            opacity: "0.5",
                        }
                    }
                }

                for (idx, (x, y)) in coords.iter().enumerate() {
                    if current_hover == Some(idx) {
                        circle {
                            cx: "{x}",
                            cy: "{y}",
                            r: "6",
                            fill: "white",
                            stroke: "#D97757",
                            stroke_width: "2",
                        }
                    } else {
                        circle {
                            cx: "{x}",
                            cy: "{y}",
                            r: "3",
                            fill: "white",
                            stroke: "#D97757",
                            stroke_width: "1.5",
                        }
                    }
                }

                for (idx, (x, _y)) in coords.iter().enumerate() {
                    rect {
                        x: "{x - 12.0}",
                        y: "{pad_top}",
                        width: "24",
                        height: "{chart_h}",
                        fill: "transparent",
                        onmouseenter: move |_| hover_idx.set(Some(idx)),
                    }
                }

                for (y, label) in y_ticks.iter() {
                    text {
                        x: "{pad_left - 8.0}",
                        y: "{y}",
                        text_anchor: "end",
                        dominant_baseline: "middle",
                        fill: "#9ca3af",
                        font_size: "10",
                        "{label}"
                    }
                }

                for (x, label) in x_labels.iter() {
                    text {
                        x: "{x}",
                        y: "{h - 6.0}",
                        text_anchor: "middle",
                        fill: "#9ca3af",
                        font_size: "10",
                        "{label}"
                    }
                }

                defs {
                    linearGradient {
                        id: "area-gradient",
                        x1: "0",
                        y1: "0",
                        x2: "0",
                        y2: "1",
                        stop { offset: "0%", stop_color: "#D97757", stop_opacity: "0.25" }
                        stop { offset: "100%", stop_color: "#D97757", stop_opacity: "0.02" }
                    }
                }
            }

            div {
                style: "position: absolute; left: 0; top: 0; transform: translate3d({mx + 12.0}px, {my + 12.0}px, 0); opacity: {tooltip_opacity}; pointer-events: none; will-change: transform, opacity; transition: opacity 0.15s ease; z-index: 50;",
                div {
                    style: if is_dark {
                        "background: #1f2937; border: 1.5px solid #D97757; border-radius: 8px; padding: 6px 12px; box-shadow: 0 2px 8px rgba(0,0,0,0.3);"
                    } else {
                        "background: white; border: 1.5px solid #D97757; border-radius: 8px; padding: 6px 12px; box-shadow: 0 2px 8px rgba(0,0,0,0.08);"
                    },
                    p {
                        style: if is_dark {
                            "font-size: 12px; font-weight: 600; color: #e5e7eb; white-space: nowrap; margin: 0;"
                        } else {
                            "font-size: 12px; font-weight: 600; color: #374151; white-space: nowrap; margin: 0;"
                        },
                        "{tip_label}"
                    }
                    p {
                        style: "font-size: 12px; color: #D97757; white-space: nowrap; margin: 2px 0 0 0;",
                        "{skills_label} : {tip_count}"
                    }
                }
            }
        }
    }
}
