use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
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

struct AppState {
    device_name: String,
    current_db: f32,
    smoothed_db: f32,
    display_db: f32, // Extra smoothing for display
    threshold_db: i32,
    status: String,
    threshold_reached: bool,
}

fn create_db_labels(width: usize, threshold_db: i32) -> Line<'static> {
    let mut spans = Vec::new();

    // Calculate threshold position (threshold_db ranges from -60 to 0)
    let threshold_ratio = ((threshold_db as f64 + 60.0) / 60.0).clamp(0.0, 1.0);
    let threshold_pos = (threshold_ratio * (width - 1) as f64).round() as usize;

    for i in 0..width {
        // Check if this position should show the threshold marker
        if i == threshold_pos {
            // Show threshold marker with bright color
            spans.push(Span::styled(
                "▲".to_string(),
                Style::default().fg(Color::White),
            ));
            continue;
        }

        // Calculate which label to show at this position
        let label = if i == 0 {
            // Always show -60 at the start
            "-60".to_string()
        } else if i == width - 1 {
            // Always show 0 at the end
            "0".to_string()
        } else if i == width / 3 {
            // Show -40 at 1/3 position
            "-40".to_string()
        } else if i == 2 * width / 3 {
            // Show -20 at 2/3 position
            "-20".to_string()
        } else {
            // No label at this position
            " ".to_string()
        };

        // Color the labels to match the bar colors at this position
        let color = if i < width / 3 {
            Color::Green
        } else if i < 2 * width / 3 {
            Color::Yellow
        } else {
            Color::Red
        };

        spans.push(Span::styled(label, Style::default().fg(color)));
    }

    Line::from(spans)
}

fn create_gradient_bar(width: usize, ratio: f64) -> Line<'static> {
    let filled = (ratio * width as f64) as usize;
    let partial_fill = (ratio * width as f64) - filled as f64;
    let mut spans = Vec::new();

    for i in 0..width {
        let color = if i < width / 3 {
            Color::Green
        } else if i < 2 * width / 3 {
            Color::Yellow
        } else {
            Color::Red
        };

        let ch = if i < filled {
            '█' // Fully filled
        } else if i == filled && partial_fill > 0.0 {
            // Partial fill characters for smoother appearance
            match (partial_fill * 8.0) as usize {
                0 => '░',
                1 => '░',
                2 => '▒',
                3 => '▒',
                4 => '▓',
                5 => '▓',
                6 => '█',
                _ => '█',
            }
        } else {
            '░' // Empty
        };
        spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
    }

    Line::from(spans)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let db_threshold = args.threshold;
    let linear_threshold = 10.0f32.powf(db_threshold as f32 / 20.0);
    let device_name_arg = args.device;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Get host
    let host = cpal::default_host();

    // Get input device
    let device = if let Some(name) = device_name_arg.clone() {
        host.input_devices()?
            .find(|d| d.name().map(|n| n == name).unwrap_or(false))
            .ok_or("Device not found")?
    } else {
        host.default_input_device()
            .ok_or("No default input device")?
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
    let state = Arc::new(Mutex::new(AppState {
        device_name: device_name.clone(),
        current_db: -60.0,
        smoothed_db: -60.0,
        display_db: -60.0,
        threshold_db: db_threshold,
        status: format!("Monitoring {}... Press Ctrl+C to quit.", device_name),
        threshold_reached: false,
    }));

    // Flag to track if currently above threshold
    let is_above = Arc::new(Mutex::new(false));

    // Build stream
    let state_clone = Arc::clone(&state);
    let state_clone_err = Arc::clone(&state);
    let is_above_clone = Arc::clone(&is_above);
    let linear_threshold_clone = linear_threshold;
    let _db_threshold_clone = db_threshold;
    let stream = device.build_input_stream(
        &config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let max_sample = data.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
            let current_db = if max_sample > 0.0 {
                20.0 * max_sample.log10()
            } else {
                -60.0
            };

            let mut state = state_clone.lock().unwrap();
            state.current_db = current_db;

            // Two-stage smoothing for ultra-smooth display
            // First stage: moderate smoothing of raw audio data
            const AUDIO_SMOOTHING: f32 = 0.4;
            state.smoothed_db =
                state.smoothed_db * (1.0 - AUDIO_SMOOTHING) + current_db * AUDIO_SMOOTHING;

            // Second stage: heavy smoothing for display (easing effect)
            const DISPLAY_SMOOTHING: f32 = 0.15;
            state.display_db = state.display_db * (1.0 - DISPLAY_SMOOTHING)
                + state.smoothed_db * DISPLAY_SMOOTHING;

            let mut is_above = is_above_clone.lock().unwrap();
            if max_sample > linear_threshold_clone {
                if !*is_above {
                    state.threshold_reached = true;
                    *is_above = true;
                }
            } else if *is_above {
                *is_above = false;
            }
        },
        move |err| {
            let mut state = state_clone_err.lock().unwrap();
            state.status = format!("Error: {}", err);
        },
        None,
    )?;

    // Start stream
    stream.play()?;

    // UI loop - very fast updates for ultra-smooth display
    let mut interval = tokio::time::interval(Duration::from_millis(10));
    loop {
        tokio::select! {
            _ = interval.tick() => {
                // Draw UI
                terminal.draw(|f| {
                    let state = state.lock().unwrap();
                    let size = f.size();

                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(3),
                            Constraint::Length(3),
                            Constraint::Length(2),
                            Constraint::Min(1),
                        ])
                        .split(size);

                    // Device and status
                    let device_block = Block::default()
                        .title("Device")
                        .borders(Borders::ALL);
                    let device_text = Paragraph::new(state.device_name.as_str())
                        .block(device_block);
                    f.render_widget(device_text, chunks[0]);

                    // Status
                    let status_block = Block::default()
                        .title("Status")
                        .borders(Borders::ALL);
                    let status_text = Paragraph::new(state.status.as_str())
                        .block(status_block);
                    f.render_widget(status_text, chunks[1]);

                    // Threshold indicator
                    let width = chunks[2].width as usize;
                    let threshold_pos = (((state.threshold_db as f32 + 60.0) / 60.0).clamp(0.0, 1.0) * (width - 2) as f32) as usize;
                    let mut bar = String::new();
                    for i in 0..(width - 2) {
                        if i == threshold_pos {
                            bar.push('|');
                        } else {
                            bar.push('─');
                        }
                    }
                    let threshold_text = Paragraph::new(format!("Threshold: {} dB\n{}", state.threshold_db, bar));
                    f.render_widget(threshold_text, chunks[2]);

                    // dB bar with labels
                    let db_ratio = ((state.display_db + 60.0) / 60.0).clamp(0.0, 1.0) as f64;
                    let bar_width = (chunks[3].width - 2) as usize; // account for borders
                    let bar_line = create_gradient_bar(bar_width, db_ratio);
                    let label_line = create_db_labels(bar_width, state.threshold_db);
                    let gauge = Paragraph::new(vec![bar_line, label_line])
                        .block(Block::default().title(format!("Current dB: {:.1} (Raw: {:.1})", state.display_db, state.current_db)).borders(Borders::ALL));
                    f.render_widget(gauge, chunks[3]);
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
