use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use crate::models::{Skill, SkillFile, SkillFrontmatter, SkillSource, parse_skill_md};
use crate::services::skill_io;

fn birth_time_secs(path: &Path) -> Option<i64> {
    let meta = fs::metadata(path).ok()?;
    let created = meta.created().ok()?;
    let dur = created.duration_since(SystemTime::UNIX_EPOCH).ok()?;
    Some(dur.as_secs() as i64)
}

fn home_dir() -> PathBuf {
    dirs::home_dir()
        .or_else(|| std::env::var_os("HOME").map(PathBuf::from))
        .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn skills_base_dir() -> PathBuf {
    let base = home_dir().join(".claude").join("skills");
    if !base.exists() {
        let _ = fs::create_dir_all(&base);
    }
    base
}

pub fn claude_install_status() -> ClaudeStatus {
    let claude_dir = home_dir().join(".claude");
    if !claude_dir.exists() {
        return ClaudeStatus::NotInstalled;
    }
    let has_settings = claude_dir.join("settings.json").exists();
    let has_credentials = claude_dir.join("credentials.json").exists()
        || claude_dir.join(".credentials.json").exists();
    if has_settings || has_credentials {
        ClaudeStatus::Installed
    } else {
        ClaudeStatus::DirOnly
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClaudeStatus {
    Installed,
    DirOnly,
    NotInstalled,
}

pub fn scan_skills() -> Vec<Skill> {
    let base = skills_base_dir();
    let initialized = skill_io::is_tracking_initialized();
    let mut skill_sources = skill_io::load_skill_sources();
    let mut skills = Vec::new();

    scan_directory(&base, true, &skill_sources, &mut skills);

    let disabled_dir = base.join(".disabled");
    if disabled_dir.exists() {
        scan_directory(&disabled_dir, false, &skill_sources, &mut skills);
    }

    let mut changed = false;

    if initialized {
        for skill in &mut skills {
            if skill.source == SkillSource::Linked {
                continue;
            }
            if !skill_sources.contains_key(&skill.dir_name) {
                if skill.path.join(".git").exists() {
                    skill.source = SkillSource::Downloaded;
                    skill.source_locked = true;
                    skill_sources.insert(skill.dir_name.clone(), "git-clone".to_string());
                } else {
                    skill.source = SkillSource::Custom;
                    skill_sources.insert(skill.dir_name.clone(), "custom".to_string());
                }
                changed = true;
            }
        }
    } else {
        for skill in &skills {
            if skill.source == SkillSource::Linked {
                continue;
            }
            if !skill_sources.contains_key(&skill.dir_name) {
                let source_str = match &skill.source {
                    SkillSource::Custom => "custom",
                    _ => "downloaded",
                };
                skill_sources.insert(skill.dir_name.clone(), source_str.to_string());
                changed = true;
            }
        }
        skill_io::mark_tracking_initialized();
    }

    if changed {
        let _ = skill_io::save_skill_sources(&skill_sources);
    }

    skills.sort_by(|a, b| a.dir_name.to_lowercase().cmp(&b.dir_name.to_lowercase()));
    skills
}

fn scan_directory(dir: &Path, enabled: bool, skill_sources: &HashMap<String, String>, skills: &mut Vec<Skill>) {
    let entries: Vec<_> = match fs::read_dir(dir) {
        Ok(e) => e.flatten().collect(),
        Err(_) => return,
    };

    let mut symlink_containers: HashSet<String> = HashSet::new();
    for entry in &entries {
        let metadata = match fs::symlink_metadata(&entry.path()) {
            Ok(m) => m,
            Err(_) => continue,
        };
        if metadata.is_symlink() {
            if let Ok(target) = fs::read_link(entry.path()) {
                // Only consider relative symlink targets for container detection.
                // Absolute symlinks don't imply a sibling directory container.
                if target.is_relative() {
                    if let Some(std::path::Component::Normal(first)) = target.components().next() {
                        let prefix = first.to_string_lossy().to_string();
                        if !prefix.starts_with('.') {
                            symlink_containers.insert(prefix);
                        }
                    }
                }
            }
        }
    }

    for entry in entries {
        let file_name = entry.file_name().to_string_lossy().to_string();

        if file_name.starts_with('.') || file_name == "node_modules" {
            continue;
        }

        let entry_path = entry.path();
        let metadata = match fs::symlink_metadata(&entry_path) {
            Ok(m) => m,
            Err(_) => continue,
        };

        if metadata.is_symlink() {
            if let Some(mut skill) = load_symlink_skill(&entry_path, &file_name, enabled) {
                apply_source(&mut skill, skill_sources);
                skills.push(skill);
            }
        } else if metadata.is_dir() {
            if symlink_containers.contains(&file_name) {
                continue;
            }
            if let Some(mut skill) = load_dir_skill(&entry_path, &file_name, SkillSource::Downloaded, enabled) {
                apply_source(&mut skill, skill_sources);
                skills.push(skill);
            }
        } else if metadata.is_file() && file_name.ends_with(".md") && file_name != "SKILL.md" {
            if let Some(mut skill) = load_loose_file(&entry_path, &file_name, enabled) {
                apply_source(&mut skill, skill_sources);
                skills.push(skill);
            }
        }
    }
}

fn apply_source(skill: &mut Skill, skill_sources: &HashMap<String, String>) {
    if skill.source == SkillSource::Linked {
        skill.source_locked = true;
        return;
    }
    if let Some(source_str) = skill_sources.get(&skill.dir_name) {
        match source_str.as_str() {
            "custom" => skill.source = SkillSource::Custom,
            "marketplace" | "git-clone" => {
                skill.source = SkillSource::Downloaded;
                skill.source_locked = true;
            }
            "downloaded" | "uploaded" => skill.source = SkillSource::Downloaded,
            _ => {}
        }
    }
}

fn load_symlink_skill(link_path: &Path, name: &str, enabled: bool) -> Option<Skill> {
    let target = fs::read_link(link_path).ok()?;

    // Determine the logical container (first relative path component).
    // For absolute symlink targets, use the parent directory name of the resolved path.
    // For relative targets, use the first component (the containing directory name).
    let container = if target.is_absolute() {
        // For absolute symlinks, the meaningful "container" is the parent directory name
        dunce::canonicalize(link_path).ok()
            .and_then(|p| p.parent().map(|par| par.to_path_buf()))
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
    } else {
        use std::path::Component;
        target.components().next().and_then(|c| {
            match c {
                Component::Normal(name) => Some(name.to_string_lossy().to_string()),
                _ => None, // skip `.`, `..`, prefix, root components
            }
        })
    };

    let resolved = dunce::canonicalize(link_path).ok()?;
    if !resolved.is_dir() {
        return None;
    }

    let mut skill = load_dir_skill_with_link(&resolved, link_path, name, SkillSource::Linked, enabled)?;
    skill.linked_from = container;
    Some(skill)
}

fn load_dir_skill(dir: &Path, name: &str, source: SkillSource, enabled: bool) -> Option<Skill> {
    load_dir_skill_with_link(dir, dir, name, source, enabled)
}

fn load_dir_skill_with_link(
    dir: &Path,
    link_path: &Path,
    name: &str,
    source: SkillSource,
    enabled: bool,
) -> Option<Skill> {
    let skill_md = dir.join("SKILL.md");
    let content = fs::read_to_string(&skill_md).ok()?;
    let (frontmatter, body) = parse_skill_md(&content)?;

    let (files, file_count, total_size) = collect_files(dir);
    let installed_at = birth_time_secs(&skill_md);

    Some(Skill {
        dir_name: name.to_string(),
        path: dir.to_path_buf(),
        link_path: link_path.to_path_buf(),
        frontmatter,
        body,
        source,
        source_locked: false,
        linked_from: None,
        enabled,
        file_count,
        total_size,
        files,
        installed_at,
    })
}

fn load_loose_file(path: &Path, file_name: &str, enabled: bool) -> Option<Skill> {
    let content = fs::read_to_string(path).ok()?;
    let name = file_name.trim_end_matches(".md").to_string();

    let (frontmatter, body) = if let Some((fm, b)) = parse_skill_md(&content) {
        (fm, b)
    } else {
        let first_line = content.lines().next().unwrap_or(&name);
        let desc_name = first_line.trim_start_matches('#').trim().to_string();
        (
            SkillFrontmatter {
                name: desc_name,
                description: content.lines().skip(1).take(3).collect::<Vec<_>>().join(" ").trim().to_string(),
                ..Default::default()
            },
            content,
        )
    };

    let size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    let installed_at = birth_time_secs(path);

    Some(Skill {
        dir_name: name,
        path: path.to_path_buf(),
        link_path: path.to_path_buf(),
        frontmatter,
        body,
        source: SkillSource::LooseFile,
        source_locked: false,
        linked_from: None,
        enabled,
        file_count: 1,
        total_size: size,
        files: vec![SkillFile {
            relative_path: file_name.to_string(),
            size,
            is_dir: false,
        }],
        installed_at,
    })
}

fn collect_files(dir: &Path) -> (Vec<SkillFile>, usize, u64) {
    let mut files = Vec::new();
    let mut total_size = 0u64;
    collect_files_recursive(dir, dir, &mut files, &mut total_size, 0);
    // Sort so children appear directly after their parent directory
    // Key: for each file, build a sort key where each path segment gets
    // "0_name" for dirs and "1_name" for files, ensuring dirs come first
    files.sort_by(|a, b| {
        let a_key = sort_key(&a.relative_path, a.is_dir);
        let b_key = sort_key(&b.relative_path, b.is_dir);
        a_key.cmp(&b_key)
    });
    let count = files.iter().filter(|f| !f.is_dir).count();
    (files, count, total_size)
}

fn sort_key(path: &str, is_dir: bool) -> String {
    let parts: Vec<&str> = path.split('/').collect();
    let mut key_parts: Vec<String> = Vec::new();
    for (i, part) in parts.iter().enumerate() {
        let is_last = i == parts.len() - 1;
        let prefix = if is_last && !is_dir { "1" } else { "0" };
        key_parts.push(format!("{}_{}", prefix, part.to_lowercase()));
    }
    key_parts.join("/")
}

const MAX_SCAN_DEPTH: usize = 10;

fn collect_files_recursive(
    base: &Path,
    current: &Path,
    files: &mut Vec<SkillFile>,
    total_size: &mut u64,
    depth: usize,
) {
    if depth > MAX_SCAN_DEPTH {
        return;
    }
    let entries = match fs::read_dir(current) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if name.starts_with('.') || name == "node_modules" {
            continue;
        }

        let relative = path
            .strip_prefix(base)
            .unwrap_or(&path)
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("/");

        if path.is_dir() {
            files.push(SkillFile {
                relative_path: relative.clone(),
                size: 0,
                is_dir: true,
            });
            collect_files_recursive(base, &path, files, total_size, depth + 1);
        } else if let Ok(meta) = fs::metadata(&path) {
            let size = meta.len();
            *total_size += size;
            files.push(SkillFile {
                relative_path: relative,
                size,
                is_dir: false,
            });
        }
    }
}
