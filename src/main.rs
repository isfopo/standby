mod audio;
mod smoothing;
mod ui;

use clap::Parser;
use cpal::traits::{DeviceTrait, StreamTrait};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// A terminal application that monitors audio input and detects when it exceeds a threshold.
#[derive(Parser)]
#[command(name = "standby")]
#[command(about = "Monitor audio threshold from input device")]
struct Args {
    /// Audio threshold in dB (e.g., -20)
    #[arg(long)]
    threshold: i32,

    /// Audio input device name (optional, uses default if not specified)
    #[arg(long)]
    device: Option<String>,
}

// Application state (kept in main for now, could be moved to a separate module later)
struct AppState {
    device_name: String,
    current_db: f32,
    smoothed_db: f32,
    display_db: f32,
    threshold_db: i32,
    status: String,
    threshold_reached: bool,
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let db_threshold = args.threshold;
    let linear_threshold = 10.0f32.powf(db_threshold as f32 / 20.0);
    let device_name_arg = args.device.clone();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Setup audio device
    let (device, audio_config) = audio::setup_audio_device(device_name_arg)?;
    let device_name = audio_config.device_name;

    let config = cpal::StreamConfig {
        channels: audio_config.channels,
        sample_rate: cpal::SampleRate(audio_config.sample_rate),
        buffer_size: cpal::BufferSize::Default,
    };

    let device_name = device.name()?;

    // Get supported config
    let _config = device.default_input_config()?;

    // For simplicity, assume f32, mono, 44100
    let sample_rate = 44100;
    let channels = 1;
    let config = cpal::StreamConfig {
        channels,
        sample_rate: cpal::SampleRate(sample_rate),
        buffer_size: cpal::BufferSize::Default,
    };

    // Shared state
    let current_db = Arc::new(Mutex::new(-60.0f32));
    let smoothed_db = Arc::new(Mutex::new(-60.0f32));
    let display_db = Arc::new(Mutex::new(-60.0f32));
    let threshold_reached = Arc::new(Mutex::new(false));

    let state = Arc::new(Mutex::new(AppState {
        device_name: device_name.clone(),
        current_db: -60.0,
        smoothed_db: -60.0,
        display_db: -60.0,
        threshold_db: db_threshold,
        status: format!("Monitoring {}... Press Ctrl+C to quit.", device_name),
        threshold_reached: false,
    }));

    // Build audio stream
    let audio_callback = audio::create_audio_callback(
        Arc::clone(&current_db),
        Arc::clone(&smoothed_db),
        Arc::clone(&display_db),
        Arc::clone(&threshold_reached),
        linear_threshold,
    );

    let stream = audio::build_audio_stream(&device, &config, audio_callback)?;

    // Start stream
    stream.play()?;

    // UI loop - very fast updates for ultra-smooth display
    let mut interval = tokio::time::interval(Duration::from_millis(10));
    loop {
        tokio::select! {
            _ = interval.tick() => {
                // Update state from shared values
                {
                    let mut state = state.lock().unwrap();
                    state.current_db = *current_db.lock().unwrap();
                    state.smoothed_db = *smoothed_db.lock().unwrap();
                    state.display_db = *display_db.lock().unwrap();
                    state.threshold_reached = *threshold_reached.lock().unwrap();
                }

                // Draw UI
                terminal.draw(|f| {
                    let state = state.lock().unwrap();
                    let ui_state = ui::UiState {
                        device_name: state.device_name.clone(),
                        current_db: state.current_db,
                        smoothed_db: state.smoothed_db,
                        display_db: state.display_db,
                        threshold_db: state.threshold_db,
                        status: state.status.clone(),
                    };
                    ui::render_ui(f, &ui_state);
                })?;

                // Check if threshold reached
                let state = state.lock().unwrap();
                if state.threshold_reached {
                    break;
                }
            }
            _ = tokio::signal::ctrl_c() => {
                break;
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Stop stream
    drop(stream);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_args() {
        let args = Args::try_parse_from(["test", "--threshold=-20", "--device", "device_name"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert_eq!(args.threshold, -20);
        assert_eq!(args.device, Some("device_name".to_string()));
    }

    #[test]
    fn test_parse_args_no_device() {
        let args = Args::try_parse_from(["test", "--threshold=-10"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert_eq!(args.threshold, -10);
        assert_eq!(args.device, None);
    }
}
