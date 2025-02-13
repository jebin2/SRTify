use serde::{Serialize, Deserialize};

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct AppState {
    pub is_generating: bool,
    pub model: String,
    pub media_file: String,
    pub output_dir: String,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            is_generating: false,
            model: String::new(),
            media_file: String::new(),
            output_dir: String::new(),
        }
    }
}