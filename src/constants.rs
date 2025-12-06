//! Application constants and configuration values

/// Audio processing constants
pub mod audio {
    /// Minimum dB level
    pub const MIN_DB_LEVEL: f32 = -60.0;
    /// Default threshold dB level
    pub const DEFAULT_THRESHOLD_DB: i32 = 0;
    /// Buffer size for audio streams
    pub const BUFFER_SIZE: cpal::BufferSize = cpal::BufferSize::Default;
}

/// UI display constants
pub mod ui {
    /// UI update interval in milliseconds
    pub const UPDATE_INTERVAL_MS: u64 = 10;
    /// Bar width calculation accounts for borders
    pub const BAR_BORDER_WIDTH: usize = 2;
}

/// Smoothing algorithm constants
pub mod smoothing {
    /// First stage audio smoothing factor (higher = more responsive)
    pub const AUDIO_SMOOTHING_FACTOR: f32 = 0.4;
    /// Second stage display smoothing factor (lower = smoother)
    pub const DISPLAY_SMOOTHING_FACTOR: f32 = 0.15;
}
