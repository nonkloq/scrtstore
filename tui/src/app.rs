use core::fmt;
use crossterm::event::{self, Event, KeyCode};

use crate::pages::{
    page::{EventResult, PageObject},
    welcome::Welcome,
};
use ratatui::{DefaultTerminal, Frame};

#[derive(Debug)]
pub enum AppError {
    AnyError(String),
    IOError(std::io::Error),
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        AppError::IOError(value)
    }
}
impl From<String> for AppError {
    fn from(value: String) -> Self {
        AppError::AnyError(value)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::AnyError(err) => write!(f, "Error occured: {err}"),
            AppError::IOError(err) => write!(f, "IO Error: {err}"),
        }
    }
}

pub struct App {
    page: PageObject,
}

impl App {
    fn render(&self, frame: &mut Frame) {
        self.page.render(frame);
    }

    fn page_consume_event(&mut self, event: Event) -> bool {
        match self.page.handle_event(event) {
            EventResult::NxtState(page) => {
                self.page = page;
                true
            }
            EventResult::Consumed => true,
            EventResult::NotConsumed => false,
        }
    }

    #[allow(unused_variables)]
    fn app_consume_special_keys(&self, event: &Event) -> bool {
        false
    }

    fn app_consume_event_and_indicate_exit(&self, event: Event) -> bool {
        if let Event::Key(e) = event {
            match e.code {
                KeyCode::Char('q') => true, // Exit the loop
                _ => false,
            }
        } else {
            false
        }
    }

    pub fn run(&mut self, term: &mut DefaultTerminal) -> Result<(), AppError> {
        loop {
            term.draw(|frame| {
                self.render(frame);
            })?;

            let event = event::read()?;

            if self.app_consume_special_keys(&event) {
                continue;
            }

            if self.page_consume_event(event.clone()) {
                continue;
            }

            if self.app_consume_event_and_indicate_exit(event) {
                break;
            }
        }
        Ok(())
    }
}

impl Default for App {
    fn default() -> Self {
        App {
            page: Welcome::new(),
        }
    }
}
