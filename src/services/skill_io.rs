use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use crate::models::{Skill, parse_skill_md};
use crate::services::scanner::skills_base_dir;

pub fn disabled_dir() -> PathBuf {
    skills_base_dir().join(".disabled")
}

fn custom_marks_path() -> PathBuf {
    skills_base_dir().join(".custom-skills.json")
}

fn skill_sources_path() -> PathBuf {
    skills_base_dir().join(".skill-sources.json")
}

fn tracking_init_path() -> PathBuf {
    skills_base_dir().join(".skill-sources-init")
}

pub fn is_tracking_initialized() -> bool {
    tracking_init_path().exists()
}

pub fn mark_tracking_initialized() {
    let _ = fs::write(tracking_init_path(), "");
}

pub fn load_skill_sources() -> HashMap<String, String> {
    let path = skill_sources_path();
    let mut sources: HashMap<String, String> = if let Ok(content) = fs::read_to_string(&path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        HashMap::new()
    };

    let legacy_path = custom_marks_path();
    if legacy_path.exists() {
        if let Ok(content) = fs::read_to_string(&legacy_path) {
            if let Ok(marks) = serde_json::from_str::<HashSet<String>>(&content) {
                for name in marks {
                    if !sources.contains_key(&name) {
                        sources.insert(name, "custom".to_string());
                    }
                }
                let _ = save_skill_sources(&sources);
            }
        }
        let _ = fs::remove_file(&legacy_path);
    }

    sources
}

pub fn save_skill_sources(sources: &HashMap<String, String>) -> Result<(), String> {
    let json = serde_json::to_string_pretty(sources)
        .map_err(|e| format!("Failed to serialize: {e}"))?;
    fs::write(skill_sources_path(), json)
        .map_err(|e| format!("Failed to write skill sources: {e}"))
}

pub fn mark_skill_source(name: &str, source: &str) -> Result<(), String> {
    let mut sources = load_skill_sources();
    sources.insert(name.to_string(), source.to_string());
    save_skill_sources(&sources)
}

fn remove_skill_source(name: &str) -> Result<(), String> {
    let mut sources = load_skill_sources();
    sources.remove(name);
    save_skill_sources(&sources)
}

pub fn toggle_custom_mark(name: &str) -> Result<bool, String> {
    let mut sources = load_skill_sources();
    if let Some(s) = sources.get(name) {
        if s == "marketplace" || s == "git-clone" {
            return Err("Source is locked and cannot be changed".to_string());
        }
    }
    let is_now_custom = match sources.get(name).map(|s| s.as_str()) {
        Some("custom") => {
            sources.insert(name.to_string(), "downloaded".to_string());
            false
        }
        _ => {
            sources.insert(name.to_string(), "custom".to_string());
            true
        }
    };
    save_skill_sources(&sources)?;
    Ok(is_now_custom)
}

pub fn disable_skill(skill: &Skill) -> Result<(), String> {
    let target = disabled_dir();
    fs::create_dir_all(&target).map_err(|e| format!("Failed to create .disabled dir: {e}"))?;

    let file_name = skill.link_path.file_name()
        .ok_or("Invalid skill path")?;
    let dest = target.join(file_name);

    let meta = fs::symlink_metadata(&skill.link_path)
        .map_err(|e| format!("Failed to read metadata: {e}"))?;
    if meta.is_symlink() {
        let real_target = dunce::canonicalize(&skill.link_path)
            .map_err(|e| format!("Failed to resolve symlink: {e}"))?;
        fs::remove_file(&skill.link_path)
            .map_err(|e| format!("Failed to remove symlink: {e}"))?;
        #[cfg(unix)]
        std::os::unix::fs::symlink(&real_target, &dest)
            .map_err(|e| format!("Failed to create symlink in .disabled: {e}"))?;
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&real_target, &dest)
            .map_err(|e| format!("Failed to create symlink in .disabled: {e}"))?;
        Ok(())
    } else {
        fs::rename(&skill.link_path, &dest)
            .map_err(|e| format!("Failed to disable skill '{}': {e}", skill.dir_name))
    }
}

