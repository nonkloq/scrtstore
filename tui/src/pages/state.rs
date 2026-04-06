use crossterm::event::{Event, KeyCode, KeyEvent, MouseEvent};
use ratatui::Frame;

use crate::pages::select_store::StoresList;

pub enum Pages {
    ListStores(StoresList),
    CreateNew,
    Login(String),
    Vault,
    HideScreen,
    QuitScreen,
}

impl Pages {
    pub fn handle_event(&mut self, event: Event) -> Result<Option<Pages>, String> {
        match event {
            Event::Key(key) => self.key_handler(key),
            Event::Mouse(click) => self.mouse_handler(click),
            Event::Paste(data) => self.paste_content(data),
            _ => Err(String::from("This event shouldn't be handled in page!")),
        }
    }

    fn key_handler(&mut self, key: KeyEvent) -> Result<Option<Pages>, String> {
        if key.code == KeyCode::Esc {
            // (Self != Vault or Vault.is_not_focus())
            return Ok(Some(Self::HideScreen));
        }

        if key.code == KeyCode::Char('q') {
            // (Self != Vault or Vault.is_not_focus())
            return Ok(Some(Self::QuitScreen));
        }

        let page = match self {
            Self::ListStores(list) => list.handle_key(key.code),
            _ => None,
        };

        Ok(page)
    }

    fn mouse_handler(&mut self, click: MouseEvent) -> Result<Option<Pages>, String> {
        let page = match self {
            Self::ListStores(list) => list.handle_mouse(click),
            _ => None,
        };

        Ok(page)
    }

    fn paste_content(&mut self, data: String) -> Result<Option<Pages>, String> {
        todo!()
    }

    pub fn render(&mut self, frame: &mut Frame) {
        match self {
            Self::ListStores(list) => list.render(frame),
            _ => {}
        }
    }
}
