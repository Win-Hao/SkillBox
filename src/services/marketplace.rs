use serde::Deserialize;
use std::fs;
use std::time::Duration;
use crate::services::scanner::skills_base_dir;
use crate::services::skill_io;

const API_BASE: &str = "https://skillsmp.com/api/skills";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_API_RESPONSE: u64 = 2 * 1024 * 1024;
const MAX_SKILL_CONTENT: u64 = 5 * 1024 * 1024;
const MAX_TREE_REQUESTS: usize = 50;

fn check_response_size(resp: &reqwest::Response, max_size: u64) -> Result<(), String> {
    if let Some(len) = resp.content_length() {
        if len > max_size {
            return Err(format!("Response too large ({} bytes)", len));
        }
    }
    Ok(())
}

fn http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .connect_timeout(CONNECT_TIMEOUT)
        .user_agent("skills-manager")
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct MarketplaceSkill {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, rename = "githubUrl")]
    pub github_url: Option<String>,
    #[serde(default)]
    pub stars: Option<u64>,
    #[serde(default)]
    pub forks: Option<u64>,
    #[serde(default, rename = "updatedAt")]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SearchResponse {
    pub skills: Vec<MarketplaceSkill>,
    #[serde(default)]
    pub pagination: Option<Pagination>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Pagination {
    #[serde(default)]
    pub page: u32,
    #[serde(default)]
    pub total: u64,
    #[serde(default, rename = "totalPages")]
    pub total_pages: u32,
    #[serde(default, rename = "hasNext")]
    pub has_next: bool,
    #[serde(default, rename = "totalIsExact")]
    pub total_is_exact: bool,
}

impl MarketplaceSkill {
    pub fn formatted_date(&self) -> Option<String> {
        let ts: i64 = self.updated_at.as_ref()?.parse().ok()?;
        let secs = chrono::DateTime::from_timestamp(ts, 0)?;
        Some(secs.format("%Y-%m-%d").to_string())
    }
}

pub async fn search_skills(query: &str, sort: &str, page: u32) -> Result<SearchResponse, String> {
    let client = http_client()?;
    let mut req = client.get(API_BASE)
        .query(&[("sortBy", sort), ("limit", "20"), ("page", &page.to_string())]);
    if !query.is_empty() {
        req = req.query(&[("search", query)]);
    }
    let resp = req.send().await.map_err(|e| format!("Request failed: {e}"))?;
    check_response_size(&resp, MAX_API_RESPONSE)?;
    resp.json::<SearchResponse>().await.map_err(|e| format!("Parse failed: {e}"))
}

