//! Audio device handling and stream processing

use crate::error::{AppError, AppResult};
use cpal::traits::{DeviceTrait, HostTrait};
use std::sync::{Arc, Mutex};

/// Audio configuration and device information
pub struct AudioConfig {
    pub device_name: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub selected_channels: Vec<usize>,
}

/// Find and configure an audio input device
pub fn setup_audio_device(device_name: Option<String>, channels: &[usize]) -> AppResult<(cpal::Device, AudioConfig)> {
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

    // Get supported input configs and determine sample rate and channels from device
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

    // Validate selected channels
    let max_supported_channels = config_range.channels() as usize;
    for &ch in channels {
        if ch >= max_supported_channels {
            return Err(AppError::AudioDevice(format!(
                "Channel {} not supported by device config (max {})",
                ch, max_supported_channels - 1
            )));
        }
    }

    let audio_config = AudioConfig {
        device_name,
        sample_rate,
        channels: config_range.channels(),
        selected_channels: channels.to_vec(),
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
    current_db: Arc<Mutex<Vec<f32>>>,
    smoothed_db: Arc<Mutex<Vec<f32>>>,
    display_db: Arc<Mutex<Vec<f32>>>,
    threshold_reached: Arc<Mutex<Vec<bool>>>,
    linear_threshold: f32,
    selected_channels: &[usize],
    total_channels: usize,
) -> impl FnMut(&[f32], &cpal::InputCallbackInfo) + Send + 'static {
    let selected_channels = selected_channels.to_vec();
    move |data: &[f32], _: &cpal::InputCallbackInfo| {
        let mut current_db_vec = current_db.lock().unwrap();
        let mut smoothed_vec = smoothed_db.lock().unwrap();
        let mut display_vec = display_db.lock().unwrap();
        let mut threshold_vec = threshold_reached.lock().unwrap();

        for (i, &ch) in selected_channels.iter().enumerate() {
            // Extract samples for this channel
            let channel_samples: Vec<f32> = data.iter().skip(ch).step_by(total_channels).map(|&s| s.abs()).collect();
            let max_sample = channel_samples.iter().fold(0.0f32, |a, &b| a.max(b));

            let current_db_value = if max_sample > 0.0 {
                20.0 * max_sample.log10()
            } else {
                crate::constants::audio::MIN_DB_LEVEL
            };

            // Update current dB
            current_db_vec[i] = current_db_value;

            // Apply smoothing
            let audio_smoothing = crate::constants::smoothing::AUDIO_SMOOTHING_FACTOR;
            smoothed_vec[i] = smoothed_vec[i] * (1.0 - audio_smoothing) + current_db_value * audio_smoothing;

            let display_smoothing = crate::constants::smoothing::DISPLAY_SMOOTHING_FACTOR;
            display_vec[i] = display_vec[i] * (1.0 - display_smoothing) + smoothed_vec[i] * display_smoothing;

            // Check threshold
            if max_sample > linear_threshold {
                threshold_vec[i] = true;
            }
        }
    }
}
