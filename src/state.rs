//! Application state management

use std::sync::{Arc, Mutex};

/// Type alias for audio processing shared state references
pub type AudioStateRefs = (
    Arc<Mutex<Vec<f32>>>,
    Arc<Mutex<Vec<f32>>>,
    Arc<Mutex<Vec<f32>>>,
    Arc<Mutex<Vec<bool>>>,
);

/// Internal application state
pub struct AppState {
    pub device_name: String,
    pub current_db: Vec<f32>,
    pub smoothed_db: Vec<f32>,
    pub display_db: Vec<f32>,
    pub threshold_db: i32,
    pub status: String,
    pub threshold_reached: Vec<bool>,
}

impl AppState {
    /// Create a new application state with default values
    pub fn new(device_name: String, threshold_db: i32, num_channels: usize) -> Self {
        let default_db = crate::constants::audio::MIN_DB_LEVEL;
        Self {
            device_name: device_name.clone(),
            current_db: vec![default_db; num_channels],
            smoothed_db: vec![default_db; num_channels],
            display_db: vec![default_db; num_channels],
            threshold_db,
            status: format!(
                "Monitoring {}... Press Ctrl+C or Escape to quit.",
                device_name
            ),
            threshold_reached: vec![false; num_channels],
        }
    }

    /// Update state from shared audio processing values
    pub fn update_from_audio(
        &mut self,
        current_db: &Arc<Mutex<Vec<f32>>>,
        smoothed_db: &Arc<Mutex<Vec<f32>>>,
        display_db: &Arc<Mutex<Vec<f32>>>,
        threshold_reached: &Arc<Mutex<Vec<bool>>>,
    ) {
        self.current_db = current_db.lock().unwrap().clone();
        self.smoothed_db = smoothed_db.lock().unwrap().clone();
        self.display_db = display_db.lock().unwrap().clone();
        self.threshold_reached = threshold_reached.lock().unwrap().clone();
    }
}

/// Thread-safe shared state wrapper
pub struct SharedState {
    pub current_db: Arc<Mutex<Vec<f32>>>,
    pub smoothed_db: Arc<Mutex<Vec<f32>>>,
    pub display_db: Arc<Mutex<Vec<f32>>>,
    pub threshold_reached: Arc<Mutex<Vec<bool>>>,
}

impl SharedState {
    /// Create new shared state with default values
    pub fn new(num_channels: usize) -> Self {
        let default_db = crate::constants::audio::MIN_DB_LEVEL;
        Self {
            current_db: Arc::new(Mutex::new(vec![default_db; num_channels])),
            smoothed_db: Arc::new(Mutex::new(vec![default_db; num_channels])),
            display_db: Arc::new(Mutex::new(vec![default_db; num_channels])),
            threshold_reached: Arc::new(Mutex::new(vec![false; num_channels])),
        }
    }

    /// Get clones of all shared state references for audio processing
    pub fn audio_refs(&self) -> AudioStateRefs {
        (
            Arc::clone(&self.current_db),
            Arc::clone(&self.smoothed_db),
            Arc::clone(&self.display_db),
            Arc::clone(&self.threshold_reached),
        )
    }
}
