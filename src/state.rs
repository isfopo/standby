//! Application state management

use std::sync::{Arc, Mutex};

/// Type alias for audio processing shared state references
pub type AudioStateRefs = (Arc<Mutex<f32>>, Arc<Mutex<f32>>, Arc<Mutex<f32>>, Arc<Mutex<bool>>);



/// Internal application state
pub struct AppState {
    pub device_name: String,
    pub current_db: f32,
    pub smoothed_db: f32,
    pub display_db: f32,
    pub threshold_db: i32,
    pub status: String,
    pub threshold_reached: bool,
}

impl AppState {
    /// Create a new application state with default values
    pub fn new(device_name: String, threshold_db: i32) -> Self {
        Self {
            device_name: device_name.clone(),
            current_db: crate::constants::audio::MIN_DB_LEVEL,
            smoothed_db: crate::constants::audio::MIN_DB_LEVEL,
            display_db: crate::constants::audio::MIN_DB_LEVEL,
            threshold_db,
            status: format!("Monitoring {}... Press Ctrl+C or Escape to quit.", device_name),
            threshold_reached: false,
        }
    }



    /// Update state from shared audio processing values
    pub fn update_from_audio(
        &mut self,
        current_db: &Arc<Mutex<f32>>,
        smoothed_db: &Arc<Mutex<f32>>,
        display_db: &Arc<Mutex<f32>>,
        threshold_reached: &Arc<Mutex<bool>>,
    ) {
        self.current_db = *current_db.lock().unwrap();
        self.smoothed_db = *smoothed_db.lock().unwrap();
        self.display_db = *display_db.lock().unwrap();
        self.threshold_reached = *threshold_reached.lock().unwrap();
    }
}

/// Thread-safe shared state wrapper
pub struct SharedState {
    pub current_db: Arc<Mutex<f32>>,
    pub smoothed_db: Arc<Mutex<f32>>,
    pub display_db: Arc<Mutex<f32>>,
    pub threshold_reached: Arc<Mutex<bool>>,
}

impl SharedState {
    /// Create new shared state with default values
    pub fn new() -> Self {
        Self {
            current_db: Arc::new(Mutex::new(crate::constants::audio::MIN_DB_LEVEL)),
            smoothed_db: Arc::new(Mutex::new(crate::constants::audio::MIN_DB_LEVEL)),
            display_db: Arc::new(Mutex::new(crate::constants::audio::MIN_DB_LEVEL)),
            threshold_reached: Arc::new(Mutex::new(false)),
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