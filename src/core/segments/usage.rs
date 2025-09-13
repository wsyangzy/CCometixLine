use super::{Segment, SegmentData};
use crate::config::{InputData, ModelConfig, SegmentId, TranscriptEntry, UsageSegmentConfig, UsageDisplayFormat, TokenUnit};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct UsageSegment {
    config: UsageSegmentConfig,
}

impl UsageSegment {
    pub fn new() -> Self {
        Self {
            config: UsageSegmentConfig::default(),
        }
    }
    
    pub fn with_config(mut self, options: &HashMap<String, serde_json::Value>) -> Self {
        self.config = UsageSegmentConfig::from_options(options);
        self
    }

    /// Get context limit for the specified model
    fn get_context_limit_for_model(model_id: &str) -> u32 {
        let model_config = ModelConfig::load();
        model_config.get_context_limit(model_id)
    }
    
    /// Format tokens according to the configuration
    fn format_tokens(&self, tokens: u32) -> String {
        match self.config.token_unit {
            TokenUnit::Raw => tokens.to_string(),
            TokenUnit::K => {
                let k_value = tokens as f64 / 1000.0;
                if k_value.fract() == 0.0 {
                    format!("{}k", k_value as u32)
                } else {
                    format!("{:.1}k", k_value)
                }
            }
            TokenUnit::Auto => {
                if tokens >= 1000 {
                    let k_value = tokens as f64 / 1000.0;
                    if k_value.fract() == 0.0 {
                        format!("{}k", k_value as u32)
                    } else {
                        format!("{:.1}k", k_value)
                    }
                } else {
                    tokens.to_string()
                }
            }
        }
    }
    
    /// Format percentage with appropriate precision
    fn format_percentage(&self, percentage: f64) -> String {
        if percentage.fract() == 0.0 {
            format!("{:.0}%", percentage)
        } else {
            format!("{:.1}%", percentage)
        }
    }
    
    /// Generate progress bar for display
    fn generate_progress_bar(&self, percentage: f64, tokens: u32, limit: u32) -> String {
        let bar_width = 10;
        let filled = (percentage / 10.0).round() as usize;
        let filled = filled.min(bar_width);
        let empty = bar_width - filled;
        
        let bar = format!("{}{}", 
            "█".repeat(filled),
            "░".repeat(empty)
        );
        
        let mut parts = vec![bar];
        
        if self.config.bar_show_percentage {
            parts.push(format!("{:.0}%", percentage));
        }
        
        if self.config.bar_show_tokens {
            let tokens_str = self.format_tokens(tokens);
            if self.config.show_limit {
                let limit_str = self.format_tokens(limit);
                parts.push(format!("{}/{}", tokens_str, limit_str));
            } else {
                parts.push(tokens_str);
            }
        }
        
        parts.join(" ")
    }
    
    /// Determine usage status based on thresholds
    fn get_usage_status(&self, percentage: f64) -> UsageStatus {
        if percentage >= self.config.critical_threshold as f64 {
            UsageStatus::Critical
        } else if percentage >= self.config.warning_threshold as f64 {
            UsageStatus::Warning
        } else {
            UsageStatus::Normal
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum UsageStatus {
    Normal,
    Warning,
    Critical,
}

impl Segment for UsageSegment {
    fn collect(&self, input: &InputData) -> Option<SegmentData> {
        // Dynamically determine context limit based on current model ID
        let context_limit = Self::get_context_limit_for_model(&input.model.id);

        let context_used_token_opt = parse_transcript_usage(&input.transcript_path);

        let (primary_display, secondary_display) = match context_used_token_opt {
            Some(context_used_token) => {
                let context_used_rate = (context_used_token as f64 / context_limit as f64) * 100.0;

                let percentage_str = self.format_percentage(context_used_rate);
                let tokens_str = self.format_tokens(context_used_token);

                let primary = match self.config.display_format {
                    UsageDisplayFormat::Percentage => percentage_str.clone(),
                    UsageDisplayFormat::Tokens => {
                        if self.config.show_limit {
                            let limit_str = self.format_tokens(context_limit);
                            format!("{}/{}", tokens_str, limit_str)
                        } else {
                            tokens_str.clone()
                        }
                    },
                    UsageDisplayFormat::Both => {
                        let separator = if self.config.compact_format { "·" } else { " · " };
                        let tokens_part = if self.config.show_limit {
                            let limit_str = self.format_tokens(context_limit);
                            format!("{}/{}", tokens_str, limit_str)
                        } else {
                            format!("{} tokens", tokens_str)
                        };
                        format!("{}{}{}", percentage_str, separator, tokens_part)
                    },
                    UsageDisplayFormat::Bar => {
                        self.generate_progress_bar(context_used_rate, context_used_token, context_limit)
                    },
                };

                let secondary = if self.config.show_limit && 
                    self.config.display_format != UsageDisplayFormat::Tokens &&
                    !(self.config.display_format == UsageDisplayFormat::Bar && self.config.bar_show_tokens && self.config.show_limit) {
                    let limit_str = self.format_tokens(context_limit);
                    format!("/{}", limit_str)
                } else {
                    String::new()
                };

                (primary, secondary)
            }
            None => {
                // No usage data available
                ("-".to_string(), String::new())
            }
        };

        let mut metadata = HashMap::new();
        match context_used_token_opt {
            Some(context_used_token) => {
                let context_used_rate = (context_used_token as f64 / context_limit as f64) * 100.0;
                let usage_status = self.get_usage_status(context_used_rate);
                
                metadata.insert("tokens".to_string(), context_used_token.to_string());
                metadata.insert("percentage".to_string(), context_used_rate.to_string());
                metadata.insert("status".to_string(), format!("{:?}", usage_status));
                metadata.insert("warning_threshold".to_string(), self.config.warning_threshold.to_string());
                metadata.insert("critical_threshold".to_string(), self.config.critical_threshold.to_string());
            }
            None => {
                metadata.insert("tokens".to_string(), "-".to_string());
                metadata.insert("percentage".to_string(), "-".to_string());
                metadata.insert("status".to_string(), "Unknown".to_string());
            }
        }
        metadata.insert("limit".to_string(), context_limit.to_string());
        metadata.insert("model".to_string(), input.model.id.clone());
        metadata.insert("display_format".to_string(), format!("{:?}", self.config.display_format));

        Some(SegmentData {
            primary: primary_display,
            secondary: secondary_display,
            metadata,
        })
    }

    fn id(&self) -> SegmentId {
        SegmentId::Usage
    }
}

fn parse_transcript_usage<P: AsRef<Path>>(transcript_path: P) -> Option<u32> {
    let path = transcript_path.as_ref();

    // Try to parse from current transcript file
    if let Some(usage) = try_parse_transcript_file(path) {
        return Some(usage);
    }

    // If file doesn't exist, try to find usage from project history
    if !path.exists() {
        if let Some(usage) = try_find_usage_from_project_history(path) {
            return Some(usage);
        }
    }

    None
}

fn try_parse_transcript_file(path: &Path) -> Option<u32> {
    let file = fs::File::open(path).ok()?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader
        .lines()
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_default();

    if lines.is_empty() {
        return None;
    }

    // Check if the last line is a summary
    let last_line = lines.last()?.trim();
    if let Ok(entry) = serde_json::from_str::<TranscriptEntry>(last_line) {
        if entry.r#type.as_deref() == Some("summary") {
            // Handle summary case: find usage by leafUuid
            if let Some(leaf_uuid) = &entry.leaf_uuid {
                let project_dir = path.parent()?;
                return find_usage_by_leaf_uuid(leaf_uuid, project_dir);
            }
        }
    }

    // Normal case: find the last assistant message in current file
    for line in lines.iter().rev() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Ok(entry) = serde_json::from_str::<TranscriptEntry>(line) {
            if entry.r#type.as_deref() == Some("assistant") {
                if let Some(message) = &entry.message {
                    if let Some(raw_usage) = &message.usage {
                        let normalized = raw_usage.clone().normalize();
                        return Some(normalized.display_tokens());
                    }
                }
            }
        }
    }

