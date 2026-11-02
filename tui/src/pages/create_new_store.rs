use crate::components::button::render_button;
use crate::components::file_picker::FilePicker;
use crate::pages::login::Login;
use crate::pages::page::{EventResult, Page};
use crate::svc::utils::{StoreMetaData, get_known_stores, get_project_dirs};
use ratatree::PickerMode;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Stylize};
use ratatui::widgets::Paragraph;
use regex::Regex;
use scrt::file::new_scrtstore;
use scrt::ppgen::generate_pass_phrase;
use std::env;
use std::path::{Path, PathBuf};

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
    error_msgs: Option<Vec<String>>,
}

fn validate_password(password: &str) -> Option<String> {
    let password = password.trim();
    if password.is_empty() {
        return Some("Password can't be empty!".to_string());
    }

    if !(10..=512).contains(&password.len()) {
        return Some("Password should have atleast 10 to max 512 characters!".to_string());
    }
    let ascii = Regex::new(r"^[\x21-\x7E]+$").unwrap();
    let lowercase = Regex::new(r"[a-z]").unwrap();
    let uppercase = Regex::new(r"[A-Z]").unwrap();
    let number = Regex::new(r"[0-9]").unwrap();
    let symbol = Regex::new(r"[^a-zA-Z0-9]").unwrap();

    if !(ascii.is_match(password)
        && lowercase.is_match(password)
        && uppercase.is_match(password)
        && number.is_match(password)
        && symbol.is_match(password))
    {
        return Some("Password should have one Uppercase, one lowercase, symbols & numerical characters. And ASCII characters only allowed.".to_string());
    }

    None
}

fn validate_store_name(name: &str) -> Option<String> {
    let name = name.trim();

    if name.is_empty() {
        return Some("Store name can't be empty!".to_string());
    }

    if !(3..=256).contains(&name.len()) {
        return Some("Store name should have 3 to 256 characters.".to_string());
    }

    None
}

fn validate_phrase(phrase: &str) -> Option<String> {
    let re = Regex::new(r"\A[A-Za-z0-9]{3,}(?: [A-Za-z0-9]{3,}){11,127}\z").unwrap();

    if re.is_match(phrase) {
        return None;
    }

    Some("Valid phrase should have atleast 12 to 128 words with each word separated by space, each word with atleast 3 characters and all ASCII.".to_string())
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

        if let Some(errors) = self.error_msgs.as_ref() {
            frame.render_widget(
                Paragraph::new(errors.join("\n")).fg(Color::Red),
                _footer_area,
            );
        }
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
            error_msgs: None,
        })
    }

    fn handle_submit(&mut self) -> Result<PathBuf, Vec<String>> {
        let mut errors: Vec<String> = Vec::new();
        if let Some(pass_msg) = validate_password(&self.password.get_content()) {
            errors.push(pass_msg);
        }
        if let Some(phrase_msg) = validate_phrase(&self.passphrase.get_content()) {
            errors.push(phrase_msg);
        }
        if let Some(name_msg) = validate_store_name(&self.store_name.get_content()) {
            errors.push(name_msg);
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        let mut filepath = self.path.get_content();
        filepath.push(self.store_name.get_content());
        filepath.set_extension("scrt");

        match new_scrtstore(
            &filepath,
            self.password.get_content(),
            self.passphrase.get_content(),
        ) {
            Ok(()) => {
                StoreMetaData::new(&filepath).save();
                Ok(filepath)
            }
            Err(e) => {
                errors.push(e.to_string());
                Err(errors)
            }
        }
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

            KeyCode::Enter if self.selected_field.is(Pf::Submit) => match self.handle_submit() {
                Ok(path) => EventResult::NxtState(Login::new(path)),
                Err(msgs) => {
                    self.error_msgs = Some(msgs);
                    EventResult::Consumed
                }
            },

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
