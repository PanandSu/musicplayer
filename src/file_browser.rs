use crate::metadata::is_supported_audio_file;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemType {
    ParentDir,
    Directory,
    AudioFile(String),
}

#[derive(Debug, Clone)]
pub struct BrowserItem {
    pub name: String,
    pub path: PathBuf,
    pub item_type: ItemType,
}

pub struct FileBrowser {
    pub current_dir: PathBuf,
    pub items: Vec<BrowserItem>,
    pub selected_idx: usize,
    pub filter: String,
}

impl FileBrowser {
    pub fn new(start_dir: Option<PathBuf>) -> Self {
        let dir = start_dir
            .and_then(|p| fs::canonicalize(p).ok())
            .unwrap_or_else(|| {
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
            });

        let mut browser = Self {
            current_dir: dir,
            items: Vec::new(),
            selected_idx: 0,
            filter: String::new(),
        };
        browser.refresh();
        browser
    }

    pub fn refresh(&mut self) {
        self.items.clear();

        // Add parent directory item if not root
        if self.current_dir.parent().is_some() {
            self.items.push(BrowserItem {
                name: "..".to_string(),
                path: self.current_dir.join(".."),
                item_type: ItemType::ParentDir,
            });
        }

        let mut dirs = Vec::new();
        let mut audio_files = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.current_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let file_name = entry.file_name().to_string_lossy().to_string();

                // Skip hidden files/directories unless explicitly searched
                if file_name.starts_with('.') && file_name != ".." {
                    continue;
                }

                if path.is_dir() {
                    dirs.push(BrowserItem {
                        name: file_name,
                        path,
                        item_type: ItemType::Directory,
                    });
                } else if is_supported_audio_file(&path) {
                    let ext = path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("AUDIO")
                        .to_uppercase();
                    audio_files.push(BrowserItem {
                        name: file_name,
                        path,
                        item_type: ItemType::AudioFile(ext),
                    });
                }
            }
        }

        dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        audio_files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        self.items.extend(dirs);
        self.items.extend(audio_files);

        if self.selected_idx >= self.filtered_items().len() {
            self.selected_idx = self.filtered_items().len().saturating_sub(1);
        }
    }

    pub fn filtered_items(&self) -> Vec<&BrowserItem> {
        if self.filter.is_empty() {
            self.items.iter().collect()
        } else {
            let lower_filter = self.filter.to_lowercase();
            self.items
                .iter()
                .filter(|item| {
                    item.item_type == ItemType::ParentDir
                        || item.name.to_lowercase().contains(&lower_filter)
                })
                .collect()
        }
    }

    pub fn selected_item(&self) -> Option<&BrowserItem> {
        let filtered = self.filtered_items();
        filtered.get(self.selected_idx).copied()
    }

    pub fn select_next(&mut self) {
        let count = self.filtered_items().len();
        if count > 0 && self.selected_idx + 1 < count {
            self.selected_idx += 1;
        }
    }

    pub fn select_prev(&mut self) {
        if self.selected_idx > 0 {
            self.selected_idx -= 1;
        }
    }

    pub fn select_page_down(&mut self, step: usize) {
        let count = self.filtered_items().len();
        if count > 0 {
            self.selected_idx = (self.selected_idx + step).min(count - 1);
        }
    }

    pub fn select_page_up(&mut self, step: usize) {
        self.selected_idx = self.selected_idx.saturating_sub(step);
    }

    pub fn select_first(&mut self) {
        self.selected_idx = 0;
    }

    pub fn select_last(&mut self) {
        let count = self.filtered_items().len();
        if count > 0 {
            self.selected_idx = count - 1;
        }
    }

    pub fn set_filter(&mut self, query: String) {
        self.filter = query;
        self.selected_idx = 0;
    }

    pub fn clear_filter(&mut self) {
        self.filter.clear();
        self.selected_idx = 0;
    }

    /// Enter directory or return audio file path to play
    pub fn activate_selected(&mut self) -> Option<PathBuf> {
        let item = self.selected_item().cloned()?;
        match item.item_type {
            ItemType::ParentDir => {
                if let Some(parent) = self.current_dir.parent() {
                    let parent_buf = parent.to_path_buf();
                    self.current_dir = parent_buf;
                    self.clear_filter();
                    self.refresh();
                }
                None
            }
            ItemType::Directory => {
                self.current_dir = item.path;
                self.clear_filter();
                self.refresh();
                None
            }
            ItemType::AudioFile(_) => Some(item.path),
        }
    }

    pub fn go_up(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            let parent_buf = parent.to_path_buf();
            self.current_dir = parent_buf;
            self.clear_filter();
            self.refresh();
        }
    }

    pub fn collect_all_audio_in_dir(&self) -> Vec<PathBuf> {
        self.items
            .iter()
            .filter_map(|item| {
                if matches!(item.item_type, ItemType::AudioFile(_)) {
                    Some(item.path.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}
