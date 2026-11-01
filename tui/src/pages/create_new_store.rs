use crate::components::button::render_button;
use crate::components::file_picker::FilePicker;
use crate::pages::page::{EventResult, Page};
use ratatree::PickerMode;
use ratatui::layout::{Constraint, Layout};
use scrt::ppgen::generate_pass_phrase;
use std::env;

use crate::components::input_field::InputField;

use crossterm::event::{Event, KeyCode, KeyEvent};

#[derive(PartialEq)]
enum Pf {
    Path,
    Name,
    Pass,
    Phrase,
    GenPhrase,
    Submit,

    None,
}
pub struct CreateNewStore<'a> {
    path: FilePicker<'a>,
    store_name: InputField<'a>,
    password: InputField<'a>,
    passphrase: InputField<'a>,

    selected_field: Pf,
}

impl<'a> Page for CreateNewStore<'a> {
    fn render(&self, frame: &mut ratatui::prelude::Frame) {
        let area = frame.area();

        let [_head_padding, field_area, _footer_area] = Layout::vertical([
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ])
        .areas(area);

        let [_, field_area, _] = Layout::horizontal([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .areas(field_area);

        let [_, path_name, password, passphrase_gen, submit, _] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .areas(field_area);

        let [path, store_name] =
            Layout::horizontal([Constraint::Percentage(60), Constraint::Percentage(40)])
                .areas(path_name);
        let [passp, genpp] =
            Layout::horizontal([Constraint::Percentage(80), Constraint::Percentage(20)])
                .areas(passphrase_gen);

        render_button(
            "Create Store  ",
            submit,
            frame,
            self.selected_field.is(Pf::Submit),
        );
        render_button(
            "Generate  ",
            genpp,
            frame,
            self.selected_field.is(Pf::GenPhrase),
        );

        self.store_name
            .render_widget(store_name, frame, self.selected_field.is(Pf::Name));
        self.password
            .render_widget(password, frame, self.selected_field.is(Pf::Pass));
        self.passphrase
            .render_widget(passp, frame, self.selected_field.is(Pf::Phrase));

        // Render path widget last to fully overwrite the frame
        self.path
            .render_widget(path, frame, self.selected_field.is(Pf::Path));
    }

    fn handle_event(&mut self, event: crossterm::event::Event) -> EventResult {
        match event {
            Event::Key(key) => self.handle_key(key),
            Event::Paste(val) => self.handle_paste(val),
            _ => EventResult::NotConsumed,
        }
    }
}

impl Pf {
    fn is(&self, rhs: Self) -> bool {
        *self == rhs
    }
    fn nxt(&self) -> Self {
        match self {
            Pf::Path => Pf::Name,
            Pf::Name => Pf::Pass,
            Pf::Pass => Pf::Phrase,
            Pf::Phrase => Pf::GenPhrase,
            Pf::GenPhrase => Pf::Submit,
            Pf::Submit => Pf::Submit,

            Pf::None => Pf::Path,
        }
    }

    fn prv(&self) -> Self {
        match self {
            Pf::Path => Pf::Path,
            Pf::Name => Pf::Path,
            Pf::Pass => Pf::Name,
            Pf::Phrase => Pf::Pass,
            Pf::GenPhrase => Pf::Phrase,
            Pf::Submit => Pf::GenPhrase,

            Pf::None => Pf::Submit,
        }
    }
}
impl<'a> CreateNewStore<'a> {
    pub fn new() -> Box<Self> {
        let pwd = env::current_dir().unwrap();

        let path = FilePicker::new("Folder", pwd, PickerMode::DirsOnly);
        let password = InputField::new("Password", None, Some("Very secure password"));
        let store_name = InputField::new("Store Name", None, Some("MyScrtStore"));
        let passphrase = InputField::new("Pass Phrase", None, Some("foo bar car ..."));

        Box::new(Self {
            path,
            password,
            store_name,
            passphrase,
            selected_field: Pf::Name,
        })
    }

    fn handle_key(&mut self, key: KeyEvent) -> EventResult {
        let field_consume = match self.selected_field {
            Pf::Path => self.path.handle_event(key),
            Pf::Name => self.store_name.handle_event(key),
            Pf::Pass => self.password.handle_event(key),
            Pf::Phrase => self.passphrase.handle_event(key),

            _ => false,
        };

        if field_consume {
            return EventResult::Consumed;
        }

        match key.code {
            KeyCode::Down | KeyCode::Tab => {
                self.selected_field = self.selected_field.nxt();
                EventResult::Consumed
            }
            KeyCode::Up | KeyCode::BackTab => {
                self.selected_field = self.selected_field.prv();
                EventResult::Consumed
            }

            KeyCode::Enter if self.selected_field.is(Pf::GenPhrase) => {
                self.passphrase.set_content(generate_pass_phrase(12));
                EventResult::Consumed
            }

            KeyCode::Enter if self.selected_field.is(Pf::Submit) => {
                todo!()
            }

            KeyCode::Esc => {
                self.selected_field = Pf::None;
                EventResult::Consumed
            }

            _ => EventResult::NotConsumed,
        }
    }

    fn handle_paste(&mut self, val: String) -> EventResult {
        let state = match self.selected_field {
            Pf::Name => self.store_name.handle_paste(val),
            Pf::Pass => self.password.handle_paste(val),
            Pf::Phrase => self.passphrase.handle_paste(val),
            _ => false,
        };

        if state {
            EventResult::Consumed
        } else {
            EventResult::NotConsumed
        }
    }
}
