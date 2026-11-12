use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::{Color, Style, Stylize};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use scrt::Vault;
use std::path::PathBuf;

use scrt::{ScrtStore, file::load_scrtstore};

use crate::components::button::render_button;
use crate::pages::page::EventResult;
use crate::svc::utils::{get_known_stores, get_project_dirs};
use crate::{
    components::input_field::InputField,
    pages::{page::Page, select_store::SelectStore, vault_page::VaultPage},
};

#[derive(PartialEq, Clone)]
enum Pf {
    Pass,
    Phrase,
    Submit,
    None,

    HomeDialog(bool),
}

impl Pf {
    fn is(&self, rhs: Self) -> bool {
        *self == rhs
    }
    fn nxt(&self) -> Self {
        match self {
            Pf::Pass => Pf::Phrase,
            Pf::Phrase => Pf::Submit,
            Pf::Submit => Pf::Submit,

            Pf::None => Pf::Pass,

            Pf::HomeDialog(x) => Pf::HomeDialog(!x),
        }
    }

    fn prv(&self) -> Self {
        match self {
            Pf::Pass => Pf::Pass,
            Pf::Phrase => Pf::Pass,
            Pf::Submit => Pf::Phrase,

            Pf::None => Pf::Submit,

            Pf::HomeDialog(x) => Pf::HomeDialog(!x),
        }
    }
}

pub struct Login<'a> {
    scrt_store: ScrtStore,
    path: PathBuf,

    password: InputField<'a>,
    passphrase: InputField<'a>,

    selected_field: Pf,

    error_msg: Option<String>,
}

impl<'a> Page for Login<'a> {
    fn render(&self, frame: &mut ratatui::prelude::Frame) {
        match self.selected_field {
            Pf::HomeDialog(x) => self.render_dialog(frame, x),
            _ => self.render_page(frame),
        }
    }

    fn handle_event(&mut self, event: crossterm::event::Event) -> super::page::EventResult {
        match event {
            Event::Key(key) => self.handle_key(key),
            Event::Paste(val) => self.handle_paste(val),
            _ => EventResult::NotConsumed,
        }
    }
}

impl<'a> Login<'a> {
    pub fn new(path: PathBuf) -> Box<Self> {
        let ss = load_scrtstore(&path).unwrap();

        Box::new(Login {
            path,
            scrt_store: ss,
            password: InputField::new("Password", None, None),
            passphrase: InputField::new("Phrase", None, None),

            selected_field: Pf::Pass,
            error_msg: None,
        })
    }

    fn handle_key(&mut self, key: KeyEvent) -> EventResult {
        let field_consume = match self.selected_field {
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

            KeyCode::Enter if self.selected_field.is(Pf::Submit) => match self.handle_submit() {
                Ok(_) => EventResult::NxtState(VaultPage::new()),
                Err(msgs) => {
                    self.error_msg = Some(msgs);
                    EventResult::Consumed
                }
            },

            KeyCode::Enter if self.selected_field.is(Pf::HomeDialog(false)) => {
                self.selected_field = Pf::None;
                EventResult::Consumed
            }

            KeyCode::Enter if self.selected_field.is(Pf::HomeDialog(true)) => {
                EventResult::NxtState(SelectStore::new(get_known_stores(&get_project_dirs())))
            }

            KeyCode::Left | KeyCode::Right if matches!(self.selected_field, Pf::HomeDialog(_)) => {
                self.selected_field = self.selected_field.nxt();
                EventResult::Consumed
            }

            KeyCode::Esc => {
                if self.selected_field.is(Pf::None) {
                    self.selected_field = Pf::HomeDialog(false);
                } else {
                    self.selected_field = Pf::None
                }
                EventResult::Consumed
            }

            _ => EventResult::NotConsumed,
        }
    }

