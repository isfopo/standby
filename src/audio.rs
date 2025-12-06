//! Audio device handling and stream processing

use crate::error::{AppError, AppResult};
use cpal::traits::{DeviceTrait, HostTrait};
use std::sync::{Arc, Mutex};

/// Audio configuration and device information
pub struct AudioConfig {
    pub device_name: String,
    pub sample_rate: u32,
    pub channels: u16,
}

/// Find and configure an audio input device
pub fn setup_audio_device(device_name: Option<String>) -> AppResult<(cpal::Device, AudioConfig)> {
    // Setup audio
    let host = cpal::default_host();

    // Get input device
    let device = if let Some(name) = device_name {
        host.input_devices()?
            .find(|d| d.name().map(|n| n == name).unwrap_or(false))
            .ok_or_else(|| AppError::AudioDevice("Specified device not found".to_string()))?
    } else {
        host.default_input_device()
            .ok_or_else(|| AppError::AudioDevice("No default input device available".to_string()))?
    };

    let device_name = device.name()?;

    // Get supported input configs and determine sample rate from device
    let mut supported_configs = device.supported_input_configs()?;
    let config_range = supported_configs
        .next()
        .ok_or_else(|| AppError::AudioDevice("No supported input configs found".to_string()))?;

    // Use the minimum sample rate as default, or a common rate if available
    let sample_rate = if config_range.min_sample_rate().0 <= 44100 && config_range.max_sample_rate().0 >= 44100 {
        44100 // Prefer 44.1kHz if supported
    } else {
        config_range.min_sample_rate().0 // Otherwise use minimum supported
    };

    // Ensure channels are supported
    let channels = if config_range.channels() >= crate::constants::audio::DEFAULT_CHANNELS {
        crate::constants::audio::DEFAULT_CHANNELS
    } else {
        config_range.channels()
    };

    let audio_config = AudioConfig {
        device_name,
        sample_rate,
        channels,
    };

    Ok((device, audio_config))
}

/// Build an audio input stream with the given callback
pub fn build_audio_stream<F>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    data_callback: F,
) -> AppResult<cpal::Stream>
where
    F: FnMut(&[f32], &cpal::InputCallbackInfo) + Send + 'static,
{
    let stream = device.build_input_stream(
        config,
        data_callback,
        |err| eprintln!("Audio stream error: {}", err),
        None,
    )?;

    Ok(stream)
}

/// Audio processing callback that updates shared state
pub fn create_audio_callback(
    current_db: Arc<Mutex<f32>>,
    smoothed_db: Arc<Mutex<f32>>,
    display_db: Arc<Mutex<f32>>,
    threshold_reached: Arc<Mutex<bool>>,
    linear_threshold: f32,
) -> impl FnMut(&[f32], &cpal::InputCallbackInfo) + Send + 'static {
    move |data: &[f32], _: &cpal::InputCallbackInfo| {
        let max_sample = data.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        let current_db_value = if max_sample > 0.0 {
            20.0 * max_sample.log10()
        } else {
            -60.0
        };

        // Update current dB
        *current_db.lock().unwrap() = current_db_value;

        // Apply smoothing
        let mut smoothed = smoothed_db.lock().unwrap();
        let mut display = display_db.lock().unwrap();

        // Two-stage smoothing
        let audio_smoothing = crate::constants::smoothing::AUDIO_SMOOTHING_FACTOR;
        *smoothed = *smoothed * (1.0 - audio_smoothing) + current_db_value * audio_smoothing;

        let display_smoothing = crate::constants::smoothing::DISPLAY_SMOOTHING_FACTOR;
        *display = *display * (1.0 - display_smoothing) + *smoothed * display_smoothing;

        // Check threshold
        let mut threshold_flag = threshold_reached.lock().unwrap();
        if max_sample > linear_threshold {
            *threshold_flag = true;
        }
    }
}
