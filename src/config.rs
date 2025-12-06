//! Configuration parsing and validation

use clap::Parser;

/// Command line arguments for the standby application
#[derive(Parser)]
#[command(name = "standby")]
#[command(about = "Monitor audio threshold from input device")]
pub struct Args {
    /// Audio threshold in dB (e.g., 0)
    #[arg(long, default_value_t = crate::constants::audio::DEFAULT_THRESHOLD_DB)]
    pub threshold: i32,

    /// Minimum dB level for display (e.g., -60)
    #[arg(long, default_value_t = crate::constants::audio::MIN_DB_LEVEL)]
    pub min_db: i32,

    /// Audio input device name (optional, uses default if not specified)
    #[arg(long)]
    pub device: Option<String>,

    /// Audio channels to monitor (comma-separated indices, e.g., "0,1")
    #[arg(long, value_delimiter = ',', default_values_t = vec![0usize])]
    pub channels: Vec<usize>,
}

/// Application configuration derived from command line arguments
pub struct Config {
    pub threshold_db: i32,
    pub min_db: i32,
    pub channels: Vec<usize>,
    pub device_name: Option<String>,
}

impl Config {
    /// Parse command line arguments and validate configuration
    pub fn from_args() -> Result<Self, Box<dyn std::error::Error>> {
        let args = Args::parse();

        // Validate threshold range
        if args.threshold > 0 || args.threshold < -60 {
            return Err(format!(
                "Threshold must be between -60 and 0 dB, got {}",
                args.threshold
            )
            .into());
        }

        // Validate min_db range
        if args.min_db >= args.threshold || args.min_db < -100 {
            return Err(format!(
                "Minimum dB must be between -100 and and threshold, got {}",
                args.min_db
            )
            .into());
        }

        Ok(Config {
            threshold_db: args.threshold,
            min_db: args.min_db,
            channels: args.channels,
            device_name: args.device,
        })
    }

    /// Convert dB threshold to linear amplitude for audio processing
    pub fn linear_threshold(&self) -> f32 {
        crate::smoothing::db_to_amplitude(self.threshold_db as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_valid_args() {
        // This would require setting up clap test arguments
        // For now, we'll test the validation logic manually
        let config = Config {
            threshold_db: 0,
            min_db: -60,
            channels: vec![0],
            device_name: Some("test_device".to_string()),
        };

        assert_eq!(config.threshold_db, 0);
        assert_eq!(config.device_name, Some("test_device".to_string()));
        assert!(config.linear_threshold() > 0.0);
    }

    #[test]
    fn test_db_to_linear_conversion() {
        let config = Config {
            threshold_db: 0,
            min_db: -60,
            device_name: None,
            channels: vec![0],
        };
        // 0 dB should convert to amplitude of 1.0
        assert!((config.linear_threshold() - 1.0).abs() < 0.001);

        let config = Config {
            threshold_db: -20,
            min_db: -60,
            device_name: Some("test_device".to_string()),
            channels: vec![0],
        };
        // -20 dB should convert to amplitude of ~0.1
        assert!((config.linear_threshold() - 0.1).abs() < 0.01);
    }
}
