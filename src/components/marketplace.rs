use dioxus::prelude::*;
use crate::hooks::{SkillsState, MarketplaceCache};
use crate::components::icons::*;
use crate::services::marketplace::{self, MarketplaceSkill};
use crate::theme::{AppSettings, t};

fn estimate_card_height(skill: &MarketplaceSkill) -> u32 {
    let base = 120u32;
    let category_height = 28;
    let desc_len = skill.description.as_deref().unwrap_or("").len() as u32;
    let desc_lines = (desc_len / 50).min(8);
    base + category_height + desc_lines * 18
}

fn distribute_mp_columns<'a>(skills: &'a [MarketplaceSkill], col_count: usize) -> Vec<Vec<&'a MarketplaceSkill>> {
    let mut columns: Vec<Vec<&MarketplaceSkill>> = (0..col_count).map(|_| Vec::new()).collect();
    let mut col_heights = vec![0u32; col_count];
    for skill in skills {
        let shortest = col_heights.iter().enumerate().min_by_key(|(_, h)| *h).map(|(i, _)| i).unwrap_or(0);
        col_heights[shortest] += estimate_card_height(skill);
        columns[shortest].push(skill);
    }
    columns
}

#[component]
pub fn MarketplaceView() -> Element {
    let mut state = use_context::<Signal<SkillsState>>();
    let mut mp_cache = use_context::<Signal<MarketplaceCache>>();
    let settings = use_context::<Signal<AppSettings>>();
    let locale = settings.read().locale;

    let (init_query, init_sort, init_page, init_skills, init_has_next, has_cache) = {
        let cache = mp_cache.read();
        let q = cache.last_query.clone();
        let s = cache.last_sort.clone();
        match cache.get(&q, &s) {
            Some(c) => (q, s, c.page, c.skills.clone(), c.has_next, true),
            None => (q, s, 1, Vec::new(), false, false),
        }
    };

    let mut query = use_signal(move || init_query.clone());
    let mut sort = use_signal(move || init_sort.clone());
    let mut page = use_signal(move || init_page);
    let mut all_skills = use_signal(move || init_skills.clone());
    let mut has_next = use_signal(move || init_has_next);
    let mut loading = use_signal(move || !has_cache);
    let mut loading_more = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut mp_col_count = use_signal(|| 3usize);

    use_future(move || async move {
        let mut eval = document::eval(r#"
            (function() {
                function calc() {
                    var el = document.getElementById('main-scroll');
                    var w = el ? el.clientWidth : window.innerWidth;
                    if (w < 580) return 1;
                    if (w < 900) return 2;
                    return 3;
                }
                dioxus.send(calc());
                var ro = new ResizeObserver(function() { dioxus.send(calc()); });
                var el = document.getElementById('main-scroll');
                if (el) ro.observe(el);
            })();
        "#);
        while let Ok(count) = eval.recv::<usize>().await {
            if count != *mp_col_count.peek() {
                mp_col_count.set(count);
            }
        }
    });
    let mut install_status = use_signal(|| None::<(bool, String)>);
    let mut initialized = use_signal(move || has_cache);

    use_effect(move || {
        document::eval(r#"document.getElementById('main-scroll')?.scrollTo({top:0})"#);
    });

    if !*initialized.read() {
        initialized.set(true);
        loading.set(true);
        let q = query.read().clone();
        let s = sort.read().clone();
        spawn(async move {
            match marketplace::search_skills(&q, &s, 1).await {
                Ok(data) => {
                    let hn = data.pagination.as_ref().is_some_and(|p| p.has_next);
                    has_next.set(hn);
                    all_skills.set(data.skills.clone());
                    loading.set(false);
                    mp_cache.write().set(&q, &s, data.skills, 1, hn);
                }
                Err(e) => { error.set(Some(e)); loading.set(false); }
            }
        });
    }

    use_future(move || async move {
        let mut eval = document::eval(r#"
            (function() {
                var el = document.getElementById('main-scroll');
                if (!el) return;
                el.addEventListener('scroll', function() {
                    var nearBottom = el.scrollTop + el.clientHeight >= el.scrollHeight - 200;
                    var topBtn = document.getElementById('mp-scroll-top');
                    if (topBtn) {
                        topBtn.style.display = (el.scrollTop > 300 && !nearBottom) ? '' : 'none';
                    }
                    if (nearBottom) {
                        dioxus.send("load");
                    }
                });
            })();
        "#);
        loop {
            match eval.recv::<String>().await {
                Ok(_) => {
                    if !*loading_more.read() && *has_next.read() {
                        let p = *page.read() + 1;
                        page.set(p);
                        loading_more.set(true);
                        let q = query.read().clone();
                        let s = sort.read().clone();
                        spawn(async move {
                            match marketplace::search_skills(&q, &s, p).await {
                                Ok(data) => {
                                    let hn = data.pagination.as_ref().is_some_and(|pg| pg.has_next);
                                    has_next.set(hn);
                                    all_skills.write().extend(data.skills);
                                    loading_more.set(false);
                                    let snapshot = all_skills.read().clone();
                                    mp_cache.write().set(&q, &s, snapshot, p, hn);
                                }
                                Err(_) => { loading_more.set(false); }
                            }
                        });
                    }
                }
                Err(_) => break,
            }
        }
    });

    rsx! {
        style { "@keyframes mp-spin {{ 0% {{ transform: rotate(0deg) }} 100% {{ transform: rotate(360deg) }} }} .mp-loader {{ width:24px; height:24px; border:3px solid #F0B3A0; border-bottom-color:transparent; border-radius:50%; display:inline-block; box-sizing:border-box; animation:mp-spin 1s linear infinite; }}" }
        div {
            class: "flex-1",

            // Search - terminal style
            div {
                class: "sticky top-0 z-10 bg-gray-50/80 dark:bg-gray-900/80 backdrop-blur-sm border-b border-gray-200 dark:border-gray-700",
                div {
                    class: "max-w-6xl mx-auto px-4 py-3 sm:px-6 sm:py-4",
                    div {
                        class: "bg-claude-50/50 dark:bg-gray-800/50 rounded-xl border-2 border-claude-light/50 dark:border-claude-dark/50 hover:border-claude-light/70 dark:hover:border-claude/70 focus-within:border-claude/70 transition-all duration-300 ease-in-out overflow-hidden",
                    // Title bar
                    div {
                        class: "flex items-center justify-between px-3 py-2 border-b border-claude-light/30 dark:border-claude-dark/30 bg-claude-50/50 dark:bg-gray-800/50",
                        div {
                            class: "flex items-center gap-1.5",
                            div { class: "w-2.5 h-2.5 rounded-full bg-red-400" }
                            div { class: "w-2.5 h-2.5 rounded-full bg-amber-400" }
                            div { class: "w-2.5 h-2.5 rounded-full bg-green-400" }
                            span { class: "text-xs font-medium text-gray-600 dark:text-gray-300 ml-2", {t("marketplace.search_title", locale)} }
                        }
                        div {
                            class: "flex items-center gap-2",
                            span { class: "text-[11px] text-gray-400 dark:text-gray-500", {t("marketplace.sort_by", locale)} }
                            div {
                                class: "relative flex items-center rounded-full border border-gray-200 dark:border-gray-600 bg-gray-100/80 dark:bg-gray-700/80 p-0.5",
                                // Sliding indicator
                                div {
                                    class: "absolute top-0.5 bottom-0.5 left-0.5 w-[calc(50%-2px)] rounded-full bg-claude-light/40 dark:bg-claude-dark/40 shadow-sm transition-transform duration-300 ease-in-out",
                                    style: if *sort.read() == "stars" { "transform: translateX(0)" } else { "transform: translateX(calc(100% + 2px))" },
                                }
                                button {
                                    class: if *sort.read() == "stars" {
                                        "relative z-10 flex items-center gap-1 px-2.5 py-1 rounded-full text-[11px] font-medium text-claude-dark dark:text-claude-light transition-colors duration-300"
                                    } else {
                                        "relative z-10 flex items-center gap-1 px-2.5 py-1 rounded-full text-[11px] font-medium text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 transition-colors duration-300"
                                    },
                                    onclick: move |_| {
                                        document::eval(r#"document.getElementById('main-scroll')?.scrollTo({top:0,behavior:'smooth'})"#);
                                        sort.set("stars".to_string());
                                        let q = query.read().clone();
                                        let hit = mp_cache.read().get(&q, "stars").cloned();
                                        if let Some(cached) = hit {
                                            page.set(cached.page);
                                            has_next.set(cached.has_next);
                                            all_skills.set(cached.skills);
                                            mp_cache.write().last_sort = "stars".to_string();
                                        } else {
                                            page.set(1);
                                            all_skills.set(vec![]);
                                            loading.set(true);
                                            spawn(async move {
                                                match marketplace::search_skills(&q, "stars", 1).await {
                                                    Ok(data) => {
                                                        let hn = data.pagination.as_ref().is_some_and(|p| p.has_next);
                                                        has_next.set(hn);
                                                        all_skills.set(data.skills.clone());
                                                        loading.set(false);
                                                        mp_cache.write().set(&q, "stars", data.skills, 1, hn);
                                                    }
                                                    Err(e) => { error.set(Some(e)); loading.set(false); }
                                                }
                                            });
                                        }
                                    },
                                    IconStar { class: "w-3 h-3 text-amber-500" }
                                    {t("marketplace.stars", locale)}
                                }
                                button {
                                    class: if *sort.read() == "recent" {
                                        "relative z-10 flex items-center gap-1 px-2.5 py-1 rounded-full text-[11px] font-medium text-claude-dark dark:text-claude-light transition-colors duration-300"
                                    } else {
                                        "relative z-10 flex items-center gap-1 px-2.5 py-1 rounded-full text-[11px] font-medium text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 transition-colors duration-300"
                                    },
                                    onclick: move |_| {
                                        document::eval(r#"document.getElementById('main-scroll')?.scrollTo({top:0,behavior:'smooth'})"#);
                                        sort.set("recent".to_string());
                                        let q = query.read().clone();
                                        let hit = mp_cache.read().get(&q, "recent").cloned();
                                        if let Some(cached) = hit {
                                            page.set(cached.page);
                                            has_next.set(cached.has_next);
                                            all_skills.set(cached.skills);
                                            mp_cache.write().last_sort = "recent".to_string();
                                        } else {
                                            page.set(1);
                                            all_skills.set(vec![]);
                                            loading.set(true);
                                            spawn(async move {
                                                match marketplace::search_skills(&q, "recent", 1).await {
                                                    Ok(data) => {
                                                        let hn = data.pagination.as_ref().is_some_and(|p| p.has_next);
                                                        has_next.set(hn);
                                                        all_skills.set(data.skills.clone());
                                                        loading.set(false);
                                                        mp_cache.write().set(&q, "recent", data.skills, 1, hn);
                                                    }
                                                    Err(e) => { error.set(Some(e)); loading.set(false); }
                                                }
                                            });
                                        }
                                    },
                                    IconCalendar { class: "w-3 h-3" }
                                    {t("marketplace.recent", locale)}
                                }
                            }
                        }
                    }
                    // Input row
                    div {
                        class: "flex items-center gap-2 px-3 py-2.5 bg-white dark:bg-gray-800",
                        span { class: "text-sm text-green-600 dark:text-green-400 font-mono flex-shrink-0", "$ find" }
                        svg {
                            class: "w-3 h-3 text-gray-300 dark:text-gray-500 flex-shrink-0",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            overflow: "visible",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607z",
                            }
                        }
                        input {
                            class: "flex-1 py-0.5 bg-transparent text-sm font-mono focus:outline-none placeholder-gray-400 dark:placeholder-gray-500 text-gray-800 dark:text-gray-200",
                            r#type: "text",
                            placeholder: t("marketplace.placeholder", locale),
                            value: "{query}",
                            oninput: move |evt| query.set(evt.value().to_string()),
                            onkeypress: move |evt: Event<KeyboardData>| {
                                if evt.data().key() == Key::Enter {
                                    document::eval(r#"document.getElementById('main-scroll')?.scrollTo({top:0,behavior:'smooth'})"#);
                                    let q = query.read().clone();
                                    let s = sort.read().clone();
                                    let hit = mp_cache.read().get(&q, &s).cloned();
                                    if let Some(cached) = hit {
                                        page.set(cached.page);
                                        has_next.set(cached.has_next);
                                        all_skills.set(cached.skills);
                                        let mut w = mp_cache.write();
                                        w.last_query = q;
                                        w.last_sort = s;
                                    } else {
                                        page.set(1);
                                        all_skills.set(vec![]);
                                        loading.set(true);
                                        error.set(None);
                                        spawn(async move {
                                            match marketplace::search_skills(&q, &s, 1).await {
                                                Ok(data) => {
                                                    let hn = data.pagination.as_ref().is_some_and(|p| p.has_next);
                                                    has_next.set(hn);
                                                    all_skills.set(data.skills.clone());
                                                    loading.set(false);
                                                    mp_cache.write().set(&q, &s, data.skills, 1, hn);
                                                }
                                                Err(e) => { error.set(Some(e)); loading.set(false); }
                                            }
                                        });
                                    }
                                }
                            },
                        }
                    }
                }
                }
            }

            // Results
            div {
                class: "px-4 py-4 sm:px-6 sm:py-6 max-w-6xl mx-auto",

                if let Some(ref err) = *error.read() {
                    div {
                        class: "bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-800 rounded-lg p-4 mb-4 text-sm text-red-700 dark:text-red-300",
                        "{err}"
                    }
                }

                if let Some((success, ref msg)) = *install_status.read() {
                    div {
                        class: if success {
                            "bg-green-50 dark:bg-green-900/30 border border-green-200 dark:border-green-800 rounded-lg p-3 mb-4 text-sm text-green-700 dark:text-green-300"
                        } else {
                            "bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-800 rounded-lg p-3 mb-4 text-sm text-red-700 dark:text-red-300"
                        },
                        "{msg}"
                    }
                }

                if *loading.read() {
                    div {
                        class: "flex gap-4",
                        for _ in 0..*mp_col_count.read() {
                          div {
                            class: "flex-1 flex flex-col gap-4 min-w-0",
                            for _ in 0..2 {
                            div {
                                class: "bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 overflow-hidden",
                                div {
                                    class: "flex items-center justify-between px-4 py-2.5 border-b border-gray-200 dark:border-gray-700 bg-gray-50/50 dark:bg-gray-800/50",
                                    div {
                                        class: "flex items-center gap-2",
                                        div { class: "flex items-center gap-1.5",
                                            div { class: "w-2.5 h-2.5 rounded-full bg-gray-200 dark:bg-gray-600" }
                                            div { class: "w-2.5 h-2.5 rounded-full bg-gray-200 dark:bg-gray-600" }
                                            div { class: "w-2.5 h-2.5 rounded-full bg-gray-200 dark:bg-gray-600" }
                                        }
                                        div { class: "skeleton h-3 w-20" }
                                    }
                                    div { class: "skeleton h-3 w-10" }
                                }
                                div {
                                    class: "px-4 py-3.5",
                                    div { class: "skeleton h-3 w-24 mb-2.5" }
                                    div { class: "skeleton h-3 w-full mb-2" }
                                    div { class: "skeleton h-3 w-4/5 mb-2" }
                                    div { class: "skeleton h-3 w-3/5 mb-3.5" }
                                    div {
                                        class: "flex items-center justify-between pt-2.5 border-t border-gray-200 dark:border-gray-700",
                                        div { class: "skeleton h-3 w-16" }
                                        div { class: "skeleton h-6 w-14 rounded-lg" }
                                    }
                                }
                            }
                          }
                         }
                        }
                    }
                } else if all_skills.read().is_empty() {
                    div {
                        class: "flex items-center justify-center py-20 text-gray-400 dark:text-gray-500",
                        {t("marketplace.no_results", locale)}
                    }
                } else {
                    {
                        let skills = all_skills.read();
                        let columns = distribute_mp_columns(&skills, *mp_col_count.read());
                        rsx! {
                            div {
                                class: "flex gap-4 mb-6",
                                for col in columns.iter() {
                                    div {
                                        class: "flex-1 flex flex-col gap-4 min-w-0",
                                        for skill in col.iter() {
                                    MarketplaceCard {
                                        key: "{skill.id}",
                                        skill: (*skill).clone(),
                                        on_click: move |s: MarketplaceSkill| {
                                            state.write().marketplace_preview = Some(s);
                                        },
                                        on_install: move |s: MarketplaceSkill| {
                                            install_status.set(None);
                                            let skill_id = s.id.clone();
                                            spawn(async move {
                                                let content = match marketplace::fetch_skill_content(&s).await {
                                                    Ok(c) => c,
                                                    Err(e) => {
                                                        install_status.set(Some((false, format!("{}: {e}", t("toast.fetch_failed", locale)))));
                                                        return;
                                                    }
                                                };
                                                mp_cache.write().set_drawer_content(&skill_id, content.clone());
                                                match marketplace::install_skill(&s).await {
                                                    Ok(name) => {
                                                        let compat = marketplace::check_compatibility(&[], &content);
                                                        let installed_msg = t("toast.installed", locale).replace("{}", &name);
                                                        let msg = if let Some(warning) = compat {
                                                            format!("{installed_msg} ⚠ {warning}")
                                                        } else {
                                                            installed_msg
                                                        };
                                                        install_status.set(Some((true, msg)));
                                                        state.write().reload();
                                                    }
                                                    Err(e) => {
                                                        install_status.set(Some((false, e)));
                                                    }
                                                }
                                            });
                                        },
                                    }
                                }
                            }
                          }
                         }

                            // Infinite scroll spinner
                            if *loading_more.read() {
                                div {
                                    class: "flex items-center justify-center py-6",
                                    div { class: "mp-loader" }
                                }
                            } else if !*has_next.read() && !all_skills.read().is_empty() {
                                div {
                                    class: "flex items-center justify-center py-6 text-xs text-gray-400 dark:text-gray-500 font-mono",
                                    {t("marketplace.end", locale)}
                                }
                            }
                        }
                    }
                }
            }
        }

        // Floating back-to-top button (visibility controlled by JS scroll handler)
        button {
            id: "mp-scroll-top",
            class: "scroll-top-btn flex items-center gap-2 px-5 py-2.5 text-sm font-mono text-gray-800 dark:text-gray-200 border border-claude-light/40 rounded-full shadow-lg",
            style: "display: none; position: fixed; bottom: 1.5rem; left: 50%; transform: translateX(-50%); z-index: 30; background: rgba(255,255,255,0.72); backdrop-filter: blur(16px); -webkit-backdrop-filter: blur(16px);",
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

#[component]
fn MarketplaceCard(
    skill: MarketplaceSkill,
    on_click: EventHandler<MarketplaceSkill>,
    on_install: EventHandler<MarketplaceSkill>,
) -> Element {
    let settings = use_context::<Signal<AppSettings>>();
    let locale = settings.read().locale;
    let name = skill.name.clone();
    let desc = skill.description.clone().unwrap_or_default();
    let author = skill.author.clone().unwrap_or_else(|| t("marketplace.unknown_author", locale).to_string());
    let stars = skill.stars.unwrap_or(0);
    let stars_display = if stars >= 1000 {
        format!("{:.1}k", stars as f64 / 1000.0)
    } else {
        format!("{}", stars)
    };
    let desc_display = if desc.len() > 400 {
        format!("{}…", desc.chars().take(400).collect::<String>().trim_end())
    } else {
        desc
    };
    let click_skill = skill.clone();
    let install_skill = skill.clone();

    rsx! {
        div {
            class: "bg-white dark:bg-gray-800 rounded-xl border border-gray-300 dark:border-gray-600 flex flex-col cursor-pointer overflow-hidden group card-interactive",
            onclick: move |_| on_click.call(click_skill.clone()),

            // Title bar with dots
            div {
                class: "flex items-center justify-between px-4 py-2.5 border-b border-gray-200 dark:border-gray-700 bg-gray-50/50 dark:bg-gray-800/50",
                div {
                    class: "flex items-center gap-2 min-w-0",
                    // macOS dots
                    div {
                        class: "flex items-center gap-1.5 flex-shrink-0 opacity-50 transition-opacity duration-200 group-hover:opacity-100",
                        div { class: "w-2.5 h-2.5 rounded-full bg-red-400" }
                        div { class: "w-2.5 h-2.5 rounded-full bg-amber-400" }
                        div { class: "w-2.5 h-2.5 rounded-full bg-green-400" }
                    }
                    span {
                        class: "text-sm font-semibold text-gray-800 dark:text-gray-100 truncate ml-1",
                        "{name}"
                    }
                }
                if stars > 0 {
                    span {
                        class: "flex items-center gap-1 text-xs text-gray-500 dark:text-gray-400 flex-shrink-0 ml-2",
                        IconStar { class: "w-3 h-3 text-amber-500 icon-animated" }
                        span { class: "font-medium", "{stars_display}" }
                    }
                }
            }

            // Body
            div {
                class: "px-4 py-3.5 flex-1 flex flex-col",
                // Author
                p {
                    class: "text-xs mb-2.5",
                    span { class: "text-gray-400 dark:text-gray-500", "{t(\"card.from\", locale)} " }
                    span { class: "text-blue-600 dark:text-blue-400 font-medium", "\"{author}\"" }
                }
                // Description
                p {
                    class: "text-[13px] text-gray-600 dark:text-gray-300 flex-1 mb-3.5 leading-relaxed",
                    "{desc_display}"
                }
                // Footer
                div {
                    class: "flex items-center justify-between pt-2.5 border-t border-gray-200 dark:border-gray-700",
                    if let Some(date) = skill.formatted_date() {
                        span {
                            class: "text-xs text-gray-400 dark:text-gray-500",
                            "{date}"
                        }
                    }
                    button {
                        class: "px-3 py-1 text-xs font-medium rounded-lg bg-claude text-white hover:bg-claude-dark active:scale-[0.95] transition-all duration-150",
                        onclick: move |evt| {
                            evt.stop_propagation();
                            on_install.call(install_skill.clone());
                        },
                        {t("marketplace.install", locale)}
                    }
                }
            }
        }
    }
}
