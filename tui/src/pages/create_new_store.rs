use crate::pages::page::{EventResult, Page};

pub struct CreateNewStore {}

impl Page for CreateNewStore {
    fn render(&self, frame: &mut ratatui::prelude::Frame) {}
}
