use core::fmt;
use gui::pages::Pages;
use gui::pages::select_store::StoresList;
use ratatui::style::Color;

use crossterm::event::{self, Event};
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::{DefaultTerminal, Frame};

struct App {
    is_exit: bool,
    is_in_background: bool,
    page: Pages,
}

#[derive(Debug)]
enum AppError {
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

impl App {
    fn new() -> Self {
        App {
            is_exit: false,
            is_in_background: false,
            page: Pages::ListStores(StoresList::default()),
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        if self.is_in_background {
            frame.render_widget(
                Block::default().style(Style::default().bg(Color::DarkGray)),
                frame.area(),
            );
        } else {
            self.page.render(frame);
        }
    }

    fn handle_events(&mut self) -> Result<(), AppError> {
        let event = event::read()?;

        if self.is_in_background {
            // Key/Mouse event triggered it to foreground
            if matches!(event, Event::Key(_) | Event::Mouse(_)) {
                self.is_in_background = false;
            }
            return Ok(());
        }

        let state = match event {
            Event::Key(_) | Event::Mouse(_) | Event::Paste(_) => self.page.handle_event(event),
            Event::Resize(_x, _y) => Ok(None),
            Event::FocusLost => {
                self.is_in_background = true;
                Ok(None)
            }
            Event::FocusGained => {
                self.is_in_background = false;
                Ok(None)
            }
        }?;

        if let Some(page) = state {
            match page {
                Pages::HideScreen => self.is_in_background = true,
                // TEMP: quit instead of a dialog
                Pages::QuitScreen => self.is_exit = true,
                _ => self.page = page,
            }
        }

        Ok(())
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), AppError> {
        while !self.is_exit {
            terminal.draw(|frame| self.render(frame))?;
            // It blocks the loop to listen for events
            self.handle_events()? // blocks (waits)
        }
        Ok(())
    }
}

// HACK: Temporary Cursor gaurd, need to modify this
struct CursorGuard;

impl CursorGuard {
    fn new() -> Self {
        print!("\x1b[5 q");
        CursorGuard
    }
}

impl Drop for CursorGuard {
    fn drop(&mut self) {
        print!("\x1b[1 q");
    }
}

fn main() {
    let _cur = CursorGuard::new();
    let _app_result = ratatui::run(|terminal| App::new().run(terminal));
}
