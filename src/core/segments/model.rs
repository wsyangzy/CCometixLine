use super::{Segment, SegmentData};
use crate::config::{InputData, ModelConfig, ModelDisplayFormat, ModelSegmentConfig, SegmentId};
use std::collections::HashMap;

pub struct ModelSegment {
    config: ModelSegmentConfig,
    models: ModelConfig,
}

impl ModelSegment {
    pub fn new() -> Self {
        Self {
            config: ModelSegmentConfig::default(),
            models: ModelConfig::load(),
        }
    }

    pub fn with_config(options: &HashMap<String, serde_json::Value>) -> Self {
        Self {
            config: ModelSegmentConfig::from_options(options),
            models: ModelConfig::load(),
        }
    }
}

impl Segment for ModelSegment {
    fn collect(&self, input: &InputData) -> Option<SegmentData> {
        let formatted_name = self.models.format_name(
            &input.model.id,
            &input.model.display_name,
            self.config.abbreviate_names,
        );

        let (primary, secondary) = match self.config.display_format {
            ModelDisplayFormat::Name => (formatted_name, String::new()),
            ModelDisplayFormat::Full => {
                let version = if self.config.show_version {
                    ModelConfig::get_version(&input.model.id)
                } else {
                    None
                };

                if let Some(ver) = version {
                    (formatted_name, format!("v{}", ver))
                } else {
                    (formatted_name, String::new())
                }
            }
            ModelDisplayFormat::Custom => {
                if let Some(custom_name) = self.config.custom_names.get(&input.model.id) {
                    (custom_name.clone(), String::new())
                } else {
                    (formatted_name, String::new())
                }
            }
        };

        let mut metadata = HashMap::new();
        metadata.insert("model_id".to_string(), input.model.id.clone());
        metadata.insert("display_name".to_string(), input.model.display_name.clone());
        metadata.insert("formatted_name".to_string(), primary.clone());
        metadata.insert(
            "display_format".to_string(),
            format!("{:?}", self.config.display_format),
        );

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

impl Default for ModelSegment {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_display_uses_the_complete_claude_version() {
        let segment = ModelSegment {
            config: ModelSegmentConfig {
                display_format: ModelDisplayFormat::Full,
                show_version: true,
                ..ModelSegmentConfig::default()
            },
            models: ModelConfig::default(),
        };
        for (model_id, expected) in [
            ("claude-fable-5-2", "5.2"),
            ("claude-fable-5-2-20261112[1m]", "5.2"),
            ("CLAUDE-OPUS-6-12", "6.12"),
            ("claude-3-5-sonnet-20241022", "3.5"),
            ("claude-new-family-8-3", "8.3"),
        ] {
            let input = serde_json::from_value(serde_json::json!({
                "model": {"id": model_id, "display_name": "Official Name"},
                "workspace": {"current_dir": "."},
                "transcript_path": ""
            }))
            .unwrap();
            let output = segment.collect(&input).unwrap();
            assert_eq!(output.secondary, format!("v{expected}"), "{model_id}");
        }
    }

    #[test]
    fn display_options_and_unknown_names_are_preserved() {
        let input = serde_json::from_value(serde_json::json!({
            "model": {"id": "unknown-model", "display_name": "gpt-6-turbo"},
            "workspace": {"current_dir": "."},
            "transcript_path": ""
        }))
        .unwrap();
        let mut segment = ModelSegment {
            config: ModelSegmentConfig::default(),
            models: ModelConfig::default(),
        };
        assert_eq!(segment.collect(&input).unwrap().primary, "GPT-6-Turbo");
        segment.config.abbreviate_names = false;
        assert_eq!(segment.collect(&input).unwrap().primary, "gpt-6-turbo");
        segment.config.display_format = ModelDisplayFormat::Custom;
        segment
            .config
            .custom_names
            .insert("unknown-model".to_string(), "My Model".to_string());
        assert_eq!(segment.collect(&input).unwrap().primary, "My Model");
    }
}
