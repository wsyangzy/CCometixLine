use super::{Segment, SegmentData};
use crate::config::{InputData, SegmentId, OutputStyleSegmentConfig, OutputStyleDisplayFormat};
use std::collections::HashMap;

pub struct OutputStyleSegment {
    config: OutputStyleSegmentConfig,
}

impl Default for OutputStyleSegment {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputStyleSegment {
    pub fn new() -> Self {
        Self {
            config: OutputStyleSegmentConfig::default(),
        }
    }
    
    pub fn with_config(options: &HashMap<String, serde_json::Value>) -> Self {
        Self {
            config: OutputStyleSegmentConfig::from_options(options),
        }
    }
    
    fn format_style_name(&self, style_name: &str) -> String {
        match self.config.display_format {
            OutputStyleDisplayFormat::Name => style_name.to_string(),
            OutputStyleDisplayFormat::Full => {
                if self.config.show_description {
                    format!("{} (output style)", style_name)
                } else {
                    style_name.to_string()
                }
            },
            OutputStyleDisplayFormat::Abbreviated => {
                if self.config.abbreviate_names {
                    self.abbreviate_name(style_name)
                } else {
                    style_name.to_string()
                }
            },
            OutputStyleDisplayFormat::Custom => {
                self.config.custom_names
                    .get(style_name)
                    .cloned()
                    .unwrap_or_else(|| style_name.to_string())
            },
        }
    }
    
    fn abbreviate_name(&self, name: &str) -> String {
        match name.to_lowercase().as_str() {
            "engineer-professional" => "Eng-Pro".to_string(),
            "creative" => "Creative".to_string(),
            "concise" => "Concise".to_string(),
            "detailed" => "Detail".to_string(),
            "technical" => "Tech".to_string(),
            "casual" => "Casual".to_string(),
            "formal" => "Formal".to_string(),
            _ => {
                // 对于未知样式，取首字母或前几个字符
                if name.len() <= 6 {
                    name.to_string()
                } else {
                    format!("{}...", &name[..6])
                }
            }
        }
    }
    
    fn get_style_description(&self, style_name: &str) -> String {
        match style_name.to_lowercase().as_str() {
            "engineer-professional" => "Professional engineering style".to_string(),
            "creative" => "Creative and expressive style".to_string(),
            "concise" => "Brief and to-the-point style".to_string(),
            "detailed" => "Comprehensive and thorough style".to_string(),
            "technical" => "Technical documentation style".to_string(),
            "casual" => "Informal and conversational style".to_string(),
            "formal" => "Formal business style".to_string(),
            "academic" => "Academic writing style".to_string(),
            "tutorial" => "Step-by-step tutorial style".to_string(),
            _ => format!("Custom style: {}", style_name),
        }
    }
}

impl Segment for OutputStyleSegment {
    fn collect(&self, input: &InputData) -> Option<SegmentData> {
        let output_style = input.output_style.as_ref()?;

        // Primary display: formatted style name
        let primary = self.format_style_name(&output_style.name);

        // Secondary display: description if enabled and format is Full
        let secondary = if self.config.show_description && 
                          self.config.display_format == OutputStyleDisplayFormat::Full {
            self.get_style_description(&output_style.name)
        } else {
            String::new()
        };

        let mut metadata = HashMap::new();
        metadata.insert("style_name".to_string(), output_style.name.clone());
        metadata.insert("display_format".to_string(), format!("{:?}", self.config.display_format));
        metadata.insert("abbreviate_names".to_string(), self.config.abbreviate_names.to_string());
        metadata.insert("show_description".to_string(), self.config.show_description.to_string());
        
        // Add custom name mapping if exists
        if let Some(custom_name) = self.config.custom_names.get(&output_style.name) {
            metadata.insert("custom_name".to_string(), custom_name.clone());
        }

        Some(SegmentData {
            primary,
            secondary,
            metadata,
        })
    }

    fn id(&self) -> SegmentId {
        SegmentId::OutputStyle
    }
}
