use crate::config::{
    AnsiColor, ColorConfig, IconConfig, SegmentConfig, SegmentId, TextStyleConfig,
};
use std::collections::HashMap;

pub fn model_segment() -> SegmentConfig {
    SegmentConfig {
        id: SegmentId::Model,
        enabled: true,
        icon: IconConfig {
            plain: "🤖".to_string(),
            nerd_font: "\u{e26d}".to_string(),
        },
        colors: ColorConfig {
            icon: Some(AnsiColor::Color16 { c16: 14 }),
            text: Some(AnsiColor::Color16 { c16: 14 }),
            background: None,
        },
        styles: TextStyleConfig { text_bold: true },
        options: {
            let mut opts = HashMap::new();
            opts.insert("display_format".to_string(), serde_json::Value::String("name".to_string()));
            opts.insert("show_version".to_string(), serde_json::Value::Bool(false));
            opts.insert("abbreviate_names".to_string(), serde_json::Value::Bool(true));
            opts
        },
    }
}

pub fn directory_segment() -> SegmentConfig {
    SegmentConfig {
        id: SegmentId::Directory,
        enabled: true,
        icon: IconConfig {
            plain: "📁".to_string(),
            nerd_font: "\u{f024b}".to_string(),
        },
        colors: ColorConfig {
            icon: Some(AnsiColor::Color16 { c16: 11 }),
            text: Some(AnsiColor::Color16 { c16: 10 }),
            background: None,
        },
        styles: TextStyleConfig { text_bold: true },
        options: {
            let mut opts = HashMap::new();
            opts.insert("max_length".to_string(), serde_json::Value::Number(serde_json::Number::from(20)));
            opts.insert("show_full_path".to_string(), serde_json::Value::Bool(false));
            opts.insert("abbreviate_home".to_string(), serde_json::Value::Bool(true));
            opts.insert("show_parent".to_string(), serde_json::Value::Bool(false));
            opts.insert("case_style".to_string(), serde_json::Value::String("original".to_string()));
            opts
        },
    }
}

pub fn git_segment() -> SegmentConfig {
    SegmentConfig {
        id: SegmentId::Git,
        enabled: true,
        icon: IconConfig {
            plain: "🌿".to_string(),
            nerd_font: "\u{f02a2}".to_string(),
        },
        colors: ColorConfig {
            icon: Some(AnsiColor::Color16 { c16: 12 }),
            text: Some(AnsiColor::Color16 { c16: 12 }),
            background: None,
        },
        styles: TextStyleConfig { text_bold: true },
        options: {
            let mut opts = HashMap::new();
            opts.insert("show_sha".to_string(), serde_json::Value::Bool(false));
            opts.insert("sha_length".to_string(), serde_json::Value::Number(serde_json::Number::from(7)));
            opts.insert("show_remote".to_string(), serde_json::Value::Bool(true));
            opts.insert("show_stash".to_string(), serde_json::Value::Bool(false));
            opts.insert("show_tag".to_string(), serde_json::Value::Bool(false));
            opts.insert("hide_clean_status".to_string(), serde_json::Value::Bool(false));
            opts.insert("branch_max_length".to_string(), serde_json::Value::Number(serde_json::Number::from(20)));
            opts.insert("status_format".to_string(), serde_json::Value::String("symbols".to_string()));
            opts
        },
    }
}

pub fn usage_segment() -> SegmentConfig {
    SegmentConfig {
        id: SegmentId::Usage,
        enabled: true,
        icon: IconConfig {
            plain: "⚡️".to_string(),
            nerd_font: "\u{f49b}".to_string(),
        },
        colors: ColorConfig {
            icon: Some(AnsiColor::Color16 { c16: 13 }),
            text: Some(AnsiColor::Color16 { c16: 13 }),
            background: None,
        },
        styles: TextStyleConfig { text_bold: true },
        options: {
            let mut opts = HashMap::new();
            opts.insert("display_format".to_string(), serde_json::Value::String("both".to_string()));
            opts.insert("show_limit".to_string(), serde_json::Value::Bool(false));
            opts.insert("warning_threshold".to_string(), serde_json::Value::Number(serde_json::Number::from(80)));
            opts.insert("critical_threshold".to_string(), serde_json::Value::Number(serde_json::Number::from(95)));
            opts.insert("compact_format".to_string(), serde_json::Value::Bool(true));
            opts.insert("token_unit".to_string(), serde_json::Value::String("auto".to_string()));
            opts.insert("bar_show_percentage".to_string(), serde_json::Value::Bool(true));
            opts.insert("bar_show_tokens".to_string(), serde_json::Value::Bool(false));
            opts
        },
    }
}

pub fn cost_segment() -> SegmentConfig {
    SegmentConfig {
        id: SegmentId::Cost,
        enabled: false,
        icon: IconConfig {
            plain: "💰".to_string(),
            nerd_font: "\u{eec1}".to_string(),
        },
        colors: ColorConfig {
            icon: Some(AnsiColor::Color16 { c16: 3 }),
            text: Some(AnsiColor::Color16 { c16: 3 }),
            background: None,
        },
        styles: TextStyleConfig { text_bold: true },
        options: {
            let mut opts = HashMap::new();
            opts.insert("currency_format".to_string(), serde_json::Value::String("auto".to_string()));
            opts.insert("precision".to_string(), serde_json::Value::Number(serde_json::Number::from(2)));
            opts.insert("show_breakdown".to_string(), serde_json::Value::Bool(false));
            opts.insert("threshold_warning".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(1.0).unwrap()));
            opts.insert("cumulative_display".to_string(), serde_json::Value::Bool(false));
            opts
        },
    }
}

pub fn session_segment() -> SegmentConfig {
    SegmentConfig {
        id: SegmentId::Session,
        enabled: false,
        icon: IconConfig {
            plain: "⏱️".to_string(),
            nerd_font: "\u{f19bb}".to_string(),
        },
        colors: ColorConfig {
            icon: Some(AnsiColor::Color16 { c16: 2 }),
            text: Some(AnsiColor::Color16 { c16: 2 }),
            background: None,
        },
        styles: TextStyleConfig { text_bold: true },
        options: {
            let mut opts = HashMap::new();
            opts.insert("time_format".to_string(), serde_json::Value::String("auto".to_string()));
            opts.insert("show_milliseconds".to_string(), serde_json::Value::Bool(false));
            opts.insert("compact_format".to_string(), serde_json::Value::Bool(true));
            opts.insert("show_idle_time".to_string(), serde_json::Value::Bool(false));
            opts.insert("show_line_changes".to_string(), serde_json::Value::Bool(true));
            opts
        },
    }
}

pub fn output_style_segment() -> SegmentConfig {
    SegmentConfig {
        id: SegmentId::OutputStyle,
        enabled: false,
        icon: IconConfig {
            plain: "🎯".to_string(),
            nerd_font: "\u{f12f5}".to_string(),
        },
        colors: ColorConfig {
            icon: Some(AnsiColor::Color16 { c16: 6 }),
            text: Some(AnsiColor::Color16 { c16: 6 }),
            background: None,
        },
        styles: TextStyleConfig { text_bold: true },
        options: {
            let mut opts = HashMap::new();
            opts.insert("display_format".to_string(), serde_json::Value::String("name".to_string()));
            opts.insert("abbreviate_names".to_string(), serde_json::Value::Bool(false));
            opts.insert("show_description".to_string(), serde_json::Value::Bool(false));
            opts
        },
    }
}
