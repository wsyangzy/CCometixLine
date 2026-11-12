use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    #[serde(default, rename = "models")]
    pub model_entries: Vec<ModelEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    pub pattern: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub match_mode: ModelMatchMode,
    pub display_name: String,
    pub context_limit: u32,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelMatchMode {
    #[default]
    Substring,
    ModelId,
}

impl ModelEntry {
    fn matches(&self, model_id: &str) -> bool {
        std::iter::once(&self.pattern)
            .chain(&self.aliases)
            .any(|pattern| {
                let pattern = pattern.to_lowercase();
                if pattern.is_empty() || pattern == "[1m]" {
                    return false;
                }
                match self.match_mode {
                    ModelMatchMode::Substring => model_id.contains(&pattern),
                    ModelMatchMode::ModelId => {
                        let Some(suffix) = model_id.strip_prefix(&pattern) else {
                            return false;
                        };
                        suffix.is_empty()
                            || suffix.strip_prefix('-').is_some_and(|suffix| {
                                suffix.starts_with(|ch: char| ch.is_ascii_alphabetic())
                                    || (suffix.len() == 8
                                        && suffix.bytes().all(|ch| ch.is_ascii_digit()))
                            })
                    }
                }
            })
    }
}

impl ModelConfig {
    fn built_in() -> &'static Self {
        static BUILT_IN: OnceLock<ModelConfig> = OnceLock::new();
        BUILT_IN.get_or_init(|| {
            toml::from_str(include_str!("../../models.toml")).unwrap_or_else(|error| {
                eprintln!("Invalid embedded models.toml: {error}");
                Self {
                    model_entries: Vec::new(),
                }
            })
        })
    }

    /// Load model configuration from TOML file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: ModelConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Load runtime user overrides ahead of the embedded model catalog.
    pub fn load() -> Self {
        let Some(home_dir) = dirs::home_dir() else {
            return Self::default();
        };
        let path = home_dir.join(".claude").join("ccline").join("models.toml");
        if !path.exists() {
            let _ = Self::create_default_file(&path);
        }
        Self::load_with_overrides(&path)
    }

    fn load_with_overrides(path: &Path) -> Self {
        let built_in = Self::default();
        match Self::load_from_file(path) {
            Ok(mut config) => {
                config.model_entries.extend(built_in.model_entries);
                config
            }
            Err(_) => built_in,
        }
    }

    fn normalize_id(model_id: &str) -> (String, bool) {
        let model_id = model_id.to_lowercase();
        let has_1m = model_id.contains("[1m]");
        (model_id.replace("[1m]", ""), has_1m)
    }

    fn find_entry(&self, model_id: &str) -> Option<&ModelEntry> {
        self.model_entries
            .iter()
            .find(|entry| entry.matches(model_id))
    }

    /// Get context limit for a model based on ID pattern matching
    /// Checks external config first, then falls back to built-in config
    ///
    /// Special handling for [1m] suffix: returns 1M context limit for models with this suffix
    pub fn get_context_limit(&self, model_id: &str) -> u32 {
        let (model_id, has_1m) = Self::normalize_id(model_id);
        if has_1m {
            return 1_000_000;
        }
        self.find_entry(&model_id)
            .map_or(200_000, |entry| entry.context_limit)
    }

    /// User overrides and embedded entries take priority over automatic Claude names.
    /// Return None for unrecognized IDs so callers can use the supplied display name.
    pub fn get_display_name(&self, model_id: &str) -> Option<String> {
        let (model_id, has_1m) = Self::normalize_id(model_id);
        let name = self
            .find_entry(&model_id)
            .map(|entry| entry.display_name.clone())
            .or_else(|| {
                Self::parse_claude_model(&model_id)
                    .map(|(family, version)| format!("{family} {version}"))
            })?;
        Some(if has_1m { format!("{name} 1M") } else { name })
    }

    /// Resolve display names and optional abbreviations outside the segment renderer.
    pub(crate) fn format_name(&self, model_id: &str, fallback: &str, abbreviate: bool) -> String {
        let name = self
            .get_display_name(model_id)
            .unwrap_or_else(|| fallback.to_string());
        if abbreviate {
            name.replace("claude-", "")
                .replace("3-5-", "3.5-")
                .replace("sonnet", "Sonnet")
                .replace("haiku", "Haiku")
                .replace("opus", "Opus")
                .replace("gpt-", "GPT-")
                .replace("turbo", "Turbo")
        } else {
            name
        }
    }

    pub(crate) fn get_version(model_id: &str) -> Option<String> {
        if let Some((_, version)) = Self::parse_claude_model(model_id) {
            return Some(version);
        }
        let start = model_id.find(|ch: char| ch.is_ascii_digit())?;
        Some(
            model_id[start..]
                .chars()
                .take_while(|ch| ch.is_ascii_digit() || *ch == '.')
                .collect(),
        )
    }

    /// Recognize both Claude ID layouts without enumerating their versions.
    pub(crate) fn parse_claude_model(model_id: &str) -> Option<(String, String)> {
        let normalized = model_id.to_ascii_lowercase().replace("[1m]", "");
        let (model, has_prefix) = match normalized.strip_prefix("claude-") {
            Some(model) => (model, true),
            None => (normalized.as_str(), false),
        };
        let parts: Vec<&str> = model.split('-').collect();
        // Eight-digit snapshot dates are suffixes, not version components.
        let is_version = |part: &str| {
            part.split('.').all(|number| {
                !number.is_empty()
                    && number.len() < 8
                    && number.bytes().all(|ch| ch.is_ascii_digit())
            })
        };
        let start = parts.iter().position(|part| is_version(part))?;
        let end = start
            + parts[start..]
                .iter()
                .take_while(|part| is_version(part))
                .count();
        let family = if start == 0 && has_prefix {
            // Legacy IDs use claude-3-5-sonnet[-date].
            parts.get(end)?.to_string()
        } else {
            // Current IDs use claude-sonnet-4-5[-date].
            parts[..start].join("-")
        };
        if family.is_empty()
            || !family
                .split('-')
                .all(|word| !word.is_empty() && word.bytes().all(|ch| ch.is_ascii_lowercase()))
        {
            return None;
        }
        // Restrict unprefixed IDs so third-party models keep their own names.
        // Any family with an explicit claude- prefix can be recognized.
        if !has_prefix
            && !Self::built_in()
                .model_entries
                .iter()
                .any(|entry| entry.pattern.starts_with(&format!("claude-{family}-")))
        {
            return None;
        }
        let family = family
            .split('-')
            .map(|word| format!("{}{}", word[..1].to_ascii_uppercase(), &word[1..]))
            .collect::<Vec<_>>()
            .join(" ");
        Some((family, parts[start..end].join(".")))
    }

    /// Create default model configuration file with minimal template
    pub fn create_default_file<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn std::error::Error>> {
        // Add comments and examples to the template
        // Keep the empty array commented so adding [[models]] is valid TOML.
        let template_content = "# CCometixLine Model Configuration\n\
             # This file defines model display names and context limits for different LLM models\n\
             # File location: ~/.claude/ccline/models.toml\n\
             \n\
             # models = []\n\
             \n\
             # Model configurations\n\
             # Each [[models]] section defines a model pattern and its properties\n\
             # Order matters: first match wins, so put more specific patterns first\n\
             # Claude family/version IDs are recognized automatically\n\
             # Use overrides for custom names, unusual IDs, or context limits\n\
             # Changes apply on the next statusline invocation; no rebuild needed\n\
             \n\
             # Example of how to add new models:\n\
             # [[models]]\n\
             # pattern = \"glm-4.5\"\n\
             # display_name = \"GLM-4.5\"\n\
             # context_limit = 128000\n";

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
        Self::built_in().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::{ModelConfig, ModelMatchMode};

    #[test]
    fn embedded_catalog_resolves_every_entry_and_alias() {
        let catalog: ModelConfig = toml::from_str(include_str!("../../models.toml")).unwrap();
        assert!(!catalog.model_entries.is_empty());
        let config = ModelConfig::default();
        for entry in &catalog.model_entries {
            assert!(!entry.pattern.is_empty());
            assert!(!entry.display_name.is_empty());
            assert!(entry.context_limit > 0);
            for id in std::iter::once(&entry.pattern).chain(&entry.aliases) {
                let matched = config.find_entry(&id.to_lowercase()).unwrap();
                assert_eq!(matched.display_name, entry.display_name, "{id}");
                assert_eq!(config.get_context_limit(id), entry.context_limit, "{id}");
                assert_eq!(
                    config.get_display_name(id).as_deref(),
                    Some(entry.display_name.as_str()),
                    "{id}"
                );
                assert_eq!(
                    config.get_display_name(&format!("{}[1M]", id.to_uppercase())),
                    Some(format!("{} 1M", entry.display_name)),
                    "{id}"
                );
                if matches!(entry.match_mode, ModelMatchMode::ModelId) {
                    for suffix in ["-20261112", "-latest", "-preview"] {
                        assert_eq!(
                            config
                                .find_entry(&format!("{id}{suffix}"))
                                .unwrap()
                                .display_name,
                            entry.display_name,
                            "{id}{suffix}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn built_in_patterns_do_not_swallow_new_numeric_versions() {
        let config = ModelConfig::default();
        for id in [
            "claude-fable-5-20",
            "claude-fable-5-3",
            "fable-5-20",
            "claude-opus-4-60",
            "sonnet-4-50",
        ] {
            assert!(config.find_entry(id).is_none(), "{id}");
        }
        assert_eq!(
            config.get_display_name("claude-fable-5-20").as_deref(),
            Some("Fable 5.20")
        );
    }

    #[test]
    fn maps_fable_5_2_and_future_claude_versions() {
        let config = ModelConfig::default();
        for (model_id, expected) in [
            ("claude-fable-5-2", "Fable 5.2"),
            ("fable-5-2", "Fable 5.2"),
            ("CLAUDE-FABLE-5-2-20261112[1M]", "Fable 5.2 1M"),
            ("claude-fable-5-20", "Fable 5.20"),
            ("claude-fable-6-1-2-latest", "Fable 6.1.2"),
            ("claude-opus-6-2", "Opus 6.2"),
            ("claude-sonnet-4-50", "Sonnet 4.50"),
            ("claude-haiku-7.12", "Haiku 7.12"),
            ("claude-new-family-8-3-preview", "New Family 8.3"),
        ] {
            assert_eq!(
                config.get_display_name(model_id).as_deref(),
                Some(expected),
                "{model_id}"
            );
        }
    }

    #[test]
    fn preserves_legacy_ids_and_excludes_snapshot_dates() {
        let config = ModelConfig::default();
        for (model_id, expected) in [
            ("claude-3-5-sonnet-20241022", "Sonnet 3.5"),
            ("claude-3-opus-20240229", "Opus 3"),
            ("claude-4-sonnet", "Sonnet 4"),
            ("claude-haiku-4-5-20251001", "Haiku 4.5"),
            ("claude-fable-5-20261112", "Fable 5"),
            ("claude-mythos-5", "Mythos 5"),
        ] {
            assert_eq!(
                config.get_display_name(model_id).as_deref(),
                Some(expected),
                "{model_id}"
            );
        }
    }

    #[test]
    fn configured_names_and_limits_override_automatic_parsing() {
        let mut config: ModelConfig = toml::from_str(
            r#"
[[models]]
pattern = "CLAUDE-FABLE-5-2"
display_name = "My Fable"
context_limit = 256000

[[models]]
pattern = "claude-fable"
display_name = "Custom Claude"
context_limit = 128000
"#,
        )
        .unwrap();
        config
            .model_entries
            .extend(ModelConfig::default().model_entries);
        assert_eq!(
            config.get_display_name("claude-fable-5-2").as_deref(),
            Some("My Fable")
        );
        assert_eq!(config.get_context_limit("claude-fable-5-2"), 256_000);
        assert_eq!(
            config.get_display_name("claude-fable-6-1").as_deref(),
            Some("Custom Claude")
        );
        assert_eq!(config.get_context_limit("claude-fable-6-1"), 128_000);
        assert_eq!(
            config.get_display_name("claude-fable-5-2[1m]").as_deref(),
            Some("My Fable 1M")
        );
        assert_eq!(config.get_context_limit("claude-fable-5-2[1m]"), 1_000_000);
    }

    #[test]
    fn unknown_models_use_the_callers_display_name() {
        let config = ModelConfig::default();
        for model_id in [
            "unknown-model",
            "unknown-model[1m]",
            "gpt-6-2",
            "claude-fable",
            "claude--5-2",
            "claude-5-2",
        ] {
            assert_eq!(config.get_display_name(model_id), None, "{model_id}");
        }
        assert_eq!(config.get_context_limit("unknown-model"), 200_000);
        assert_eq!(config.get_context_limit("unknown-model[1m]"), 1_000_000);
    }

    #[test]
    fn preserves_third_party_models() {
        let config = ModelConfig::default();
        for (model_id, expected, context_limit) in [
            ("glm-4.5", "GLM-4.5", 128_000),
            ("kimi-k2-turbo", "Kimi K2 Turbo", 128_000),
            ("kimi-k2", "Kimi K2", 128_000),
            ("qwen3-coder", "Qwen Coder", 256_000),
            ("GLM-4.5[1M]", "GLM-4.5 1M", 1_000_000),
        ] {
            assert_eq!(
                config.get_display_name(model_id).as_deref(),
                Some(expected),
                "{model_id}"
            );
            assert_eq!(
                config.get_context_limit(model_id),
                context_limit,
                "{model_id}"
            );
        }
    }

    #[test]
    fn empty_model_configuration_allows_automatic_names() {
        for content in ["", "# models = []", "models = []"] {
            let config: ModelConfig = toml::from_str(content).unwrap();
            assert_eq!(
                config.get_display_name("claude-fable-5-2").as_deref(),
                Some("Fable 5.2")
            );
        }
    }

    #[test]
    fn generated_template_accepts_models_and_reloads_file_edits() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory =
            std::env::temp_dir().join(format!("ccline-models-{}-{unique}", std::process::id()));
        let path = directory.join("models.toml");
        ModelConfig::create_default_file(&path).unwrap();
        let template = std::fs::read_to_string(&path).unwrap();
        let empty = ModelConfig::load_with_overrides(&path);
        assert!(empty.find_entry("claude-fable-5-2").is_some());
        assert_eq!(
            empty.get_display_name("claude-fable-5-2").as_deref(),
            Some("Fable 5.2")
        );

        let configured = template
            .replace("# [[models]]", "[[models]]")
            .replace("# pattern =", "pattern =")
            .replace("# display_name =", "display_name =")
            .replace("# context_limit =", "context_limit =");
        std::fs::write(&path, &configured).unwrap();
        let first = ModelConfig::load_with_overrides(&path);
        assert_eq!(
            first.get_display_name("glm-4.5").as_deref(),
            Some("GLM-4.5")
        );

        std::fs::write(
            &path,
            configured
                .replace("GLM-4.5", "My Model")
                .replace("128000", "256000"),
        )
        .unwrap();
        let updated = ModelConfig::load_with_overrides(&path);
        std::fs::write(&path, "invalid = [").unwrap();
        let invalid = ModelConfig::load_with_overrides(&path);
        assert!(invalid.find_entry("claude-fable-5-2").is_some());
        std::fs::remove_file(&path).unwrap();
        let missing = ModelConfig::load_with_overrides(&path);
        std::fs::remove_dir(&directory).unwrap();
        assert!(missing.find_entry("claude-fable-5-2").is_some());
        assert_eq!(
            updated.get_display_name("glm-4.5").as_deref(),
            Some("My Model")
        );
        assert_eq!(updated.get_context_limit("glm-4.5"), 256_000);
    }

    #[test]
    fn maps_opus_5_5_model_variants() {
        let config = ModelConfig::default();

        for model_id in [
            "claude-opus-5-5",
            "opus-5-5",
            "claude-opus-5-5-20260101",
            "opus-5-5-20260101",
            "CLAUDE-OPUS-5-5",
        ] {
            assert_eq!(
                config.get_display_name(model_id).as_deref(),
                Some("Opus 5.5"),
                "{model_id}"
            );
            assert_eq!(config.get_context_limit(model_id), 200_000, "{model_id}");
        }
    }

    #[test]
    fn maps_opus_5_5_1m_model_variants() {
        let config = ModelConfig::default();

        for model_id in [
            "claude-opus-5-5[1m]",
            "opus-5-5[1m]",
            "claude-opus-5-5-20260101[1m]",
            "opus-5-5-20260101[1m]",
            "CLAUDE-OPUS-5-5[1M]",
        ] {
            assert_eq!(
                config.get_display_name(model_id).as_deref(),
                Some("Opus 5.5 1M"),
                "{model_id}"
            );
            assert_eq!(config.get_context_limit(model_id), 1_000_000, "{model_id}");
        }
    }

    #[test]
    fn preserves_opus_5_model_variants() {
        let config = ModelConfig::default();

        for (model_id, display_name, context_limit) in [
            ("claude-opus-5", "Opus 5", 200_000),
            ("claude-opus-5[1m]", "Opus 5 1M", 1_000_000),
        ] {
            assert_eq!(
                config.get_display_name(model_id).as_deref(),
                Some(display_name),
                "{model_id}"
            );
            assert_eq!(
                config.get_context_limit(model_id),
                context_limit,
                "{model_id}"
            );
        }
    }

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

    #[test]
    fn maps_sonnet_4_6_model_name() {
        let config = ModelConfig::default();
        let display_name = config.get_display_name("claude-sonnet-4-6-20260101");
        assert_eq!(display_name, Some("Sonnet 4.6".to_string()));
    }

    #[test]
    fn maps_sonnet_4_6_1m_model_name() {
        let config = ModelConfig::default();
        let display_name = config.get_display_name("claude-sonnet-4-6-20260101[1m]");
        assert_eq!(display_name, Some("Sonnet 4.6 1M".to_string()));
    }

    #[test]
    fn maps_fable_5_1_1m_model_name() {
        let config = ModelConfig::default();
        let display_name = config.get_display_name("claude-fable-5-1-20260101[1m]");
        assert_eq!(display_name, Some("Fable 5.1 1M".to_string()));
    }

    #[test]
    fn maps_fable_5_1_model_name() {
        let config = ModelConfig::default();
        let display_name = config.get_display_name("claude-fable-5-1-20260101");
        assert_eq!(display_name, Some("Fable 5.1".to_string()));
    }
}
