use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Usage Segment specific configuration types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UsageDisplayFormat {
    Percentage,
    Tokens,
    Both,
    Bar,
}

impl Default for UsageDisplayFormat {
    fn default() -> Self {
        UsageDisplayFormat::Both
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TokenUnit {
    Auto,
    K,
    Raw,
}

impl Default for TokenUnit {
    fn default() -> Self {
        TokenUnit::Auto
    }
}

// Directory Segment specific configuration types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CaseStyle {
    Original,
    Lowercase,
    Uppercase,
}

impl Default for CaseStyle {
    fn default() -> Self {
        CaseStyle::Original
    }
}

// Directory Segment configuration helper
#[derive(Debug, Clone)]
pub struct DirectorySegmentConfig {
    pub max_length: usize,
    pub show_full_path: bool,
    pub abbreviate_home: bool,
    pub show_parent: bool,
    pub case_style: CaseStyle,
}

impl Default for DirectorySegmentConfig {
    fn default() -> Self {
        Self {
            max_length: 20,
            show_full_path: false,
            abbreviate_home: true,
            show_parent: false,
            case_style: CaseStyle::Original,
        }
    }
}

impl DirectorySegmentConfig {
    pub fn from_options(options: &HashMap<String, serde_json::Value>) -> Self {
        let mut config = Self::default();
        
        config.max_length = options.get("max_length")
            .and_then(|v| v.as_u64())
            .map(|v| v.max(5).min(100) as usize)
            .unwrap_or(config.max_length);
            
        config.show_full_path = options.get("show_full_path")
            .and_then(|v| v.as_bool())
            .unwrap_or(config.show_full_path);
            
        config.abbreviate_home = options.get("abbreviate_home")
            .and_then(|v| v.as_bool())
            .unwrap_or(config.abbreviate_home);
            
        config.show_parent = options.get("show_parent")
            .and_then(|v| v.as_bool())
            .unwrap_or(config.show_parent);
            
        if let Some(case_value) = options.get("case_style") {
            if let Ok(case_style) = serde_json::from_value::<CaseStyle>(case_value.clone()) {
                config.case_style = case_style;
            }
        }
        
        config
    }
}

// Git Segment specific configuration types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum GitStatusFormat {
    Symbols,
    Text,
    Count,
}

impl Default for GitStatusFormat {
    fn default() -> Self {
        GitStatusFormat::Symbols
    }
}

// Git Segment configuration helper
#[derive(Debug, Clone)]
pub struct GitSegmentConfig {
    pub show_sha: bool,
    pub sha_length: u8,
    pub show_remote: bool,
    pub show_stash: bool,
    pub show_tag: bool,
    pub hide_clean_status: bool,
    pub branch_max_length: usize,
    pub status_format: GitStatusFormat,
}

impl Default for GitSegmentConfig {
    fn default() -> Self {
        Self {
            show_sha: false,
            sha_length: 7,
            show_remote: false,
            show_stash: false,
            show_tag: false,
            hide_clean_status: false,
            branch_max_length: 15,
            status_format: GitStatusFormat::Symbols,
        }
    }
}

impl GitSegmentConfig {
    pub fn from_options(options: &HashMap<String, serde_json::Value>) -> Self {
        let mut config = Self::default();
        
        config.show_sha = options.get("show_sha")
            .and_then(|v| v.as_bool())
            .unwrap_or(config.show_sha);
            
        config.sha_length = options.get("sha_length")
            .and_then(|v| v.as_u64())
            .map(|v| v.max(4).min(40) as u8)
            .unwrap_or(config.sha_length);
            
        config.show_remote = options.get("show_remote")
            .and_then(|v| v.as_bool())
            .unwrap_or(config.show_remote);
            
        config.show_stash = options.get("show_stash")
            .and_then(|v| v.as_bool())
            .unwrap_or(config.show_stash);
            
        config.show_tag = options.get("show_tag")
            .and_then(|v| v.as_bool())
            .unwrap_or(config.show_tag);
            
        config.hide_clean_status = options.get("hide_clean_status")
            .and_then(|v| v.as_bool())
            .unwrap_or(config.hide_clean_status);
            
        config.branch_max_length = options.get("branch_max_length")
            .and_then(|v| v.as_u64())
            .map(|v| v.max(5).min(50) as usize)
            .unwrap_or(config.branch_max_length);
            
        if let Some(format_value) = options.get("status_format") {
            if let Ok(format) = serde_json::from_value::<GitStatusFormat>(format_value.clone()) {
                config.status_format = format;
            }
        }
        
        config
    }
}

