use super::skill::SkillFrontmatter;

pub fn parse_skill_md(content: &str) -> Option<(SkillFrontmatter, String)> {
    let content = content.trim_start();
    if !content.starts_with("---") {
        return None;
    }

    let after_first = &content[3..];
    // Find the closing "---" on its own line. The frontmatter section cannot contain
    // fenced code blocks, so we only need to find the first "\n---" that is immediately
    // followed by a newline or EOF (i.e., "---" alone on a line).
    let close_idx = find_closing_fence(after_first)?;
    let yaml_str = &after_first[..close_idx].trim();
    let body_start = close_idx + 4; // skip "\n---"
    // Skip optional trailing characters on the closing fence line (e.g., "\n---\n")
    let body_start = if body_start < after_first.len() && after_first.as_bytes()[body_start] == b'\n' {
        body_start + 1
    } else {
        body_start
    };
    let body = if body_start < after_first.len() {
        after_first[body_start..].trim_start_matches('\n').to_string()
    } else {
        String::new()
    };

    let frontmatter: SkillFrontmatter = serde_yaml::from_str(yaml_str).ok()?;
    Some((frontmatter, body))
}

/// Find the closing `---` fence for YAML frontmatter.
/// The closing fence must be `---` alone on a line (preceded by `\n` and followed by `\n` or EOF).
fn find_closing_fence(text: &str) -> Option<usize> {
    let mut search_from = 0;
    loop {
        let idx = text[search_from..].find("\n---")?;
        let absolute_idx = search_from + idx;
        let after_dashes = absolute_idx + 4; // position after "\n---"
        // Check that the "---" is followed by newline, EOF, or only whitespace then newline
        if after_dashes >= text.len() {
            return Some(absolute_idx);
        }
        let rest = &text[after_dashes..];
        if rest.starts_with('\n') || rest.starts_with("\r\n") {
            return Some(absolute_idx);
        }
        // Not a valid closing fence (e.g., "---something" or "----"), keep searching
        search_from = absolute_idx + 1;
    }
}

pub fn serialize_skill_md(frontmatter: &SkillFrontmatter, body: &str) -> Result<String, String> {
    let yaml = serde_yaml::to_string(frontmatter)
        .map_err(|e| format!("Failed to serialize frontmatter: {e}"))?;
    Ok(format!("---\n{}---\n\n{}", yaml, body))
}
