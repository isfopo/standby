//! Main application logic and orchestration

use crate::audio;
use crate::config::Config;
use cpal::traits::StreamTrait;
use crate::error::{AppError, AppResult};
use crate::state::{AppState, SharedState};
use crate::ui;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;

/// Main application struct
pub struct App {
    config: Config,
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
}

/// Exit codes for the application
#[derive(Debug, Clone, Copy)]
pub enum ExitCode {
    Success = 0,
    UserExit = 1,  // User pressed Escape or Ctrl+C
    Error = 2,     // Actual application error
}

/// Result type that includes user exit information
pub type AppRunResult = Result<(), AppError>;

/// Extended result that tracks exit reason
pub struct RunResult {
    pub result: AppRunResult,
    pub exit_code: ExitCode,
}

impl App {
    /// Initialize the application with configuration
    pub fn new() -> AppResult<Self> {
        let config = Config::from_args().map_err(|e| AppError::Config(e.to_string()))?;

        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(App { config, terminal })
    }

    /// Run the main application loop
    pub async fn run(mut self) -> RunResult {
        // Setup audio
        let (device, audio_config) = match audio::setup_audio_device(self.config.device_name.clone()) {
            Ok(result) => result,
            Err(e) => return RunResult {
                result: Err(e),
                exit_code: ExitCode::Error,
            },
        };
        let device_name = audio_config.device_name;

        // Create shared state
        let shared_state = SharedState::new();
        let (current_db, smoothed_db, display_db, threshold_reached) = shared_state.audio_refs();

        // Create app state
        let mut app_state = AppState::new(device_name, self.config.threshold_db);

        // Build audio stream
        let audio_callback = audio::create_audio_callback(
            current_db,
            smoothed_db,
            display_db,
            threshold_reached,
            self.config.linear_threshold(),
        );

        let config = cpal::StreamConfig {
            channels: audio_config.channels,
            sample_rate: cpal::SampleRate(audio_config.sample_rate),
            buffer_size: crate::constants::audio::BUFFER_SIZE,
        };

        let stream = match audio::build_audio_stream(&device, &config, audio_callback) {
            Ok(stream) => stream,
            Err(e) => return RunResult {
                result: Err(e),
                exit_code: ExitCode::Error,
            },
        };

        if let Err(e) = stream.play() {
            return RunResult {
                result: Err(e.into()),
                exit_code: ExitCode::Error,
            };
        }

        // Main UI loop
        let mut interval = tokio::time::interval(Duration::from_millis(crate::constants::ui::UPDATE_INTERVAL_MS));
        let mut exit_reason = ExitCode::Success;

        loop {
            // Update state from shared values
            app_state.update_from_audio(&shared_state.current_db, &shared_state.smoothed_db, &shared_state.display_db, &shared_state.threshold_reached);

            // Render UI
            if let Err(e) = self.terminal.draw(|f| {
                let ui_state = ui::UiState {
                    device_name: app_state.device_name.clone(),
                    current_db: app_state.current_db,
                    display_db: app_state.display_db,
                    threshold_db: app_state.threshold_db,
                    status: app_state.status.clone(),
                };
                ui::render_ui(f, &ui_state);
            }) {
                return RunResult {
                    result: Err(e.into()),
                    exit_code: ExitCode::Error,
                };
            }

            // Check if threshold reached
            if app_state.threshold_reached {
                exit_reason = ExitCode::Success;
                break;
            }

            // Check for keyboard events and signals
            let mut should_exit = false;

            // Check for Ctrl+C signal
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    should_exit = true;
                    exit_reason = ExitCode::UserExit;
                }
                _ = tokio::time::sleep(Duration::from_millis(1)) => {
                    // Timeout - check for keyboard events
                }
            }

            // Check for keyboard events (Escape to quit)
            if !should_exit && crossterm::event::poll(Duration::from_millis(0)).unwrap_or(false)
                && let Ok(Event::Key(key_event)) = crossterm::event::read() {
                match key_event.code {
                    KeyCode::Esc => {
                        should_exit = true;
                        exit_reason = ExitCode::UserExit;
                    }
                    KeyCode::Char('c') if key_event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                        should_exit = true;
                        exit_reason = ExitCode::UserExit;
                    }
                    _ => {}
                }
            }

            if should_exit {
                break;
            }

            // Wait for next interval
            interval.tick().await;
        }

        // Cleanup - ensure graceful exit
        drop(stream);
        let _ = self.cleanup(); // Ignore cleanup errors

        RunResult {
            result: Ok(()),
            exit_code: exit_reason,
        }
    }

    /// Clean up terminal state
    fn cleanup(mut self) -> AppResult<()> {
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}