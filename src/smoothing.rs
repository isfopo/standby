//! Audio level smoothing and processing utilities

/// Applies exponential smoothing to audio levels for stable display
pub struct AudioSmoother {
    smoothed_value: f32,
    display_value: f32,
}

impl AudioSmoother {
    /// Create a new smoother initialized to the given value
    pub fn new(initial_value: f32) -> Self {
        Self {
            smoothed_value: initial_value,
            display_value: initial_value,
        }
    }

    /// Update the smoother with a new raw audio value
    /// Returns the display value after smoothing
    pub fn update(&mut self, raw_value: f32) -> f32 {
        // Two-stage smoothing for ultra-smooth display
        // First stage: moderate smoothing of raw audio data
        const AUDIO_SMOOTHING: f32 = 0.4;
        self.smoothed_value =
            self.smoothed_value * (1.0 - AUDIO_SMOOTHING) + raw_value * AUDIO_SMOOTHING;

        // Second stage: heavy smoothing for display (easing effect)
        const DISPLAY_SMOOTHING: f32 = 0.15;
        self.display_value = self.display_value * (1.0 - DISPLAY_SMOOTHING) + self.smoothed_value * DISPLAY_SMOOTHING;

        self.display_value
    }

    /// Get the current smoothed value
    pub fn smoothed(&self) -> f32 {
        self.smoothed_value
    }

    /// Get the current display value
    pub fn display(&self) -> f32 {
        self.display_value
    }
}

/// Convert linear amplitude to decibels
pub fn amplitude_to_db(amplitude: f32) -> f32 {
    if amplitude > 0.0 {
        20.0 * amplitude.log10()
    } else {
        -60.0 // Minimum dB level
    }
}

/// Convert decibels to linear amplitude
pub fn db_to_amplitude(db: f32) -> f32 {
    10.0f32.powf(db / 20.0)
}