pub fn enable_skill(skill: &Skill) -> Result<(), String> {
    let file_name = skill.link_path.file_name()
        .ok_or("Invalid skill path")?;
    let source = disabled_dir().join(file_name);
    let dest = skills_base_dir().join(file_name);

    let meta = fs::symlink_metadata(&source)
        .map_err(|e| format!("Failed to read metadata: {e}"))?;
    if meta.is_symlink() {
        let real_target = dunce::canonicalize(&source)
            .map_err(|e| format!("Failed to resolve symlink: {e}"))?;
        fs::remove_file(&source)
            .map_err(|e| format!("Failed to remove symlink: {e}"))?;
        #[cfg(unix)]
        std::os::unix::fs::symlink(&real_target, &dest)
            .map_err(|e| format!("Failed to create symlink: {e}"))?;
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&real_target, &dest)
            .map_err(|e| format!("Failed to create symlink: {e}"))?;
        Ok(())
    } else {
        fs::rename(&source, &dest)
            .map_err(|e| format!("Failed to enable skill '{}': {e}", skill.dir_name))
    }
}

fn validate_path_under_skills_dir(path: &Path) -> Result<PathBuf, String> {
    let base = dunce::canonicalize(skills_base_dir())
        .map_err(|e| format!("Failed to resolve skills dir: {e}"))?;
    let resolved = dunce::canonicalize(path)
        .map_err(|e| format!("Failed to resolve path: {e}"))?;
    if !resolved.starts_with(&base) {
        return Err(format!("Path '{}' is outside the skills directory", resolved.display()));
    }
    Ok(resolved)
}

pub fn delete_skill(skill: &Skill) -> Result<(), String> {
    validate_path_under_skills_dir(&skill.link_path)?;
    trash::delete(&skill.link_path)
        .map_err(|e| format!("Failed to move to trash: {e}"))?;
    let _ = remove_skill_source(&skill.dir_name);
    Ok(())
}

pub fn delete_skill_completely(skill: &Skill) -> Result<Vec<String>, String> {
    let base = skills_base_dir();
    let real_path = validate_path_under_skills_dir(&skill.path)?;

    let mut removed = Vec::new();

    // Find and remove all symlinks pointing to this real path
    if let Ok(entries) = fs::read_dir(&base) {
        for entry in entries.flatten() {
            let meta = match fs::symlink_metadata(entry.path()) {
                Ok(m) => m,
                Err(_) => continue,
            };
            if !meta.is_symlink() {
                continue;
            }
            if let Ok(resolved) = dunce::canonicalize(entry.path()) {
                if resolved == real_path || resolved.starts_with(&real_path) {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let _ = fs::remove_file(entry.path());
                    let _ = remove_skill_source(&name);
                    removed.push(name);
                }
            }
        }
    }

    // Delete the original directory
    trash::delete(&real_path)
        .map_err(|e| format!("Failed to move to trash: {e}"))?;

    Ok(removed)
}

pub fn preview_skill(file_path: &Path) -> Result<(crate::models::SkillFrontmatter, String), String> {
    let ext = file_path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let content = match ext.as_str() {
        "md" => fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file: {e}"))?,
        "zip" => {
            let file = fs::File::open(file_path)
                .map_err(|e| format!("Failed to open zip: {e}"))?;
            let mut archive = zip::ZipArchive::new(file)
                .map_err(|e| format!("Invalid zip file: {e}"))?;

            let skill_md_path = (0..archive.len()).find_map(|i| {
                let entry = archive.by_index(i).ok()?;
                let name = entry.name().to_string();
                if name.ends_with("SKILL.md") && name.matches('/').count() <= 1 {
                    Some(name)
                } else {
                    None
                }
            }).ok_or("Zip must contain a SKILL.md file")?;

            let mut entry = archive.by_name(&skill_md_path)
                .map_err(|e| format!("Failed to read SKILL.md from zip: {e}"))?;
            let mut buf = String::new();
            io::Read::read_to_string(&mut entry, &mut buf)
                .map_err(|e| format!("Failed to read SKILL.md content: {e}"))?;
            buf
        }
        _ => return Err("Unsupported file type. Use .md or .zip".to_string()),
    };

    let (fm, body) = parse_skill_md(&content)
        .ok_or("Invalid skill file: must have YAML frontmatter with name and description")?;

    if fm.name.is_empty() || fm.description.is_empty() {
        return Err("Skill must have both name and description in frontmatter".to_string());
    }

    Ok((fm, body))
}

pub fn upload_skill(file_path: &Path) -> Result<String, String> {
    let ext = file_path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "md" => upload_md_file(file_path),
        "zip" => upload_zip_file(file_path),
        _ => Err("Unsupported file type. Use .md or .zip".to_string()),
    }
}

