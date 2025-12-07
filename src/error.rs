//! Custom error types for the application

use std::fmt;

/// Application-specific error type
#[derive(Debug)]
pub enum AppError {
    /// Audio device related errors
    AudioDevice(String),
    /// Audio stream related errors
    AudioStream(String),

    /// General I/O errors
    Io(std::io::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::AudioDevice(msg) => write!(f, "Audio device error: {}", msg),
            AppError::AudioStream(msg) => write!(f, "Audio stream error: {}", msg),
            AppError::Io(err) => write!(f, "I/O error: {}", err),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<cpal::DevicesError> for AppError {
    fn from(err: cpal::DevicesError) -> Self {
        AppError::AudioDevice(format!("Failed to enumerate devices: {}", err))
    }
}

impl From<cpal::DeviceNameError> for AppError {
    fn from(err: cpal::DeviceNameError) -> Self {
        AppError::AudioDevice(format!("Failed to get device name: {}", err))
    }
}

impl From<cpal::DefaultStreamConfigError> for AppError {
    fn from(err: cpal::DefaultStreamConfigError) -> Self {
        AppError::AudioDevice(format!("Failed to get default stream config: {}", err))
    }
}

impl From<cpal::SupportedStreamConfigsError> for AppError {
    fn from(err: cpal::SupportedStreamConfigsError) -> Self {
        AppError::AudioDevice(format!("Failed to get supported stream configs: {}", err))
    }
}

impl From<cpal::BuildStreamError> for AppError {
    fn from(err: cpal::BuildStreamError) -> Self {
        AppError::AudioStream(format!("Failed to build audio stream: {}", err))
    }
}

impl From<cpal::PlayStreamError> for AppError {
    fn from(err: cpal::PlayStreamError) -> Self {
        AppError::AudioStream(format!("Failed to play audio stream: {}", err))
    }
}

/// Result type alias for application operations
pub type AppResult<T> = Result<T, AppError>;
