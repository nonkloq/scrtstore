use crate::{
    pages::{create_new_store::CreateNewStore, login::Login, page::Page},
    svc::utils::StoreMetaData,
};

pub struct SelectStore {
    stores: Vec<StoreMetaData>,
}

impl Page for SelectStore {
    fn render(&self, frame: &mut ratatui::prelude::Frame) {}
}

impl SelectStore {
    pub fn new(stores: Vec<StoreMetaData>) -> Box<Self> {
        Box::new(Self { stores })
    }
}