fn sanitize_upload_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn upload_md_file(file_path: &Path) -> Result<String, String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file: {e}"))?;

    let (fm, _) = parse_skill_md(&content)
        .ok_or("Invalid skill file: must have YAML frontmatter with name and description")?;

    if fm.name.is_empty() || fm.description.is_empty() {
        return Err("Skill must have both name and description in frontmatter".to_string());
    }

    let skill_name = sanitize_upload_name(&fm.name);
    if skill_name.is_empty() {
        return Err("Skill name contains only invalid characters".to_string());
    }
    let dest_dir = skills_base_dir().join(&skill_name);

    if dest_dir.exists() {
        return Err(format!("Skill '{}' already exists", skill_name));
    }

    fs::create_dir_all(&dest_dir)
        .map_err(|e| format!("Failed to create directory: {e}"))?;
    fs::write(dest_dir.join("SKILL.md"), &content)
        .map_err(|e| format!("Failed to write SKILL.md: {e}"))?;

    let _ = mark_skill_source(&skill_name, "uploaded");
    Ok(skill_name)
}

fn upload_zip_file(file_path: &Path) -> Result<String, String> {
    let file = fs::File::open(file_path)
        .map_err(|e| format!("Failed to open zip: {e}"))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("Invalid zip file: {e}"))?;

    let skill_md_entry = (0..archive.len()).find_map(|i| {
        let entry = archive.by_index(i).ok()?;
        let name = entry.name().to_string();
        if name.ends_with("SKILL.md") {
            let depth = name.matches('/').count();
            if depth <= 1 {
                Some(name)
            } else {
                None
            }
        } else {
            None
        }
    });

    let skill_md_path = skill_md_entry
        .ok_or("Zip must contain a SKILL.md file")?;

    let prefix = if skill_md_path.contains('/') {
        skill_md_path.rsplit_once('/').unwrap().0.to_string() + "/"
    } else {
        String::new()
    };

    let content = {
        let mut entry = archive.by_name(&skill_md_path)
            .map_err(|e| format!("Failed to read SKILL.md from zip: {e}"))?;
        let mut buf = String::new();
        io::Read::read_to_string(&mut entry, &mut buf)
            .map_err(|e| format!("Failed to read SKILL.md content: {e}"))?;
        buf
    };

    let (fm, _) = parse_skill_md(&content)
        .ok_or("SKILL.md in zip must have valid YAML frontmatter")?;

    if fm.name.is_empty() {
        return Err("SKILL.md must have a name field".to_string());
    }

    let skill_name = sanitize_upload_name(&fm.name);
    if skill_name.is_empty() {
        return Err("Skill name contains only invalid characters".to_string());
    }
    let dest_dir = skills_base_dir().join(&skill_name);

    if dest_dir.exists() {
        return Err(format!("Skill '{}' already exists", skill_name));
    }

    fs::create_dir_all(&dest_dir)
        .map_err(|e| format!("Failed to create dest dir: {e}"))?;
    let canonical_dest = dunce::canonicalize(&dest_dir)
        .map_err(|e| format!("Failed to resolve dest dir: {e}"))?;

    const MAX_EXTRACT_SIZE: u64 = 50 * 1024 * 1024; // 50 MB
    const MAX_FILE_COUNT: usize = 500;
    let mut total_extracted: u64 = 0;
    let mut file_count: usize = 0;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)
            .map_err(|e| format!("Failed to read zip entry: {e}"))?;

        let entry_name = entry.name().to_string();
        if !entry_name.starts_with(&prefix) {
            continue;
        }

        let relative = &entry_name[prefix.len()..];
        if relative.is_empty() {
            continue;
        }
        let rel_path = std::path::Path::new(relative);
        if rel_path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
            continue;
        }

        let out_path = canonical_dest.join(relative);
        if !out_path.starts_with(&canonical_dest) {
            continue;
        }

        file_count += 1;
        if file_count > MAX_FILE_COUNT {
            let _ = fs::remove_dir_all(&dest_dir);
            return Err("ZIP contains too many files (limit: 500)".to_string());
        }

        if entry.is_dir() {
            fs::create_dir_all(&out_path)
                .map_err(|e| format!("Failed to create dir: {e}"))?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent dir: {e}"))?;
            }
            let mut outfile = fs::File::create(&out_path)
                .map_err(|e| format!("Failed to create file: {e}"))?;
            let mut limited = io::Read::take(&mut entry, MAX_EXTRACT_SIZE - total_extracted);
            let written = io::copy(&mut limited, &mut outfile)
                .map_err(|e| format!("Failed to extract file: {e}"))?;
            total_extracted += written;
            if total_extracted >= MAX_EXTRACT_SIZE {
                let _ = fs::remove_dir_all(&dest_dir);
                return Err("ZIP extraction exceeded 50 MB size limit".to_string());
            }
        }
    }

    let _ = mark_skill_source(&skill_name, "uploaded");
    Ok(skill_name)
}
