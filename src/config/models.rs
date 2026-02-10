use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    #[serde(rename = "models")]
    pub model_entries: Vec<ModelEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    pub pattern: String,
    pub display_name: String,
    pub context_limit: u32,
}

impl ModelConfig {
    /// Load model configuration from TOML file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: ModelConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Load model configuration with fallback locations
    pub fn load() -> Self {
        let mut model_config = Self::default();

        // First, try to create default models.toml if it doesn't exist
        if let Some(home_dir) = dirs::home_dir() {
            let user_models_path = home_dir.join(".claude").join("ccline").join("models.toml");
            if !user_models_path.exists() {
                let _ = Self::create_default_file(&user_models_path);
            }
        }

        // Try loading from user config directory first, then local
        let config_paths = [
            dirs::home_dir().map(|d| d.join(".claude").join("ccline").join("models.toml")),
            Some(Path::new("models.toml").to_path_buf()),
        ];

        for path in config_paths.iter().flatten() {
            if path.exists() {
                if let Ok(config) = Self::load_from_file(path) {
                    // Prepend external models to built-in ones for priority
                    let mut merged_entries = config.model_entries;
                    merged_entries.extend(model_config.model_entries);
                    model_config.model_entries = merged_entries;
                    return model_config;
                }
            }
        }

        // Fallback to default configuration if no file found
        model_config
    }

    /// Get context limit for a model based on ID pattern matching
    /// Checks external config first, then falls back to built-in config
    ///
    /// Special handling for [1m] suffix: returns 1M context limit for models with this suffix
    pub fn get_context_limit(&self, model_id: &str) -> u32 {
        let model_lower = model_id.to_lowercase();

        // Check if model has [1m] suffix
        let has_1m = model_lower.contains("[1m]");

        // If has [1m] suffix, remove it for base model matching
        let match_id = if has_1m {
            model_lower.replace("[1m]", "")
        } else {
            model_lower.clone()
        };

        // Match base model (skip the generic [1m] pattern)
        for entry in &self.model_entries {
            // Skip the generic [1m] pattern to avoid early matching
            if entry.pattern == "[1m]" {
                continue;
            }

            if match_id.contains(&entry.pattern.to_lowercase()) {
                return if has_1m {
                    1_000_000 // Override with 1M context for [1m] suffix
                } else {
                    entry.context_limit
                };
            }
        }

        // If no match found but has [1m] suffix, return 1M context
        if has_1m {
            return 1_000_000;
        }

        200_000
    }

    /// Get display name for a model based on ID pattern matching
    /// Checks external config first, then falls back to built-in config
    /// Returns None if no match found (should use fallback display_name)
    ///
    /// Special handling for [1m] suffix: automatically appends " 1M" to the base model name
    pub fn get_display_name(&self, model_id: &str) -> Option<String> {
        let model_lower = model_id.to_lowercase();

        // Check if model has [1m] suffix
        let has_1m = model_lower.contains("[1m]");

        // If has [1m] suffix, remove it for base model matching
        let match_id = if has_1m {
            model_lower.replace("[1m]", "")
        } else {
            model_lower.clone()
        };

        // Match base model (skip the generic [1m] pattern)
        for entry in &self.model_entries {
            // Skip the generic [1m] pattern to avoid early matching
            if entry.pattern == "[1m]" {
                continue;
            }

            if match_id.contains(&entry.pattern.to_lowercase()) {
                let base_name = entry.display_name.clone();
                return Some(if has_1m {
                    format!("{} 1M", base_name) // Append " 1M" suffix
                } else {
                    base_name
                });
            }
        }

        // If no match found but has [1m] suffix, use generic 1M display
        if has_1m {
            return Some("Sonnet 4 1M".to_string());
        }

        None
    }

