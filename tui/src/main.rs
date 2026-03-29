use core::fmt;
use gui::components::{input_widget::InputWidget, text_box::TextBox};

use crossterm::event::{self, Event, KeyCode};
use ratatui::{DefaultTerminal, Frame};

struct App {
    is_exit: bool,
    is_in_background: bool,
    text_box: TextBox,
}

#[derive(Debug)]
enum AppError {
    // AnyError(String),
    IOError(std::io::Error),
}
impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        AppError::IOError(value)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // AppError::AnyError(err) => write!(f, "Error occured: {err}"),
            AppError::IOError(err) => write!(f, "IO Error: {err}"),
        }
    }
}

impl App {
    fn new() -> Self {
        App {
            is_exit: false,
            is_in_background: false,
            text_box: TextBox::new(String::from("Input Box")),
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();
        self.text_box.render_widget(frame, area);
        self.text_box.set_cursor(frame, area);
    }

    fn handle_events(&mut self) -> Result<(), AppError> {
        match event::read()? {
            Event::Key(k) => {
                if k.code == KeyCode::Esc {
                    self.is_exit = true;
                } else {
                    self.text_box.handle_key(k);
                }
            }
            Event::Mouse(_m) => (),
            Event::Paste(_content) => (),
            Event::Resize(_x, _y) => (),
            Event::FocusLost => self.is_in_background = true,
            Event::FocusGained => self.is_in_background = false,
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