    None
}

fn find_usage_by_leaf_uuid(leaf_uuid: &str, project_dir: &Path) -> Option<u32> {
    // Search for the leafUuid across all session files in the project directory
    let entries = fs::read_dir(project_dir).ok()?;

    for entry in entries {
        let entry = entry.ok()?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("jsonl") {
            continue;
        }

        if let Some(usage) = search_uuid_in_file(&path, leaf_uuid) {
            return Some(usage);
        }
    }

    None
}

fn search_uuid_in_file(path: &Path, target_uuid: &str) -> Option<u32> {
    let file = fs::File::open(path).ok()?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader
        .lines()
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_default();

    // Find the message with target_uuid
    for line in &lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Ok(entry) = serde_json::from_str::<TranscriptEntry>(line) {
            if let Some(uuid) = &entry.uuid {
                if uuid == target_uuid {
                    // Found the target message, check its type
                    if entry.r#type.as_deref() == Some("assistant") {
                        // Direct assistant message with usage
                        if let Some(message) = &entry.message {
                            if let Some(raw_usage) = &message.usage {
                                let normalized = raw_usage.clone().normalize();
                                return Some(normalized.display_tokens());
                            }
                        }
                    } else if entry.r#type.as_deref() == Some("user") {
                        // User message, need to find the parent assistant message
                        if let Some(parent_uuid) = &entry.parent_uuid {
                            return find_assistant_message_by_uuid(&lines, parent_uuid);
                        }
                    }
                    break;
                }
            }
        }
    }

    None
}

fn find_assistant_message_by_uuid(lines: &[String], target_uuid: &str) -> Option<u32> {
    for line in lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Ok(entry) = serde_json::from_str::<TranscriptEntry>(line) {
            if let Some(uuid) = &entry.uuid {
                if uuid == target_uuid && entry.r#type.as_deref() == Some("assistant") {
                    if let Some(message) = &entry.message {
                        if let Some(raw_usage) = &message.usage {
                            let normalized = raw_usage.clone().normalize();
                            return Some(normalized.display_tokens());
                        }
                    }
                }
            }
        }
    }

    None
}

fn try_find_usage_from_project_history(transcript_path: &Path) -> Option<u32> {
    let project_dir = transcript_path.parent()?;

    // Find the most recent session file in the project directory
    let mut session_files: Vec<PathBuf> = Vec::new();
    let entries = fs::read_dir(project_dir).ok()?;

    for entry in entries {
        let entry = entry.ok()?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
            session_files.push(path);
        }
    }

    if session_files.is_empty() {
        return None;
    }

    // Sort by modification time (most recent first)
    session_files.sort_by_key(|path| {
        fs::metadata(path)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::UNIX_EPOCH)
    });
    session_files.reverse();

    // Try to find usage from the most recent session
    for session_path in &session_files {
        if let Some(usage) = try_parse_transcript_file(session_path) {
            return Some(usage);
        }
    }

    None
}
