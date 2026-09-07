use crate::pages::{page::Page, select_store::SelectStore, vault_page::VaultPage};

pub struct Login {}

impl Page for Login {
    fn render(&self, frame: &mut ratatui::prelude::Frame) {}
}

impl Login {
    pub fn new() -> Box<Self> {
        Box::new(Login {})
    }
}
