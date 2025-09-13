use super::{Segment, SegmentData};
use crate::config::{InputData, SegmentId, CostSegmentConfig, CurrencyFormat};
use std::collections::HashMap;

pub struct CostSegment {
    config: CostSegmentConfig,
}

impl CostSegment {
    pub fn new() -> Self {
        Self {
            config: CostSegmentConfig::default(),
        }
    }
    
    pub fn with_config(options: &HashMap<String, serde_json::Value>) -> Self {
        Self {
            config: CostSegmentConfig::from_options(options),
        }
    }
    
    fn format_cost(&self, cost: f64) -> String {
        match self.config.currency_format {
            CurrencyFormat::Auto => {
                if cost == 0.0 {
                    "$0".to_string()
                } else if cost < 0.01 {
                    format!("${:.3}", cost)
                } else if cost < 1.0 {
                    format!("${:.2}", cost)
                } else {
                    format!("${:.2}", cost)
                }
            },
            CurrencyFormat::Fixed => {
                format!("${:.prec$}", cost, prec = self.config.precision as usize)
            },
            CurrencyFormat::Compact => {
                if cost == 0.0 {
                    "$0".to_string()
                } else if cost < 0.01 {
                    format!("{:.1}¢", cost * 100.0)
                } else if cost < 1.0 {
                    format!("{:.0}¢", cost * 100.0)
                } else {
                    format!("${:.1}", cost)
                }
            },
            CurrencyFormat::Scientific => {
                format!("${:.2e}", cost)
            },
        }
    }
    
    fn is_warning_threshold(&self, cost: f64) -> bool {
        cost >= self.config.threshold_warning
    }
}

impl Segment for CostSegment {
    fn collect(&self, input: &InputData) -> Option<SegmentData> {
        let cost_data = input.cost.as_ref()?;

        // Primary display: total cost
        let cost = cost_data.total_cost_usd?;
        let primary = self.format_cost(cost);

        // Secondary display: cumulative if enabled
        let secondary = if self.config.cumulative_display {
            // Could show cumulative cost if available
            String::new() // Placeholder for cumulative logic
        } else {
            String::new()
        };

        let mut metadata = HashMap::new();
        metadata.insert("cost".to_string(), cost.to_string());
        metadata.insert("currency_format".to_string(), format!("{:?}", self.config.currency_format));
        metadata.insert("warning_threshold".to_string(), self.is_warning_threshold(cost).to_string());
        
        if let Some(duration) = cost_data.total_duration_ms {
            metadata.insert("duration_ms".to_string(), duration.to_string());
        }

        Some(SegmentData {
            primary,
            secondary,
            metadata,
        })
    }

    fn id(&self) -> SegmentId {
        SegmentId::Cost
    }
}

impl Default for CostSegment {
    fn default() -> Self {
        Self::new()
    }
}
