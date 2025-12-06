//! UI rendering and layout utilities

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Application state for UI rendering
#[derive(Clone)]
pub struct UiState {
    pub device_name: String,
    pub current_db: f32,
    pub display_db: f32,
    pub threshold_db: i32,
    pub status: String,
}

/// Create a gradient bar showing audio levels
pub fn create_gradient_bar(width: usize, ratio: f64) -> Line<'static> {
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

/// Create dB level labels with threshold indicator
pub fn create_db_labels(width: usize, threshold_db: i32) -> Line<'static> {
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

/// Render the complete UI
pub fn render_ui(f: &mut Frame, state: &UiState) {
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
    let device_block = Block::default().title("Device").borders(Borders::ALL);
    let device_text = Paragraph::new(state.device_name.as_str()).block(device_block);
    f.render_widget(device_text, chunks[0]);

    // Status
    let status_block = Block::default().title("Status").borders(Borders::ALL);
    let status_text = Paragraph::new(state.status.as_str()).block(status_block);
    f.render_widget(status_text, chunks[1]);

    // Threshold indicator
    let width = chunks[2].width as usize;
    let threshold_pos =
        (((state.threshold_db as f32 + 60.0) / 60.0).clamp(0.0, 1.0) * (width - 2) as f32) as usize;
    let mut bar = String::new();
    for i in 0..(width - 2) {
        bar.push('─');
    }

    let threshold_text = Paragraph::new(format!("Threshold: {} dB\n{}", state.threshold_db, bar));
    f.render_widget(threshold_text, chunks[2]);

    // dB bar with labels
    let min_db = crate::constants::audio::MIN_DB_LEVEL;
    let db_range = -min_db; // Range from MIN_DB_LEVEL to 0
    let db_ratio = ((state.display_db - min_db) / db_range).clamp(0.0, 1.0) as f64;
    let bar_width =
        (chunks[3].width as usize).saturating_sub(crate::constants::ui::BAR_BORDER_WIDTH);
    let bar_line = create_gradient_bar(bar_width, db_ratio);
    let label_line = create_db_labels(bar_width, state.threshold_db);
    let gauge = Paragraph::new(vec![bar_line, label_line]).block(
        Block::default()
            .title(format!(
                "Current dB: {:.1} (Raw: {:.1})",
                state.display_db, state.current_db
            ))
            .borders(Borders::ALL),
    );
    f.render_widget(gauge, chunks[3]);
}
