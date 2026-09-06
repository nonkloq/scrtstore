use crate::pages::{page::Page, select_store::SelectStore};

pub struct VaultPage {}

impl Page for VaultPage {
    fn render(&self, frame: &mut ratatui::prelude::Frame) {}
}
