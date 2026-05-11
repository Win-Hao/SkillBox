fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("icons/SkillBox.ico");
        res.set("ProductName", "SkillBox");
        res.set("FileDescription", "SkillBox");
        res.set("ProductVersion", "0.1.0");
        res.compile().expect("failed to compile windows resources");
    }
}