// Main config structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub style: StyleConfig,
    pub segments: Vec<SegmentConfig>,
    pub theme: String,
}

// Default implementation moved to ui/themes/presets.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleConfig {
    pub mode: StyleMode,
    pub separator: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StyleMode {
    Plain,
    NerdFont,
    Powerline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentConfig {
    pub id: SegmentId,
    pub enabled: bool,
    pub icon: IconConfig,
    pub colors: ColorConfig,
    pub styles: TextStyleConfig,
    pub options: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconConfig {
    pub plain: String,
    pub nerd_font: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorConfig {
    pub icon: Option<AnsiColor>,
    pub text: Option<AnsiColor>,
    pub background: Option<AnsiColor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TextStyleConfig {
    pub text_bold: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnsiColor {
    Color16 { c16: u8 },
    Color256 { c256: u8 },
    Rgb { r: u8, g: u8, b: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SegmentId {
    Model,
    Directory,
    Git,
    Usage,
    Cost,
    Session,
    OutputStyle,
    Update,
}

// Legacy compatibility structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SegmentsConfig {
    pub directory: bool,
    pub git: bool,
    pub model: bool,
    pub usage: bool,
}

// Data structures compatible with existing main.rs
#[derive(Deserialize)]
pub struct Model {
    pub id: String,
    pub display_name: String,
}

#[derive(Deserialize)]
pub struct Workspace {
    pub current_dir: String,
}

#[derive(Deserialize)]
pub struct Cost {
    pub total_cost_usd: Option<f64>,
    pub total_duration_ms: Option<u64>,
    pub total_api_duration_ms: Option<u64>,
    pub total_lines_added: Option<u32>,
    pub total_lines_removed: Option<u32>,
}

#[derive(Deserialize)]
pub struct OutputStyle {
    pub name: String,
}

#[derive(Deserialize)]
pub struct InputData {
    pub model: Model,
    pub workspace: Workspace,
    pub transcript_path: String,
    pub cost: Option<Cost>,
    pub output_style: Option<OutputStyle>,
}

// OpenAI-style nested token details
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct PromptTokensDetails {
    #[serde(default)]
    pub cached_tokens: Option<u32>,
    #[serde(default)]
    pub audio_tokens: Option<u32>,
}

// Raw usage data from different LLM providers (flexible parsing)
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct RawUsage {
    // Anthropic-style input tokens
    #[serde(default)]
    pub input_tokens: Option<u32>,

    // OpenAI-style input tokens (separate field to handle both formats)
    #[serde(default)]
    pub prompt_tokens: Option<u32>,

    // Anthropic-style output tokens
    #[serde(default)]
    pub output_tokens: Option<u32>,

    // OpenAI-style output tokens (separate field to handle both formats)
    #[serde(default)]
    pub completion_tokens: Option<u32>,

    // Total tokens (some providers only provide this)
    #[serde(default)]
    pub total_tokens: Option<u32>,

    // Anthropic-style cache fields
    #[serde(default)]
    pub cache_creation_input_tokens: Option<u32>,

    #[serde(default)]
    pub cache_read_input_tokens: Option<u32>,

    // OpenAI-style cache fields (separate fields to handle both formats)
    #[serde(default)]
    pub cache_creation_prompt_tokens: Option<u32>,

    #[serde(default)]
    pub cache_read_prompt_tokens: Option<u32>,

    #[serde(default)]
    pub cached_tokens: Option<u32>,

    // OpenAI-style nested details
    #[serde(default)]
    pub prompt_tokens_details: Option<PromptTokensDetails>,

    // Completion token details (OpenAI)
    #[serde(default)]
    pub completion_tokens_details: Option<HashMap<String, u32>>,

    // Catch unknown fields for future compatibility and debugging
    #[serde(flatten, skip_serializing)]
    pub extra: HashMap<String, serde_json::Value>,
}

// Normalized internal representation after processing
#[derive(Debug, Clone, Serialize, Default, PartialEq)]
pub struct NormalizedUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
    pub cache_creation_input_tokens: u32,
    pub cache_read_input_tokens: u32,

    // Metadata for debugging and analysis
    pub calculation_source: String,
    pub raw_data_available: Vec<String>,
}

impl NormalizedUsage {
    /// Get tokens that count toward context window
    /// This includes all tokens that consume context window space
    /// Output tokens from this turn will become input tokens in the next turn
    pub fn context_tokens(&self) -> u32 {
        self.input_tokens
            + self.cache_creation_input_tokens
            + self.cache_read_input_tokens
            + self.output_tokens
    }

    /// Get total tokens for cost calculation
    /// Priority: use total_tokens if available, otherwise sum all components
    pub fn total_for_cost(&self) -> u32 {
        if self.total_tokens > 0 {
            self.total_tokens
        } else {
            self.input_tokens
                + self.output_tokens
                + self.cache_creation_input_tokens
                + self.cache_read_input_tokens
        }
    }

    /// Get the most appropriate token count for general display
    /// For OpenAI format: use total_tokens directly
    /// For Anthropic format: use context_tokens (input + cache)
    pub fn display_tokens(&self) -> u32 {
        // For Claude/Anthropic format: prefer input-related tokens for context window display
        let context = self.context_tokens();
        if context > 0 {
            return context;
        }

        // For OpenAI format: use total_tokens when no input breakdown available
        if self.total_tokens > 0 {
            return self.total_tokens;
        }

        // Fallback to any available tokens
        self.input_tokens.max(self.output_tokens)
    }
}

impl Config {
    /// Check if current config matches the specified theme preset
    pub fn matches_theme(&self, theme_name: &str) -> bool {
        let theme_preset = crate::ui::themes::ThemePresets::get_theme(theme_name);

        // Compare style config
        if self.style.mode != theme_preset.style.mode
            || self.style.separator != theme_preset.style.separator
        {
            return false;
        }

        // Compare segments count and order
        if self.segments.len() != theme_preset.segments.len() {
            return false;
        }

        // Compare each segment config
        for (current, preset) in self.segments.iter().zip(theme_preset.segments.iter()) {
            if !self.segment_matches(current, preset) {
                return false;
            }
        }

        true
    }

    /// Check if current config has been modified from the selected theme
    pub fn is_modified_from_theme(&self) -> bool {
        !self.matches_theme(&self.theme)
    }

    /// Compare two segment configs for equality
    fn segment_matches(&self, current: &SegmentConfig, preset: &SegmentConfig) -> bool {
        current.id == preset.id
            && current.enabled == preset.enabled
            && current.icon.plain == preset.icon.plain
            && current.icon.nerd_font == preset.icon.nerd_font
            && self.color_matches(&current.colors.icon, &preset.colors.icon)
            && self.color_matches(&current.colors.text, &preset.colors.text)
            && self.color_matches(&current.colors.background, &preset.colors.background)
            && current.styles.text_bold == preset.styles.text_bold
            && current.options == preset.options
    }

    /// Compare two optional colors for equality
    fn color_matches(&self, current: &Option<AnsiColor>, preset: &Option<AnsiColor>) -> bool {
        match (current, preset) {
            (None, None) => true,
            (Some(c1), Some(c2)) => c1 == c2,
            _ => false,
        }
    }
}

impl PartialEq for AnsiColor {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AnsiColor::Color16 { c16: a }, AnsiColor::Color16 { c16: b }) => a == b,
            (AnsiColor::Color256 { c256: a }, AnsiColor::Color256 { c256: b }) => a == b,
            (
                AnsiColor::Rgb {
                    r: r1,
                    g: g1,
                    b: b1,
                },
                AnsiColor::Rgb {
                    r: r2,
                    g: g2,
                    b: b2,
                },
            ) => r1 == r2 && g1 == g2 && b1 == b2,
            _ => false,
        }
    }
}

impl RawUsage {
    /// Convert raw usage data to normalized format with intelligent token inference
    pub fn normalize(self) -> NormalizedUsage {
        let mut result = NormalizedUsage::default();
        let mut sources = Vec::new();

        // Collect available raw data fields and merge tokens with Anthropic priority
        let mut available_fields = Vec::new();

        // Merge input tokens (priority: input_tokens > prompt_tokens)
        let input = self.input_tokens.or(self.prompt_tokens).unwrap_or(0);
        if input > 0 {
            available_fields.push("input_tokens".to_string());
        }

        // Merge output tokens (priority: output_tokens > completion_tokens)
        let output = self.output_tokens.or(self.completion_tokens).unwrap_or(0);
        if output > 0 {
            available_fields.push("output_tokens".to_string());
        }

        let total = self.total_tokens.unwrap_or(0);
        if total > 0 {
            available_fields.push("total_tokens".to_string());
        }

        // Merge cache creation tokens (priority: Anthropic > OpenAI)
        let cache_creation = self
            .cache_creation_input_tokens
            .or(self.cache_creation_prompt_tokens)
            .unwrap_or(0);
        if cache_creation > 0 {
            available_fields.push("cache_creation".to_string());
        }

        // Merge cache read tokens (priority: Anthropic > OpenAI > nested format)
        let cache_read = self
            .cache_read_input_tokens
            .or(self.cache_read_prompt_tokens)
            .or(self.cached_tokens)
            .or_else(|| {
                // Fallback to OpenAI nested format
                self.prompt_tokens_details
                    .as_ref()
                    .and_then(|d| d.cached_tokens)
            })
            .unwrap_or(0);
        if cache_read > 0 {
            available_fields.push("cache_read".to_string());
        }

        result.raw_data_available = available_fields;

        // Use merged cache values (already calculated above with Anthropic priority)

        // Token calculation logic - prioritize total_tokens for OpenAI format
        let total_value = if total > 0 {
            sources.push("total_tokens_direct".to_string());
            total
        } else if input > 0 || output > 0 || cache_read > 0 || cache_creation > 0 {
            let calculated = input + output + cache_read + cache_creation;
            sources.push("total_from_components".to_string());
            calculated
        } else {
            0
        };

        // Assignment
        result.input_tokens = input;
        result.output_tokens = output;
        result.total_tokens = total_value;
        result.cache_creation_input_tokens = cache_creation;
        result.cache_read_input_tokens = cache_read;
        result.calculation_source = sources.join("+");

        result
    }
}

// Usage Segment configuration helper
#[derive(Debug, Clone)]
pub struct UsageSegmentConfig {
    pub display_format: UsageDisplayFormat,
    pub show_limit: bool,
    pub warning_threshold: u8,
    pub critical_threshold: u8,
    pub compact_format: bool,
    pub token_unit: TokenUnit,
    pub bar_show_percentage: bool,
    pub bar_show_tokens: bool,
}

impl Default for UsageSegmentConfig {
    fn default() -> Self {
        Self {
            display_format: UsageDisplayFormat::Both,
            show_limit: false,
            warning_threshold: 80,
            critical_threshold: 95,
            compact_format: true,
            token_unit: TokenUnit::Auto,
            bar_show_percentage: true,
            bar_show_tokens: false,
        }
    }
}

impl UsageSegmentConfig {
    pub fn from_options(options: &HashMap<String, serde_json::Value>) -> Self {
        let mut config = Self::default();
        
        if let Some(format_value) = options.get("display_format") {
            if let Ok(format) = serde_json::from_value::<UsageDisplayFormat>(format_value.clone()) {
                config.display_format = format;
            }
        }
        
        config.show_limit = options.get("show_limit")
            .and_then(|v| v.as_bool())
            .unwrap_or(config.show_limit);
            
        config.warning_threshold = options.get("warning_threshold")
            .and_then(|v| v.as_u64())
            .map(|v| v.min(100) as u8)
            .unwrap_or(config.warning_threshold);
            
        config.critical_threshold = options.get("critical_threshold")
            .and_then(|v| v.as_u64())
            .map(|v| v.min(100) as u8)
            .unwrap_or(config.critical_threshold);
            
        config.compact_format = options.get("compact_format")
            .and_then(|v| v.as_bool())
            .unwrap_or(config.compact_format);
            
        if let Some(unit_value) = options.get("token_unit") {
            if let Ok(unit) = serde_json::from_value::<TokenUnit>(unit_value.clone()) {
                config.token_unit = unit;
            }
        }
        
        config.bar_show_percentage = options.get("bar_show_percentage")
            .and_then(|v| v.as_bool())
            .unwrap_or(config.bar_show_percentage);
            
        config.bar_show_tokens = options.get("bar_show_tokens")
            .and_then(|v| v.as_bool())
            .unwrap_or(config.bar_show_tokens);
        
        config
    }
}

// Model Segment configuration helper
#[derive(Debug, Clone)]
pub struct ModelSegmentConfig {
    pub display_format: ModelDisplayFormat,
    pub show_version: bool,
    pub abbreviate_names: bool,
    pub custom_names: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelDisplayFormat {
    Name,      // 仅显示模型名称
    Full,      // 显示完整信息
    Custom,    // 使用自定义名称
}

impl Default for ModelSegmentConfig {
    fn default() -> Self {
        Self {
            display_format: ModelDisplayFormat::Name,
            show_version: false,
            abbreviate_names: true,
            custom_names: std::collections::HashMap::new(),
        }
    }
}

impl ModelSegmentConfig {
    pub fn from_options(options: &HashMap<String, serde_json::Value>) -> Self {
        let mut config = Self::default();
        
        if let Some(format_value) = options.get("display_format") {
            if let Ok(format) = serde_json::from_value::<ModelDisplayFormat>(format_value.clone()) {
                config.display_format = format;
            }
        }
        
        config.show_version = options.get("show_version")
            .and_then(|v| v.as_bool())
            .unwrap_or(config.show_version);
            
        config.abbreviate_names = options.get("abbreviate_names")
            .and_then(|v| v.as_bool())
            .unwrap_or(config.abbreviate_names);
            
        if let Some(names_value) = options.get("custom_names") {
            if let Ok(names) = serde_json::from_value::<std::collections::HashMap<String, String>>(names_value.clone()) {
                config.custom_names = names;
            }
        }
        
        config
    }
}

// Session Segment configuration helper
#[derive(Debug, Clone)]
pub struct SessionSegmentConfig {
    pub time_format: TimeFormat,
    pub show_milliseconds: bool,
    pub compact_format: bool,
    pub show_idle_time: bool,
    pub show_line_changes: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeFormat {
    Auto,      // 自动选择最合适的单位
    Short,     // 简短格式 (1m30s)
    Long,      // 完整格式 (1 min 30 sec)
    Digital,   // 数字格式 (01:30)
}

impl Default for SessionSegmentConfig {
    fn default() -> Self {
        Self {
            time_format: TimeFormat::Auto,
            show_milliseconds: false,
            compact_format: true,
            show_idle_time: false,
            show_line_changes: true,
        }
    }
}

impl SessionSegmentConfig {
    pub fn from_options(options: &HashMap<String, serde_json::Value>) -> Self {
        let mut config = Self::default();
        
        if let Some(format_value) = options.get("time_format") {
            if let Ok(format) = serde_json::from_value::<TimeFormat>(format_value.clone()) {
                config.time_format = format;
            }
        }
        
        config.show_milliseconds = options.get("show_milliseconds")
            .and_then(|v| v.as_bool())
            .unwrap_or(config.show_milliseconds);
            
        config.compact_format = options.get("compact_format")
            .and_then(|v| v.as_bool())
            .unwrap_or(config.compact_format);
            
        config.show_idle_time = options.get("show_idle_time")
            .and_then(|v| v.as_bool())
            .unwrap_or(config.show_idle_time);
        
        config.show_line_changes = options.get("show_line_changes")
            .and_then(|v| v.as_bool())
            .unwrap_or(config.show_line_changes);
        
        config
    }
}

// OutputStyle Segment configuration helper
#[derive(Debug, Clone)]
pub struct OutputStyleSegmentConfig {
    pub display_format: OutputStyleDisplayFormat,
    pub abbreviate_names: bool,
    pub show_description: bool,
    pub custom_names: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputStyleDisplayFormat {
    Name,        // 只显示名称
    Full,        // 显示完整信息
    Abbreviated, // 显示缩写
    Custom,      // 使用自定义名称映射
}

impl Default for OutputStyleSegmentConfig {
    fn default() -> Self {
        Self {
            display_format: OutputStyleDisplayFormat::Name,
            abbreviate_names: false,
            show_description: false,
            custom_names: HashMap::new(),
        }
    }
}

impl OutputStyleSegmentConfig {
    pub fn from_options(options: &HashMap<String, serde_json::Value>) -> Self {
        let mut config = Self::default();
        
        if let Some(format_value) = options.get("display_format") {
            if let Ok(format) = serde_json::from_value::<OutputStyleDisplayFormat>(format_value.clone()) {
                config.display_format = format;
            }
        }
        
        config.abbreviate_names = options.get("abbreviate_names")
            .and_then(|v| v.as_bool())
            .unwrap_or(config.abbreviate_names);
            
        config.show_description = options.get("show_description")
            .and_then(|v| v.as_bool())
            .unwrap_or(config.show_description);
        
        // Parse custom_names if provided
        if let Some(custom_names_value) = options.get("custom_names") {
            if let Ok(custom_names) = serde_json::from_value::<HashMap<String, String>>(custom_names_value.clone()) {
                config.custom_names = custom_names;
            }
        }
        
        config
    }
}

// Cost Segment configuration helper
#[derive(Debug, Clone)]
pub struct CostSegmentConfig {
    pub currency_format: CurrencyFormat,
    pub precision: u8,
    pub show_breakdown: bool,
    pub threshold_warning: f64,
    pub cumulative_display: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CurrencyFormat {
    Auto,       // 自动格式 ($0.05 或 $1.50)
    Fixed,      // 固定小数位 ($0.05)
    Compact,    // 紧凑格式 (5¢ 或 $1.5)
    Scientific, // 科学记数法 (5e-2)
}

impl Default for CostSegmentConfig {
    fn default() -> Self {
        Self {
            currency_format: CurrencyFormat::Auto,
            precision: 2,
            show_breakdown: false,
            threshold_warning: 1.0,
            cumulative_display: false,
        }
    }
}

impl CostSegmentConfig {
    pub fn from_options(options: &HashMap<String, serde_json::Value>) -> Self {
        let mut config = Self::default();
        
        if let Some(format_value) = options.get("currency_format") {
            if let Ok(format) = serde_json::from_value::<CurrencyFormat>(format_value.clone()) {
                config.currency_format = format;
            }
        }
        
        config.precision = options.get("precision")
            .and_then(|v| v.as_u64())
            .map(|v| v.min(6) as u8)
            .unwrap_or(config.precision);
            
        config.show_breakdown = options.get("show_breakdown")
            .and_then(|v| v.as_bool())
            .unwrap_or(config.show_breakdown);
            
        config.threshold_warning = options.get("threshold_warning")
            .and_then(|v| v.as_f64())
            .unwrap_or(config.threshold_warning);
            
        config.cumulative_display = options.get("cumulative_display")
            .and_then(|v| v.as_bool())
            .unwrap_or(config.cumulative_display);
        
        config
    }
}

// Legacy alias for backward compatibility
pub type Usage = RawUsage;

#[derive(Deserialize)]
pub struct Message {
    pub usage: Option<Usage>,
}

#[derive(Deserialize)]
pub struct TranscriptEntry {
    pub r#type: Option<String>,
    pub message: Option<Message>,
    #[serde(rename = "leafUuid")]
    pub leaf_uuid: Option<String>,
    pub uuid: Option<String>,
    #[serde(rename = "parentUuid")]
    pub parent_uuid: Option<String>,
    pub summary: Option<String>,
}
