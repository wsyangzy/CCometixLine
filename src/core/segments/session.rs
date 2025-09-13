use super::{Segment, SegmentData};
use crate::config::{InputData, SegmentId, SessionSegmentConfig, TimeFormat};
use std::collections::HashMap;

pub struct SessionSegment {
    config: SessionSegmentConfig,
}

impl SessionSegment {
    pub fn new() -> Self {
        Self {
            config: SessionSegmentConfig::default(),
        }
    }
    
    pub fn with_config(options: &HashMap<String, serde_json::Value>) -> Self {
        Self {
            config: SessionSegmentConfig::from_options(options),
        }
    }

    fn format_duration(&self, ms: u64) -> String {
        match self.config.time_format {
            TimeFormat::Auto => self.format_auto(ms),
            TimeFormat::Short => self.format_short(ms),
            TimeFormat::Long => self.format_long(ms),
            TimeFormat::Digital => self.format_digital(ms),
        }
    }
    
    fn format_auto(&self, ms: u64) -> String {
        if self.config.show_milliseconds && ms < 1000 {
            format!("{}ms", ms)
        } else if ms < 60_000 {
            let seconds = ms / 1000;
            if self.config.show_milliseconds && ms % 1000 != 0 {
                format!("{}.{:01}s", seconds, (ms % 1000) / 100)
            } else {
                format!("{}s", seconds)
            }
        } else if ms < 3_600_000 {
            let minutes = ms / 60_000;
            let seconds = (ms % 60_000) / 1000;
            if self.config.compact_format {
                if seconds == 0 {
                    format!("{}m", minutes)
                } else {
                    format!("{}m{}s", minutes, seconds)
                }
            } else {
                if seconds == 0 {
                    format!("{} min", minutes)
                } else {
                    format!("{} min {} sec", minutes, seconds)
                }
            }
        } else {
            let hours = ms / 3_600_000;
            let minutes = (ms % 3_600_000) / 60_000;
            if self.config.compact_format {
                if minutes == 0 {
                    format!("{}h", hours)
                } else {
                    format!("{}h{}m", hours, minutes)
                }
            } else {
                if minutes == 0 {
                    format!("{} hr", hours)
                } else {
                    format!("{} hr {} min", hours, minutes)
                }
            }
        }
    }
    
    fn format_short(&self, ms: u64) -> String {
        if ms < 1000 {
            format!("{}ms", ms)
        } else if ms < 60_000 {
            format!("{}s", ms / 1000)
        } else if ms < 3_600_000 {
            let minutes = ms / 60_000;
            let seconds = (ms % 60_000) / 1000;
            format!("{}m{}s", minutes, seconds)
        } else {
            let hours = ms / 3_600_000;
            let minutes = (ms % 3_600_000) / 60_000;
            format!("{}h{}m", hours, minutes)
        }
    }
    
    fn format_long(&self, ms: u64) -> String {
        if ms < 1000 {
            format!("{} milliseconds", ms)
        } else if ms < 60_000 {
            let seconds = ms / 1000;
            if seconds == 1 {
                format!("{} second", seconds)
            } else {
                format!("{} seconds", seconds)
            }
        } else if ms < 3_600_000 {
            let minutes = ms / 60_000;
            let seconds = (ms % 60_000) / 1000;
            let mut result = if minutes == 1 {
                format!("{} minute", minutes)
            } else {
                format!("{} minutes", minutes)
            };
            if seconds > 0 {
                if seconds == 1 {
                    result.push_str(&format!(" {} second", seconds));
                } else {
                    result.push_str(&format!(" {} seconds", seconds));
                }
            }
            result
        } else {
            let hours = ms / 3_600_000;
            let minutes = (ms % 3_600_000) / 60_000;
            let mut result = if hours == 1 {
                format!("{} hour", hours)
            } else {
                format!("{} hours", hours)
            };
            if minutes > 0 {
                if minutes == 1 {
                    result.push_str(&format!(" {} minute", minutes));
                } else {
                    result.push_str(&format!(" {} minutes", minutes));
                }
            }
            result
        }
    }
    
    fn format_digital(&self, ms: u64) -> String {
        let total_seconds = ms / 1000;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;
        
        if hours > 0 {
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            format!("{:02}:{:02}", minutes, seconds)
        }
    }
}

impl Segment for SessionSegment {
    fn collect(&self, input: &InputData) -> Option<SegmentData> {
        let cost_data = input.cost.as_ref()?;

        // Primary display: total duration
        let primary = if let Some(duration) = cost_data.total_duration_ms {
            self.format_duration(duration)
        } else {
            return None;
        };

        // Secondary display: line changes if available and enabled
        let secondary = if self.config.show_line_changes {
            if let Some(lines_added) = cost_data.total_lines_added {
                if let Some(lines_removed) = cost_data.total_lines_removed {
                    if lines_added > 0 || lines_removed > 0 {
                        format!("+{} -{}", lines_added, lines_removed)
                    } else {
                        String::new()
                    }
                } else if lines_added > 0 {
                    format!("+{}", lines_added)
                } else {
                    String::new()
                }
            } else if let Some(lines_removed) = cost_data.total_lines_removed {
                if lines_removed > 0 {
                    format!("-{}", lines_removed)
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let mut metadata = HashMap::new();
        metadata.insert("duration_ms".to_string(), cost_data.total_duration_ms.unwrap_or(0).to_string());
        metadata.insert("time_format".to_string(), format!("{:?}", self.config.time_format));
        if let Some(lines_added) = cost_data.total_lines_added {
            metadata.insert("lines_added".to_string(), lines_added.to_string());
        }
        if let Some(lines_removed) = cost_data.total_lines_removed {
            metadata.insert("lines_removed".to_string(), lines_removed.to_string());
        }

        Some(SegmentData {
            primary,
            secondary,
            metadata,
        })
    }

    fn id(&self) -> SegmentId {
        SegmentId::Session
    }
}

impl Default for SessionSegment {
    fn default() -> Self {
        Self::new()
    }
}
