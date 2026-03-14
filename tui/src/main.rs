use std::path::{Path, PathBuf};
use std::str::FromStr;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::terminal;
use ratatui::style::Stylize;
use ratatui::widgets::{Block, Paragraph, Widget};
use ratatui::{DefaultTerminal, widgets};
use scrt::auth::twofa::TwoFAMethod;
use scrt::{ScrtStore, Vault};

const SCRT_PATH: &str = "./my.scrt";

struct Button {
    label: String,
    is_active: bool,
}
impl Button {
    fn new(name: &str) -> Button {
        Button {
            label: String::from(name),
            is_active: true,
        }
    }
}

impl Widget for Button {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let mut block = widgets::Block::bordered().title(self.label);
        if self.is_active {
            block = block.green();
        } else {
            block = block.gray();
        }
        block.render(area, buf);
    }
}

struct LoginPage {
    store_path: Option<PathBuf>,
    password: Option<String>,
    verif_method: TwoFAMethod,
    verif_data: Option<String>,
    exit: bool,
}

impl LoginPage {
    fn new(scrt_path: Option<&str>) -> LoginPage {
        LoginPage {
            store_path: scrt_path.map(PathBuf::from),
            password: None,
            verif_method: TwoFAMethod::PassPhrase,
            verif_data: None,
            exit: false,
        }
    }
    fn handle_events(&mut self) -> std::io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }
    fn exit(&mut self) {
        self.exit = true;
    }
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), Box<dyn std::error::Error>> {
        while !self.exit {
            terminal.draw(|frame| frame.render_widget(Button::new("Ligma"), frame.area()))?;
            self.handle_events()?;
        }
        Ok(())
    }
}

fn main() {
    let _vault = ratatui::run(|terminal| LoginPage::new(Some(SCRT_PATH)).run(terminal));
}
