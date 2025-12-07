//! Main application logic and orchestration

use crate::audio;
use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::state::{AppState, SharedState};
use crate::ui;
use cpal::traits::StreamTrait;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
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
    UserExit = 1, // User pressed Escape or Ctrl+C
    Error = 2,    // Actual application error
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
    pub fn new_with_config(config: Config) -> AppResult<Self> {
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
        let (device, audio_config) =
            match audio::setup_audio_device(self.config.device_name.clone(), &self.config.channels)
            {
                Ok(result) => result,
                Err(e) => {
                    return RunResult {
                        result: Err(e),
                        exit_code: ExitCode::Error,
                    };
                }
            };
        let device_name = audio_config.device_name;

        // Create shared state
        let shared_state = SharedState::new(self.config.channels.len());
        let (current_db, smoothed_db, display_db, threshold_reached) = shared_state.audio_refs();

        // Create app state
        let mut app_state = AppState::new(
            device_name,
            self.config.threshold_db,
            self.config.channels.len(),
        );

        // Build audio stream
        let audio_callback = audio::create_audio_callback(
            current_db,
            smoothed_db,
            display_db,
            threshold_reached,
            self.config.linear_threshold(),
            &audio_config.selected_channels,
            audio_config.channels as usize,
        );

        let config = cpal::StreamConfig {
            channels: audio_config.channels,
            sample_rate: cpal::SampleRate(audio_config.sample_rate),
            buffer_size: crate::constants::audio::BUFFER_SIZE,
        };

        let stream = match audio::build_audio_stream(&device, &config, audio_callback) {
            Ok(stream) => stream,
            Err(e) => {
                return RunResult {
                    result: Err(e),
                    exit_code: ExitCode::Error,
                };
            }
        };

        if let Err(e) = stream.play() {
            return RunResult {
                result: Err(e.into()),
                exit_code: ExitCode::Error,
            };
        }

        // Main UI loop
        let mut interval = tokio::time::interval(Duration::from_millis(
            crate::constants::ui::UPDATE_INTERVAL_MS,
        ));
        let mut exit_reason = ExitCode::Success;

        loop {
            // Update state from shared values
            app_state.update_from_audio(
                &shared_state.current_db,
                &shared_state.smoothed_db,
                &shared_state.display_db,
                &shared_state.threshold_reached,
            );

            // Render UI
            if let Err(e) = self.terminal.draw(|f| {
                let ui_state = ui::UiState {
                    device_name: app_state.device_name.clone(),
                    current_db: app_state.current_db.clone(),
                    display_db: app_state.display_db.clone(),
                    threshold_db: app_state.threshold_db,
                    min_db: self.config.min_db,
                    status: app_state.status.clone(),
                };
                ui::render_ui(f, &ui_state);
            }) {
                return RunResult {
                    result: Err(e.into()),
                    exit_code: ExitCode::Error,
                };
            }

            // Check if threshold reached on any channel
            if app_state.threshold_reached.iter().any(|&r| r) {
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
            if !should_exit
                && crossterm::event::poll(Duration::from_millis(0)).unwrap_or(false)
                && let Ok(Event::Key(key_event)) = crossterm::event::read()
            {
                match key_event.code {
                    KeyCode::Esc => {
                        should_exit = true;
                        exit_reason = ExitCode::UserExit;
                    }
                    KeyCode::Char('c')
                        if key_event
                            .modifiers
                            .contains(crossterm::event::KeyModifiers::CONTROL) =>
                    {
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

    /// Run max monitoring mode
    pub async fn run_max(&mut self, duration: Option<f32>) -> Result<Vec<f32>, AppError> {
        // Setup audio
        let (device, audio_config) =
            match audio::setup_audio_device(self.config.device_name.clone(), &self.config.channels)
            {
                Ok(result) => result,
                Err(e) => return Err(e),
            };
        let device_name = audio_config.device_name;

        // Create shared state
        let shared_state = SharedState::new(self.config.channels.len());
        let (current_db, smoothed_db, display_db, threshold_reached) = shared_state.audio_refs();

        // Create app state with max tracking
        let mut app_state = AppState::new(
            device_name,
            self.config.threshold_db,
            self.config.channels.len(),
        );

        // Build audio stream
        let audio_callback = audio::create_audio_callback(
            current_db,
            smoothed_db,
            display_db,
            threshold_reached,
            self.config.linear_threshold(),
            &audio_config.selected_channels,
            audio_config.channels as usize,
        );

        let config = cpal::StreamConfig {
            channels: audio_config.channels,
            sample_rate: cpal::SampleRate(audio_config.sample_rate),
            buffer_size: crate::constants::audio::BUFFER_SIZE,
        };

        let stream = match audio::build_audio_stream(&device, &config, audio_callback) {
            Ok(stream) => stream,
            Err(e) => return Err(e),
        };

        if let Err(e) = stream.play() {
            return Err(e.into());
        }

        // Main UI loop with timeout
        let mut interval = tokio::time::interval(Duration::from_millis(
            crate::constants::ui::UPDATE_INTERVAL_MS,
        ));
        let start_time = tokio::time::Instant::now();
        let mut max_levels =
            vec![crate::constants::audio::MIN_DB_LEVEL as f32; self.config.channels.len()];

        loop {
            // Update state from shared values
            app_state.update_from_audio(
                &shared_state.current_db,
                &shared_state.smoothed_db,
                &shared_state.display_db,
                &shared_state.threshold_reached,
            );

            // Update max levels
            for (i, &current) in app_state.current_db.iter().enumerate() {
                if current > max_levels[i] {
                    max_levels[i] = current;
                }
            }

            // Render UI
            if let Err(e) = self.terminal.draw(|f| {
                let ui_state = ui::UiState {
                    device_name: app_state.device_name.clone(),
                    current_db: app_state.current_db.clone(),
                    display_db: app_state.display_db.clone(),
                    threshold_db: app_state.threshold_db,
                    min_db: self.config.min_db,
                    status: app_state.status.clone(),
                };
                ui::render_ui(f, &ui_state);
            }) {
                return Err(e.into());
            }

            // Check for timeout
            if let Some(dur) = duration {
                if start_time.elapsed() >= Duration::from_secs_f32(dur) {
                    break;
                }
            }

            // Check for keyboard events
            if crossterm::event::poll(Duration::from_millis(0)).unwrap_or(false) {
                if let Ok(Event::Key(key_event)) = crossterm::event::read() {
                    match key_event.code {
                        KeyCode::Enter => break,
                        KeyCode::Char('c')
                            if key_event
                                .modifiers
                                .contains(crossterm::event::KeyModifiers::CONTROL) =>
                        {
                            break;
                        }
                        _ => {}
                    }
                }
            }

            // Wait for next interval
            interval.tick().await;
        }

        // Cleanup
        drop(stream);
        let _ = self.cleanup();

        // Return max levels as JSON or something? Wait, the user said "return the max decibel level", but in code, we print them.

        // Actually, since it's a command, we can return the levels, but in main, we print them.

        // But to return, perhaps change to return Vec<f32>

        Ok(max_levels)
    }

    /// Run average monitoring mode
    pub async fn run_average(&mut self, duration: Option<f32>) -> Result<Vec<f32>, AppError> {
        // Setup audio
        let (device, audio_config) =
            match audio::setup_audio_device(self.config.device_name.clone(), &self.config.channels)
            {
                Ok(result) => result,
                Err(e) => return Err(e),
            };
        let device_name = audio_config.device_name;

        // Create shared state
        let shared_state = SharedState::new(self.config.channels.len());
        let (current_db, smoothed_db, display_db, threshold_reached) = shared_state.audio_refs();

        // Create app state with average tracking
        let mut app_state = AppState::new(
            device_name,
            self.config.threshold_db,
            self.config.channels.len(),
        );

        // Build audio stream
        let audio_callback = audio::create_audio_callback(
            current_db,
            smoothed_db,
            display_db,
            threshold_reached,
            self.config.linear_threshold(),
            &audio_config.selected_channels,
            audio_config.channels as usize,
        );

        let config = cpal::StreamConfig {
            channels: audio_config.channels,
            sample_rate: cpal::SampleRate(audio_config.sample_rate),
            buffer_size: crate::constants::audio::BUFFER_SIZE,
        };

        let stream = match audio::build_audio_stream(&device, &config, audio_callback) {
            Ok(stream) => stream,
            Err(e) => return Err(e),
        };

        if let Err(e) = stream.play() {
            return Err(e.into());
        }

        // Main UI loop with timeout
        let mut interval = tokio::time::interval(Duration::from_millis(
            crate::constants::ui::UPDATE_INTERVAL_MS,
        ));
        let start_time = tokio::time::Instant::now();
        let mut sums: Vec<f32> = vec![0.0; self.config.channels.len()];
        let mut counts: Vec<u32> = vec![0; self.config.channels.len()];

        loop {
            // Update state from shared values
            app_state.update_from_audio(
                &shared_state.current_db,
                &shared_state.smoothed_db,
                &shared_state.display_db,
                &shared_state.threshold_reached,
            );

            // Accumulate for average
            for (i, &current) in app_state.current_db.iter().enumerate() {
                sums[i] += current;
                counts[i] += 1;
            }

            // Render UI
            if let Err(e) = self.terminal.draw(|f| {
                let ui_state = ui::UiState {
                    device_name: app_state.device_name.clone(),
                    current_db: app_state.current_db.clone(),
                    display_db: app_state.display_db.clone(),
                    threshold_db: app_state.threshold_db,
                    min_db: self.config.min_db,
                    status: app_state.status.clone(),
                };
                ui::render_ui(f, &ui_state);
            }) {
                return Err(e.into());
            }

            // Check for timeout
            if let Some(dur) = duration {
                if start_time.elapsed() >= Duration::from_secs_f32(dur) {
                    break;
                }
            }

            // Check for keyboard events
            if crossterm::event::poll(Duration::from_millis(0)).unwrap_or(false) {
                if let Ok(Event::Key(key_event)) = crossterm::event::read() {
                    match key_event.code {
                        KeyCode::Enter => break,
                        KeyCode::Char('c')
                            if key_event
                                .modifiers
                                .contains(crossterm::event::KeyModifiers::CONTROL) =>
                        {
                            break;
                        }
                        _ => {}
                    }
                }
            }

            // Wait for next interval
            interval.tick().await;
        }

        // Cleanup
        drop(stream);
        let _ = self.cleanup();

        // Calculate averages
        let mut averages = Vec::new();
        for (i, &sum) in sums.iter().enumerate() {
            let avg = if counts[i] > 0 {
                sum / counts[i] as f32
            } else {
                0.0
            };
            averages.push(avg);
        }

        Ok(averages)
    }

    /// Clean up terminal state
    fn cleanup(&mut self) -> AppResult<()> {
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