    /// Create default model configuration file with minimal template
    pub fn create_default_file<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn std::error::Error>> {
        // Create a minimal template config (not the full fallback config)
        let template_config = Self {
            model_entries: vec![], // Empty - just provide the structure
        };

        let toml_content = toml::to_string_pretty(&template_config)?;

        // Add comments and examples to the template
        let template_content = format!(
            "# CCometixLine Model Configuration\n\
             # This file defines model display names and context limits for different LLM models\n\
             # File location: ~/.claude/ccline/models.toml\n\
             \n\
             {}\n\
             \n\
             # Model configurations\n\
             # Each [[models]] section defines a model pattern and its properties\n\
             # Order matters: first match wins, so put more specific patterns first\n\
             \n\
             # Example of how to add new models:\n\
             # [[models]]\n\
             # pattern = \"glm-4.5\"\n\
             # display_name = \"GLM-4.5\"\n\
             # context_limit = 128000\n",
            toml_content.trim()
        );

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, template_content)?;
        Ok(())
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model_entries: vec![
                // Sonnet 4.5 (more specific pattern, must come before sonnet-4)
                ModelEntry {
                    pattern: "claude-sonnet-4-5".to_string(),
                    display_name: "Sonnet 4.5".to_string(),
                    context_limit: 200_000,
                },
                ModelEntry {
                    pattern: "sonnet-4-5".to_string(),
                    display_name: "Sonnet 4.5".to_string(),
                    context_limit: 200_000,
                },
                // Sonnet 4 (more general pattern, must come after 4.5)
                ModelEntry {
                    pattern: "claude-sonnet-4".to_string(),
                    display_name: "Sonnet 4".to_string(),
                    context_limit: 200_000,
                },
                ModelEntry {
                    pattern: "claude-4-sonnet".to_string(),
                    display_name: "Sonnet 4".to_string(),
                    context_limit: 200_000,
                },
                // Opus 4.6 (more specific pattern, must come before opus-4)
                ModelEntry {
                    pattern: "claude-opus-4-6".to_string(),
                    display_name: "Opus 4.6".to_string(),
                    context_limit: 200_000,
                },
                ModelEntry {
                    pattern: "opus-4-6".to_string(),
                    display_name: "Opus 4.6".to_string(),
                    context_limit: 200_000,
                },
                // Opus 4 (generic patterns)
                ModelEntry {
                    pattern: "claude-opus-4".to_string(),
                    display_name: "Opus 4".to_string(),
                    context_limit: 200_000,
                },
                ModelEntry {
                    pattern: "claude-4-opus".to_string(),
                    display_name: "Opus 4".to_string(),
                    context_limit: 200_000,
                },
                ModelEntry {
                    pattern: "sonnet-4".to_string(),
                    display_name: "Sonnet 4".to_string(),
                    context_limit: 200_000,
                },
                ModelEntry {
                    pattern: "claude-3-7-sonnet".to_string(),
                    display_name: "Sonnet 3.7".to_string(),
                    context_limit: 200_000,
                },
                // Third-party models
                ModelEntry {
                    pattern: "glm-4.5".to_string(),
                    display_name: "GLM-4.5".to_string(),
                    context_limit: 128_000,
                },
                ModelEntry {
                    pattern: "kimi-k2-turbo".to_string(),
                    display_name: "Kimi K2 Turbo".to_string(),
                    context_limit: 128_000,
                },
                ModelEntry {
                    pattern: "kimi-k2".to_string(),
                    display_name: "Kimi K2".to_string(),
                    context_limit: 128_000,
                },
                ModelEntry {
                    pattern: "qwen3-coder".to_string(),
                    display_name: "Qwen Coder".to_string(),
                    context_limit: 256_000,
                },
                // Generic [1m] suffix fallback (automatically handled by matching logic)
                // This pattern is skipped during matching but serves as documentation
                // The actual [1m] detection is handled by get_display_name and get_context_limit
                ModelEntry {
                    pattern: "[1m]".to_string(),
                    display_name: "Sonnet 4 1M".to_string(),
                    context_limit: 1_000_000,
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ModelConfig;

    #[test]
    fn maps_opus_4_6_model_name() {
        let config = ModelConfig::default();
        let display_name = config.get_display_name("claude-opus-4-6-20260101");
        assert_eq!(display_name, Some("Opus 4.6".to_string()));
    }

    #[test]
    fn maps_opus_4_6_1m_model_name() {
        let config = ModelConfig::default();
        let display_name = config.get_display_name("claude-opus-4-6-20260101[1m]");
        assert_eq!(display_name, Some("Opus 4.6 1M".to_string()));
    }
}
