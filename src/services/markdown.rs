use pulldown_cmark::{Parser, Options, html};

pub fn md_to_html(md: &str) -> String {
    let opts = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS;
    let parser = Parser::new_ext(md, opts);
    let mut raw = String::new();
    html::push_html(&mut raw, parser);
    ammonia::clean(&raw)
}
