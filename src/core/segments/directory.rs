use super::{Segment, SegmentData};
use crate::config::{InputData, SegmentId, DirectorySegmentConfig, CaseStyle};
use std::collections::HashMap;
use std::path::Path;

#[derive(Default)]
pub struct DirectorySegment {
    config: DirectorySegmentConfig,
}

impl DirectorySegment {
    pub fn new() -> Self {
        Self {
            config: DirectorySegmentConfig::default(),
        }
    }
    
    pub fn with_config(mut self, options: &HashMap<String, serde_json::Value>) -> Self {
        self.config = DirectorySegmentConfig::from_options(options);
        self
    }

    /// Extract and format directory name from path according to configuration
    fn format_directory_path(&self, path: &str) -> String {
        let formatted_path = if self.config.abbreviate_home {
            self.abbreviate_home_directory(path)
        } else {
            path.to_string()
        };
        
        if self.config.show_full_path {
            self.apply_length_limit(&formatted_path)
        } else if self.config.show_parent {
            self.extract_parent_and_current(&formatted_path)
        } else {
            let dir_name = Self::extract_directory_name(&formatted_path);
            self.apply_case_style(&self.apply_length_limit(&dir_name))
        }
    }
    
    /// Abbreviate home directory with ~
    fn abbreviate_home_directory(&self, path: &str) -> String {
        if let Ok(home) = std::env::var("HOME") {
            if path.starts_with(&home) {
                return path.replace(&home, "~");
            }
        }
        
        // Windows home directory
        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            if path.starts_with(&userprofile) {
                return path.replace(&userprofile, "~");
            }
        }
        
        path.to_string()
    }
    
    /// Extract parent and current directory
    fn extract_parent_and_current(&self, path: &str) -> String {
        let path_obj = Path::new(path);
        
        if let (Some(parent), Some(current)) = (path_obj.parent(), path_obj.file_name()) {
            let parent_name = parent.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            let current_name = current.to_str().unwrap_or("");
            
            if !parent_name.is_empty() {
                let combined = format!("{}/{}", parent_name, current_name);
                self.apply_case_style(&self.apply_length_limit(&combined))
            } else {
                let current_formatted = Self::extract_directory_name(current_name);
                self.apply_case_style(&self.apply_length_limit(&current_formatted))
            }
        } else {
            let dir_name = Self::extract_directory_name(path);
            self.apply_case_style(&self.apply_length_limit(&dir_name))
        }
    }
    
    /// Apply length limit with ellipsis
    fn apply_length_limit(&self, text: &str) -> String {
        if text.len() <= self.config.max_length {
            text.to_string()
        } else {
            let ellipsis = "...";
            if self.config.max_length <= ellipsis.len() {
                ellipsis.to_string()
            } else {
                let take_len = self.config.max_length - ellipsis.len();
                format!("{}{}", ellipsis, &text[text.len() - take_len..])
            }
        }
    }
    
    /// Apply case style transformation
    fn apply_case_style(&self, text: &str) -> String {
        match self.config.case_style {
            CaseStyle::Original => text.to_string(),
            CaseStyle::Lowercase => text.to_lowercase(),
            CaseStyle::Uppercase => text.to_uppercase(),
        }
    }
    
    /// Extract directory name from path, handling both Unix and Windows separators
    fn extract_directory_name(path: &str) -> String {
        // Handle both Unix and Windows separators by trying both
        let unix_name = path.split('/').next_back().unwrap_or("");
        let windows_name = path.split('\\').next_back().unwrap_or("");

        // Choose the name that indicates actual path splitting occurred
        let result = if windows_name.len() < path.len() {
            // Windows path separator was found
            windows_name
        } else if unix_name.len() < path.len() {
            // Unix path separator was found
            unix_name
        } else {
            // No separator found, use the whole path
            path
        };

        if result.is_empty() {
            "root".to_string()
        } else {
            result.to_string()
        }
    }
}

impl Segment for DirectorySegment {
    fn collect(&self, input: &InputData) -> Option<SegmentData> {
        let current_dir = &input.workspace.current_dir;

        // Use the new formatting logic
        let formatted_dir = self.format_directory_path(current_dir);

        // Store configuration and path information in metadata
        let mut metadata = HashMap::new();
        metadata.insert("full_path".to_string(), current_dir.clone());
        metadata.insert("max_length".to_string(), self.config.max_length.to_string());
        metadata.insert("show_full_path".to_string(), self.config.show_full_path.to_string());
        metadata.insert("abbreviate_home".to_string(), self.config.abbreviate_home.to_string());
        metadata.insert("show_parent".to_string(), self.config.show_parent.to_string());
        metadata.insert("case_style".to_string(), format!("{:?}", self.config.case_style));

        Some(SegmentData {
            primary: formatted_dir,
            secondary: String::new(),
            metadata,
        })
    }

    fn id(&self) -> SegmentId {
        SegmentId::Directory
    }
}
