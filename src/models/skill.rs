use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillSource {
    Custom,
    Downloaded,
    Linked,
    LooseFile,
}

impl SkillSource {
    pub fn label(&self) -> &'static str {
        match self {
            SkillSource::Custom => "My Skill",
            SkillSource::Downloaded => "Downloaded",
            SkillSource::Linked => "Linked",
            SkillSource::LooseFile => "File",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SkillFrontmatter {
    pub name: String,
    pub description: String,
    #[serde(default, rename = "preamble-tier", skip_serializing_if = "Option::is_none")]
    pub preamble_tier: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, rename = "allowed-tools", skip_serializing_if = "Option::is_none")]
    pub allowed_tools: Option<Vec<String>>,
    #[serde(default, rename = "user-invocable", skip_serializing_if = "Option::is_none")]
    pub user_invocable: Option<bool>,
    #[serde(default, rename = "argument-hint", skip_serializing_if = "Option::is_none")]
    pub argument_hint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hooks: Option<serde_yaml::Value>,
    #[serde(default, rename = "benefits-from", skip_serializing_if = "Option::is_none")]
    pub benefits_from: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_yaml::Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SkillFile {
    pub relative_path: String,
    pub size: u64,
    pub is_dir: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Skill {
    pub dir_name: String,
    pub path: PathBuf,
    pub link_path: PathBuf,
    pub frontmatter: SkillFrontmatter,
    pub body: String,
    pub source: SkillSource,
    pub source_locked: bool,
    pub linked_from: Option<String>,
    pub enabled: bool,
    pub file_count: usize,
    pub total_size: u64,
    pub files: Vec<SkillFile>,
    pub installed_at: Option<i64>,
}

impl Skill {
    pub fn description_excerpt(&self, max_len: usize) -> String {
        let desc = &self.frontmatter.description;
        if desc.chars().count() <= max_len {
            desc.clone()
        } else {
            let truncated: String = desc.chars().take(max_len).collect();
            format!("{}…", truncated.trim_end())
        }
    }

    pub fn human_size(&self) -> String {
        bytesize::ByteSize(self.total_size).to_string()
    }

    pub fn formatted_installed_date(&self) -> Option<String> {
        let ts = self.installed_at?;
        let dt = chrono::DateTime::from_timestamp(ts, 0)?;
        Some(dt.format("%Y-%m-%d").to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillFilter {
    All,
    Custom,
    Downloaded,
    Linked,
    Other,
    Enabled,
    Disabled,
}

impl SkillFilter {
    pub fn label(&self) -> &'static str {
        match self {
            SkillFilter::All => "All",
            SkillFilter::Custom => "My Skills",
            SkillFilter::Downloaded => "Downloaded",
            SkillFilter::Linked => "Linked",
            SkillFilter::Other => "Other",
            SkillFilter::Enabled => "Enabled",
            SkillFilter::Disabled => "Disabled",
        }
    }

    pub fn i18n_key(&self) -> &'static str {
        match self {
            SkillFilter::All => "sidebar.all_skills",
            SkillFilter::Custom => "sidebar.my_skills",
            SkillFilter::Downloaded => "sidebar.downloaded",
            SkillFilter::Linked => "sidebar.linked",
            SkillFilter::Other => "sidebar.other",
            SkillFilter::Enabled => "sidebar.enabled",
            SkillFilter::Disabled => "sidebar.disabled",
        }
    }
}
