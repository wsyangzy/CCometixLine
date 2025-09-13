use super::{Segment, SegmentData};
use crate::config::{InputData, ModelConfig, SegmentId, ModelSegmentConfig, ModelDisplayFormat};
use std::collections::HashMap;

pub struct ModelSegment {
    config: ModelSegmentConfig,
}

impl ModelSegment {
    pub fn new() -> Self {
        Self {
            config: ModelSegmentConfig::default(),
        }
    }
    
    pub fn with_config(options: &HashMap<String, serde_json::Value>) -> Self {
        Self {
            config: ModelSegmentConfig::from_options(options),
        }
    }
}

impl Segment for ModelSegment {
    fn collect(&self, input: &InputData) -> Option<SegmentData> {
        let formatted_name = self.format_model_name(&input.model.id, &input.model.display_name);
        
        let (primary, secondary) = match self.config.display_format {
            ModelDisplayFormat::Name => (formatted_name, String::new()),
            ModelDisplayFormat::Full => {
                let version = if self.config.show_version {
                    self.extract_version(&input.model.id)
                } else {
                    None
                };
                
                if let Some(ver) = version {
                    (formatted_name, format!("v{}", ver))
                } else {
                    (formatted_name, String::new())
                }
            },
            ModelDisplayFormat::Custom => {
                if let Some(custom_name) = self.config.custom_names.get(&input.model.id) {
                    (custom_name.clone(), String::new())
                } else {
                    (formatted_name, String::new())
                }
            },
        };

        let mut metadata = HashMap::new();
        metadata.insert("model_id".to_string(), input.model.id.clone());
        metadata.insert("display_name".to_string(), input.model.display_name.clone());
        metadata.insert("formatted_name".to_string(), primary.clone());
        metadata.insert("display_format".to_string(), format!("{:?}", self.config.display_format));

        Some(SegmentData {
            primary,
            secondary,
            metadata,
        })
    }

    fn id(&self) -> SegmentId {
        SegmentId::Model
    }
}

impl ModelSegment {
    fn format_model_name(&self, id: &str, display_name: &str) -> String {
        let model_config = ModelConfig::load();

        // Try to get display name from external config first
        let base_name = if let Some(config_name) = model_config.get_display_name(id) {
            config_name
        } else {
            // Fallback to Claude Code's official display_name for unrecognized models
            display_name.to_string()
        };
        
        if self.config.abbreviate_names {
            self.abbreviate_model_name(&base_name)
        } else {
            base_name
        }
    }
    
    fn abbreviate_model_name(&self, name: &str) -> String {
        // Apply common abbreviations
        name.replace("claude-", "")
            .replace("3-5-", "3.5-")
            .replace("4-", "4-")
            .replace("sonnet", "Sonnet")
            .replace("haiku", "Haiku")
            .replace("opus", "Opus")
            .replace("gpt-", "GPT-")
            .replace("turbo", "Turbo")
    }
    
    fn extract_version(&self, model_id: &str) -> Option<String> {
        // Extract version from model ID using simple string parsing
        // Look for patterns like "3.5", "4", "3-5", etc.
        
        if model_id.contains("3-5") || model_id.contains("3.5") {
            Some("3.5".to_string())
        } else if model_id.contains("4-") || model_id.contains("4.") {
            Some("4".to_string())
        } else if model_id.contains("3-") {
            Some("3".to_string())
        } else {
            // Try to find any number pattern
            let chars: Vec<char> = model_id.chars().collect();
            let mut version = String::new();
            let mut in_number = false;
            
            for &ch in &chars {
                if ch.is_ascii_digit() || ch == '.' {
                    version.push(ch);
                    in_number = true;
                } else if in_number && !version.is_empty() {
                    break;
                }
            }
            
            if !version.is_empty() {
                Some(version)
            } else {
                None
            }
        }
    }
}

impl Default for ModelSegment {
    fn default() -> Self {
        Self::new()
    }
}