pub async fn install_skill(skill: &MarketplaceSkill) -> Result<String, String> {
    let github_url = skill.github_url.as_ref()
        .ok_or("No GitHub URL available")?;

    let raw_url = github_to_raw_url(github_url, skill.branch.as_deref().unwrap_or("main"));
    let client = http_client()?;
    let resp = client.get(&raw_url).send().await
        .map_err(|e| format!("Failed to fetch SKILL.md: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("Failed to fetch SKILL.md: HTTP {}", resp.status()));
    }
    check_response_size(&resp, MAX_SKILL_CONTENT)?;
    let content = resp.text().await
        .map_err(|e| format!("Failed to read response: {e}"))?;

    if !content.contains("---") {
        return Err("Invalid SKILL.md: no frontmatter found".to_string());
    }

    let skill_name = sanitize_name(&skill.name);
    if skill_name.is_empty() {
        return Err("Skill name contains only invalid characters".to_string());
    }
    let dest = skills_base_dir().join(&skill_name);

    if dest.exists() {
        return Err(format!("Skill '{}' already exists", skill_name));
    }

    fs::create_dir_all(&dest).map_err(|e| format!("Failed to create directory: {e}"))?;
    fs::write(dest.join("SKILL.md"), &content).map_err(|e| format!("Failed to write SKILL.md: {e}"))?;

    let _ = skill_io::mark_skill_source(&skill_name, "marketplace");
    Ok(skill_name)
}

pub async fn fetch_skill_content(skill: &MarketplaceSkill) -> Result<String, String> {
    let github_url = skill.github_url.as_ref()
        .ok_or("No GitHub URL available")?;
    let raw_url = github_to_raw_url(github_url, skill.branch.as_deref().unwrap_or("main"));
    let client = http_client()?;
    let resp = client.get(&raw_url).send().await
        .map_err(|e| format!("Failed to fetch: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("Failed to fetch content: HTTP {}", resp.status()));
    }
    check_response_size(&resp, MAX_SKILL_CONTENT)?;
    let content = resp.text().await
        .map_err(|e| format!("Failed to read: {e}"))?;
    Ok(content)
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct GithubFile {
    pub name: String,
    #[serde(default)]
    pub path: String,
    #[serde(default, rename = "type")]
    pub file_type: String,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub download_url: Option<String>,
}

pub async fn fetch_full_tree(skill: &MarketplaceSkill) -> Result<Vec<GithubFile>, String> {
    let github_url = skill.github_url.as_ref().ok_or("No GitHub URL")?;
    let (owner, repo, skill_path) = parse_github_url(github_url)?;
    let branch = skill.branch.as_deref().unwrap_or("main");
    let base_dir = skill_path.trim_end_matches("/SKILL.md").trim_end_matches('/');

    let client = http_client()?;
    let mut all_files: Vec<GithubFile> = Vec::new();

    let mut queue: Vec<(String, String)> = vec![(base_dir.to_string(), String::new())];
    let max_depth = 5;
    let mut depth = 0;
    let mut total_requests: usize = 0;

    while !queue.is_empty() && depth < max_depth && total_requests < MAX_TREE_REQUESTS {
        depth += 1;
        let batch: Vec<(String, String)> = queue.drain(..).collect();
        total_requests += batch.len();
        let mut handles = Vec::new();

        for (full_path, rel_prefix) in batch {
            let url = format!(
                "https://api.github.com/repos/{}/{}/contents/{}?ref={}",
                owner, repo, full_path, branch
            );
            let client = client.clone();
            handles.push(tokio::spawn(async move {
                let resp = client.get(&url)
                    .send().await.ok()?;
                if !resp.status().is_success() {
                    return None;
                }
                if check_response_size(&resp, MAX_API_RESPONSE).is_err() {
                    return None;
                }
                let files: Vec<GithubFile> = resp.json().await.ok()?;
                Some((rel_prefix, files))
            }));
        }

        for handle in handles {
            if let Ok(Some((rel_prefix, entries))) = handle.await {
                for f in entries {
                    let mut file = f.clone();
                    let rel_path = if rel_prefix.is_empty() {
                        f.name.clone()
                    } else {
                        format!("{}/{}", rel_prefix, f.name)
                    };
                    if file.file_type == "dir" {
                        queue.push((f.path.clone(), rel_path.clone()));
                    }
                    file.path = rel_path;
                    all_files.push(file);
                }
            }
        }
    }

    // Sort in tree order: directories before files at each level, then alphabetical.
    // Use a sort key that appends a trailing slash to directories so they group before files.
    all_files.sort_by(|a, b| {
        let key = |f: &GithubFile| -> String {
            let parts: Vec<&str> = f.path.split('/').collect();
            let mut segs: Vec<String> = Vec::with_capacity(parts.len());
            for (i, part) in parts.iter().enumerate() {
                let is_last = i == parts.len() - 1;
                if is_last && f.file_type != "dir" {
                    // Files sort after dirs: prefix with '1'
                    segs.push(format!("1{}", part));
                } else {
                    // Dirs sort first: prefix with '0'
                    segs.push(format!("0{}", part));
                }
            }
            segs.join("/")
        };
        key(a).cmp(&key(b))
    });

    Ok(all_files)
}

pub async fn fetch_github_file_content(url: &str) -> Result<String, String> {
    if !url.starts_with("https://raw.githubusercontent.com/")
        && !url.starts_with("https://github.com/")
        && !url.starts_with("https://objects.githubusercontent.com/")
    {
        return Err(format!("Blocked fetch from untrusted domain"));
    }
    let client = http_client()?;
    let resp = client.get(url)
        .send().await
        .map_err(|e| format!("Fetch error: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("Fetch error: HTTP {}", resp.status()));
    }
    check_response_size(&resp, MAX_SKILL_CONTENT)?;
    let content = resp.text().await
        .map_err(|e| format!("Read error: {e}"))?;
    Ok(content)
}

fn parse_github_url(url: &str) -> Result<(String, String, String), String> {
    // https://github.com/owner/repo/tree/branch/path/to/dir
    // → (owner, repo, path/to/dir)
    let stripped = url
        .trim_start_matches("https://github.com/")
        .trim_start_matches("http://github.com/");
    let parts: Vec<&str> = stripped.splitn(5, '/').collect();
    if parts.len() < 2 {
        return Err("Invalid GitHub URL".to_string());
    }
    let owner = parts[0].to_string();
    let repo = parts[1].to_string();
    // parts[2] = "tree" or "blob", parts[3] = branch, parts[4] = actual path
    let path = if parts.len() >= 5 {
        parts[4].to_string()
    } else {
        String::new()
    };
    Ok((owner, repo, path))
}

fn github_to_raw_url(github_url: &str, branch: &str) -> String {
    // https://github.com/owner/repo/blob/main/path/SKILL.md
    // → https://raw.githubusercontent.com/owner/repo/{branch}/path/SKILL.md
    //
    // Parse the URL to extract owner/repo and file path, then reconstruct
    // with the provided branch (from the API response) rather than relying
    // on whatever branch name appears in the URL.
    let stripped = github_url
        .trim_start_matches("https://github.com/")
        .trim_start_matches("http://github.com/");
    let parts: Vec<&str> = stripped.splitn(5, '/').collect();

    if parts.len() >= 5 {
        // parts = [owner, repo, "blob"|"tree", url_branch, path...]
        let owner = parts[0];
        let repo = parts[1];
        let path = parts[4];
        let raw_path = if path.ends_with("SKILL.md") {
            path.to_string()
        } else {
            format!("{}/SKILL.md", path.trim_end_matches('/'))
        };
        format!("https://raw.githubusercontent.com/{}/{}/{}/{}", owner, repo, branch, raw_path)
    } else if parts.len() >= 2 {
        // Minimal URL with just owner/repo — use branch and assume SKILL.md at root
        let owner = parts[0];
        let repo = parts[1];
        format!("https://raw.githubusercontent.com/{}/{}/{}/SKILL.md", owner, repo, branch)
    } else {
        // URL doesn't match expected GitHub format; return empty to trigger fetch failure
        String::new()
    }
}

pub fn check_compatibility(files: &[GithubFile], skill_content: &str) -> Option<String> {
    let platform_files: Vec<&str> = files.iter()
        .filter_map(|f| {
            let name = f.name.as_str();
            if name == "openai.yaml" || name == "codex.yaml" || name == "cursor.yaml"
                || name == "gemini.yaml" || name == "windsurf.yaml"
                || (f.path.starts_with("agents/") && f.file_type == "file" && f.name != "SKILL.md")
            {
                Some(name)
            } else {
                None
            }
        })
        .collect();

    let content_lower = skill_content.to_lowercase();
    let has_platform_refs = content_lower.contains("codex cli")
        || content_lower.contains("cursor agent")
        || content_lower.contains("openai codex");

    if !platform_files.is_empty() {
        Some(format!("Contains platform-specific files: {}", platform_files.join(", ")))
    } else if has_platform_refs {
        Some("May reference non-Claude platform features".to_string())
    } else {
        None
    }
}

fn sanitize_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}
