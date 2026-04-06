use ratatui::Frame;
use scrt::ppgen::generate_pass_phrase;
use std::path::PathBuf;

pub struct NewStore {
    password: String,
    passphrase: String,
    filename: String,
    folder: Option<PathBuf>,

    // 0..=3 indicating the store field
    idx_pos: usize,
}

impl NewStore {
    pub fn render(&mut self, frame: &mut Frame) {}
}

impl Default for NewStore {
    fn default() -> Self {
        Self {
            folder: None,
            password: "".to_string(),
            passphrase: generate_pass_phrase(32),
            filename: "passwords".to_string(),
            idx_pos: 0,
        }
    }
}
