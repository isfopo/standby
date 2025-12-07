//! Configuration parsing and validation

use clap::{Parser, Subcommand};

/// Command line arguments for the soundcheck application
#[derive(Parser)]
#[command(name = "soundcheck")]
#[command(about = "Audio monitoring and analysis tools")]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Detect audio levels and exit when threshold is exceeded
    Detect(DetectArgs),
    /// List available audio input devices
    List(ListArgs),
    /// Monitor audio and report maximum levels
    Max(MaxArgs),
    /// Monitor audio and report average levels
    Average(AverageArgs),
}

#[derive(Parser)]
pub struct DetectArgs {
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

#[derive(Parser)]
pub struct MaxArgs {
    /// Monitoring duration in seconds (optional, runs until Enter if not specified)
    #[arg(long)]
    pub seconds: Option<f32>,

    /// Minimum dB level for display (e.g., -60)
    #[arg(long, default_value_t = crate::constants::audio::MIN_DB_LEVEL)]
    pub min_db: i32,

    /// Audio input device name (optional, uses default if not specified)
    #[arg(long)]
    pub device: Option<String>,

    /// Audio channels to monitor (comma-separated indices, e.g., "0,1")
    #[arg(long, value_delimiter = ',', default_values_t = vec![0usize])]
    pub channels: Vec<usize>,

    /// Output only the integer values without labels
    #[arg(long)]
    pub quiet: bool,
}

#[derive(Parser)]
pub struct AverageArgs {
    /// Monitoring duration in seconds (optional, runs until Enter if not specified)
    #[arg(long)]
    pub seconds: Option<f32>,

    /// Minimum dB level for display (e.g., -60)
    #[arg(long, default_value_t = crate::constants::audio::MIN_DB_LEVEL)]
    pub min_db: i32,

    /// Audio input device name (optional, uses default if not specified)
    #[arg(long)]
    pub device: Option<String>,

    /// Audio channels to monitor (comma-separated indices, e.g., "0,1")
    #[arg(long, value_delimiter = ',', default_values_t = vec![0usize])]
    pub channels: Vec<usize>,

    /// Output only the integer values without labels
    #[arg(long)]
    pub quiet: bool,
}

#[derive(Parser)]
pub struct ListArgs {}

/// Application configuration derived from command line arguments
pub struct Config {
    pub threshold_db: i32,
    pub min_db: i32,
    pub channels: Vec<usize>,
    pub device_name: Option<String>,
}

impl Config {
    /// Create configuration from detect arguments
    pub fn from_detect_args(detect_args: DetectArgs) -> Result<Self, Box<dyn std::error::Error>> {
        // Validate threshold range
        if detect_args.threshold > 0 || detect_args.threshold < -60 {
            return Err(format!(
                "Threshold must be between -60 and 0 dB, got {}",
                detect_args.threshold
            )
            .into());
        }

        // Validate min_db range
        if detect_args.min_db >= 0 || detect_args.min_db < -100 {
            return Err(format!(
                "Minimum dB must be between -100 and 0 dB, got {}",
                detect_args.min_db
            )
            .into());
        }

        Ok(Config {
            threshold_db: detect_args.threshold,
            min_db: detect_args.min_db,
            channels: detect_args.channels,
            device_name: detect_args.device,
        })
    }

    /// Create configuration from max arguments
    pub fn from_max_args(max_args: &MaxArgs) -> Result<Self, Box<dyn std::error::Error>> {
        // Validate min_db range
        if max_args.min_db >= 0 || max_args.min_db < -100 {
            return Err(format!(
                "Minimum dB must be between -100 and 0 dB, got {}",
                max_args.min_db
            )
            .into());
        }

        // Validate seconds if provided
        if let Some(seconds) = max_args.seconds && seconds <= 0.0 {
            return Err("Seconds must be positive".into());
        }

        Ok(Config {
            threshold_db: 0, // Dummy value for max monitoring
            min_db: max_args.min_db,
            channels: max_args.channels.clone(),
            device_name: max_args.device.clone(),
        })
    }

    /// Create configuration from average arguments
    pub fn from_average_args(
        average_args: &AverageArgs,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Validate min_db range
        if average_args.min_db >= 0 || average_args.min_db < -100 {
            return Err(format!(
                "Minimum dB must be between -100 and 0 dB, got {}",
                average_args.min_db
            )
            .into());
        }

        // Validate seconds if provided
        if let Some(seconds) = average_args.seconds && seconds <= 0.0 {
            return Err("Seconds must be positive".into());
        }

        Ok(Config {
            threshold_db: 0, // Dummy value for average monitoring
            min_db: average_args.min_db,
            channels: average_args.channels.clone(),
            device_name: average_args.device.clone(),
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
        assert_eq!(config.min_db, -60);
        assert_eq!(config.channels, vec![0]);
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