    fn handle_paste(&mut self, val: String) -> EventResult {
        let state = match self.selected_field {
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

    fn handle_submit(&self) -> Result<Vault, String> {
        let master_key = match self.scrt_store.get_master_key(
            scrt::auth::twofa::TwoFAMethod::PassPhrase,
            self.password.get_content(),
            self.passphrase.get_content(),
        ) {
            Ok(k) => k,
            Err(de) => return Err(de.to_string()),
        };

        self.scrt_store
            .unlock(&master_key)
            .map_err(|de| de.to_string())
    }
}

impl<'a> Login<'a> {
    fn render_dialog(&self, frame: &mut ratatui::prelude::Frame, option: bool) {
        let [_head_padding, cont_area, _footer_area] = Layout::vertical([
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ])
        .areas(frame.area());

        let [_, dalog_area, _] = Layout::horizontal([
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ])
        .areas(cont_area);

        let oblock = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .fg(Color::LightBlue);
        let inner = oblock.inner(dalog_area);

        frame.render_widget(oblock, dalog_area);

        let [bth, ays, _, options] = Layout::vertical([
            Constraint::Percentage(10),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Percentage(20),
        ])
        .areas(inner);

        let title = Paragraph::new("Go back to home.").alignment(Alignment::Center);
        frame.render_widget(title, bth);
        let ays_txt =
            Paragraph::new("Are you sure?\n Press [Esc] / [No] to exit. [Yes] to Go back to home.\nUse [Tab]/[Arrow] Keys to switch options.\n")
                .alignment(Alignment::Center);
        frame.render_widget(ays_txt, ays);

        let [_, yes, _, no, _] = Layout::horizontal([
            Constraint::Percentage(30),
            Constraint::Percentage(15),
            Constraint::Percentage(10),
            Constraint::Percentage(15),
            Constraint::Percentage(30),
        ])
        .areas(options);

        let yess = Paragraph::new("yes").alignment(Alignment::Center).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Thick)
                .fg(if option {
                    Color::Yellow
                } else {
                    Color::DarkGray
                }),
        );

        let noo = Paragraph::new("no").alignment(Alignment::Center).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Thick)
                .fg(if !option {
                    Color::Yellow
                } else {
                    Color::DarkGray
                }),
        );

        frame.render_widget(yess, yes);
        frame.render_widget(noo, no);
    }

    fn render_page(&self, frame: &mut ratatui::prelude::Frame) {
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

        let [_, head, password, passphrase, submit, dyn_info, _] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Fill(1),
        ])
        .areas(field_area);

        let [erro_msg_line, _] =
            Layout::vertical([Constraint::Length(10), Constraint::Fill(1)]).areas(_footer_area);

        frame.render_widget(
            Paragraph::new(format!("Opening: {}", self.path.to_str().unwrap_or("Na")))
                .alignment(Alignment::Center)
                .style(Style::default().bold().fg(Color::Green)),
            head,
        );

        if let Some(error) = self.error_msg.as_ref() {
            frame.render_widget(
                Paragraph::new(error.as_str())
                    .fg(Color::Red)
                    .alignment(Alignment::Left)
                    .wrap(Wrap { trim: false }),
                erro_msg_line,
            );
        }

        let common_nav = "Use [Tab/Shift-Tab]/[Up/Down Arrow] Keys to navigate";
        let info_txt = match self.selected_field {
            Pf::None => format!(
                "[q] To quit\n[Esc] to go back to home\n{}",
                common_nav.replace("navigate", "select")
            ),
            Pf::Submit => format!(
                "Press [Enter] to Login\n[Esc] to de-select the field\n[q] To quit\n{}",
                common_nav
            ),
            Pf::Pass | Pf::Phrase => {
                format!(
                    "Use [Left/Right Arrow] key to move cursor\n[Esc] to de-select the field\n{}",
                    common_nav
                )
            }
            Pf::HomeDialog(_) => "".to_string(),
        };
        frame.render_widget(
            Paragraph::new(info_txt)
                .fg(Color::LightBlue)
                .alignment(Alignment::Right),
            dyn_info,
        );
        render_button(
            "Login 󰍂 ",
            submit,
            frame,
            self.selected_field.is(Pf::Submit),
        );
        self.password
            .render_widget(password, frame, self.selected_field.is(Pf::Pass));

        self.passphrase
            .render_widget(passphrase, frame, self.selected_field.is(Pf::Phrase));
    }
}
