use std::collections::{HashMap, HashSet, VecDeque};
use crate::models::{Skill, SkillFilter, SkillSource};
use crate::services::scanner;
use crate::services::marketplace::{MarketplaceSkill, GithubFile};

const MAX_DRAWER_CACHE: usize = 20;
const MAX_SEARCH_CACHE: usize = 10;

#[derive(Clone)]
pub struct CachedSearchResult {
    pub skills: Vec<MarketplaceSkill>,
    pub page: u32,
    pub has_next: bool,
}

#[derive(Clone)]
pub struct DrawerCacheEntry {
    pub content: Option<String>,
    pub files: Option<Vec<GithubFile>>,
}

#[derive(Clone)]
pub struct MarketplaceCache {
    pub results: HashMap<(String, String), CachedSearchResult>,
    results_order: VecDeque<(String, String)>,
    pub drawer_cache: HashMap<String, DrawerCacheEntry>,
    drawer_order: VecDeque<String>,
    pub last_query: String,
    pub last_sort: String,
}

impl MarketplaceCache {
    pub fn new() -> Self {
        Self {
            results: HashMap::new(),
            results_order: VecDeque::new(),
            drawer_cache: HashMap::new(),
            drawer_order: VecDeque::new(),
            last_query: String::new(),
            last_sort: "stars".to_string(),
        }
    }

    pub fn get(&self, query: &str, sort: &str) -> Option<&CachedSearchResult> {
        self.results.get(&(query.to_string(), sort.to_string()))
    }

    pub fn set(&mut self, query: &str, sort: &str, skills: Vec<MarketplaceSkill>, page: u32, has_next: bool) {
        let key = (query.to_string(), sort.to_string());
        if !self.results.contains_key(&key) {
            if self.results_order.len() >= MAX_SEARCH_CACHE {
                if let Some(oldest) = self.results_order.pop_front() {
                    self.results.remove(&oldest);
                }
            }
            self.results_order.push_back(key.clone());
        } else {
            self.results_order.retain(|k| k != &key);
            self.results_order.push_back(key.clone());
        }
        self.results.insert(key, CachedSearchResult { skills, page, has_next });
        self.last_query = query.to_string();
        self.last_sort = sort.to_string();
    }

    pub fn get_drawer(&self, skill_id: &str) -> Option<&DrawerCacheEntry> {
        self.drawer_cache.get(skill_id)
    }

    pub fn set_drawer_content(&mut self, skill_id: &str, content: String) {
        self.touch_drawer(skill_id);
        self.drawer_cache.entry(skill_id.to_string())
            .or_insert_with(|| DrawerCacheEntry { content: None, files: None })
            .content = Some(content);
    }

    pub fn set_drawer_files(&mut self, skill_id: &str, files: Vec<GithubFile>) {
        self.touch_drawer(skill_id);
        self.drawer_cache.entry(skill_id.to_string())
            .or_insert_with(|| DrawerCacheEntry { content: None, files: None })
            .files = Some(files);
    }

    fn touch_drawer(&mut self, skill_id: &str) {
        let id = skill_id.to_string();
        if self.drawer_cache.contains_key(&id) {
            self.drawer_order.retain(|k| k != &id);
        } else if self.drawer_order.len() >= MAX_DRAWER_CACHE {
            if let Some(oldest) = self.drawer_order.pop_front() {
                self.drawer_cache.remove(&oldest);
            }
        }
        self.drawer_order.push_back(id);
    }
}

#[derive(Clone, PartialEq)]
pub enum AppView {
    Skills,
    Marketplace,
    Dashboard,
}

#[derive(Clone)]
pub struct SkillsState {
    pub skills: Vec<Skill>,
    pub search_query: String,
    pub filter: SkillFilter,
    pub select_mode: bool,
    pub selected: HashSet<String>,
    pub detail_skill: Option<String>,
    pub marketplace_preview: Option<MarketplaceSkill>,
    pub view: AppView,
    pub show_upload: bool,
}

impl SkillsState {
    pub fn new() -> Self {
        Self {
            skills: scanner::scan_skills(),
            search_query: String::new(),
            filter: SkillFilter::All,
            select_mode: false,
            selected: HashSet::new(),
            detail_skill: None,
            marketplace_preview: None,
            view: AppView::Skills,
            show_upload: false,
        }
    }

    pub fn reload(&mut self) {
        self.skills = scanner::scan_skills();
        self.selected.retain(|name| self.skills.iter().any(|s| &s.dir_name == name));
    }

    pub fn toggle_selected(&mut self, name: &str) {
        if self.selected.contains(name) {
            self.selected.remove(name);
            if self.selected.is_empty() {
                self.select_mode = false;
            }
        } else {
            self.selected.insert(name.to_string());
            self.select_mode = true;
        }
    }

    pub fn clear_selection(&mut self) {
        self.selected.clear();
        self.select_mode = false;
    }

    pub fn select_all(&mut self, names: &[String]) {
        for name in names {
            self.selected.insert(name.clone());
        }
        if !self.selected.is_empty() {
            self.select_mode = true;
        }
    }

    pub fn is_selected(&self, name: &str) -> bool {
        self.selected.contains(name)
    }

    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }

    pub fn filtered_skills(&self) -> Vec<&Skill> {
        self.skills
            .iter()
            .filter(|s| {
                let q = self.search_query.to_lowercase();
                if !q.is_empty() {
                    let name_match = s.frontmatter.name.to_lowercase().contains(&q);
                    let desc_match = s.frontmatter.description.to_lowercase().contains(&q);
                    let dir_match = s.dir_name.to_lowercase().contains(&q);
                    if !(name_match || desc_match || dir_match) {
                        return false;
                    }
                }
                match self.filter {
                    SkillFilter::All => true,
                    SkillFilter::Custom => s.source == SkillSource::Custom,
                    SkillFilter::Downloaded => s.source != SkillSource::Custom,
                    SkillFilter::Linked => s.source == SkillSource::Linked,
                    SkillFilter::Other => s.source == SkillSource::Downloaded || s.source == SkillSource::LooseFile,
                    SkillFilter::Enabled => s.enabled,
                    SkillFilter::Disabled => !s.enabled,
                }
            })
            .collect()
    }

    pub fn find_skill(&self, name: &str) -> Option<&Skill> {
        self.skills.iter().find(|s| s.dir_name == name)
    }

    pub fn total_count(&self) -> usize {
        self.skills.len()
    }

    pub fn enabled_count(&self) -> usize {
        self.skills.iter().filter(|s| s.enabled).count()
    }
}
