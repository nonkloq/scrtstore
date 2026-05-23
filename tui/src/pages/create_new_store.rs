// use crate::{
//     components::input_box::{SecretBox, SecretBoxW, TextBox, TextBoxW},
//     pages::Pages,
// };
// use crossterm::event::KeyCode;
// use ratatui::{
//     Frame,
//     layout::{Alignment, Constraint, Direction, Layout},
//     style::{Color, Style},
//     widgets::{Block, Borders, Paragraph},
// };
// use scrt::ppgen::generate_pass_phrase;
// use std::{collections::HashMap, path::PathBuf};
//
// const PASS_MINLEN: usize = 10;
// const PASS_MAXLEN: usize = 128;
//
// #[derive(PartialEq)]
// enum ActiveField {
//     Pass,
//     Phrase,
//     File,
//     Folder,
//     Submit,
// }
// macro_rules! set_cursor {
//     ($frame:expr, $chunk:expr, $state:expr) => {{
//         let y = $state.get_cursor_pos();
//         $frame.set_cursor($chunk.x, y);
//     }};
// }
// impl ActiveField {
//     fn next(&mut self) -> Self {
//         match self {
//             Self::Pass => Self::Phrase,
//             Self::Phrase => Self::File,
//             Self::File => Self::Folder,
//             Self::Folder => Self::Submit,
//             Self::Submit => Self::Pass,
//         }
//     }
//
//     fn prev(&mut self) -> Self {
//         match self {
//             Self::Pass => Self::Submit,
//             Self::Phrase => Self::Pass,
//             Self::File => Self::Phrase,
//             Self::Folder => Self::File,
//             Self::Submit => Self::Folder,
//         }
//     }
// }
// pub struct NewStore {
//     password: SecretBox,
//     passphrase: SecretBox,
//     filename: TextBox,
//     folder: TextBox,
//
//     // 0..=3 indicating the store field
//     idx_pos: ActiveField,
// }
//
// impl NewStore {
//     pub fn render(&mut self, frame: &mut Frame) {
//         let block = Block::default()
//             .title("Create New Store")
//             .borders(Borders::ALL);
//
//         let area = frame.area();
//
//         let inner = block.inner(area);
//         frame.render_widget(block, area);
//         let chunks = Layout::default()
//             .direction(Direction::Vertical)
//             .constraints([
//                 Constraint::Length(3), // password
//                 Constraint::Length(3), // passphrase
//                 Constraint::Length(3), // filename
//                 Constraint::Length(3), // folder
//                 Constraint::Length(3), // submit
//                 Constraint::Min(0),
//             ])
//             .split(inner);
//         // 0 → Password
//         frame.render_stateful_widget(SecretBoxW, chunks[0], &mut self.password);
//
//         // 1 → Passphrase
//         frame.render_stateful_widget(SecretBoxW, chunks[1], &mut self.passphrase);
//
//         // 2 → Filename
//         frame.render_stateful_widget(TextBoxW, chunks[2], &mut self.filename);
//
//         // 3 → Folder
//         frame.render_stateful_widget(TextBoxW, chunks[3], &mut self.folder);
//
//         // 4 → Submit button
//         let is_selected = matches!(self.idx_pos, ActiveField::Submit);
//
//         let submit = Paragraph::new(" Submit ")
//             .block(
//                 Block::default()
//                     .borders(Borders::ALL)
//                     .title("Action")
//                     .border_style(if is_selected {
//                         Style::default().fg(Color::Yellow)
//                     } else {
//                         Style::default()
//                     }),
//             )
//             .alignment(Alignment::Center);
//
//         frame.render_widget(submit, chunks[4]);
//
//         let (x, y) = match self.idx_pos {
//             ActiveField::Pass => {
//                 let y = self.password.get_cursor_pos();
//                 (chunks[0].x, chunks[0].y + y)
//             }
//             ActiveField::Phrase => {
//                 let y = self.passphrase.get_cursor_pos();
//                 (chunks[1].x, chunks[1].y + y)
//             }
//             ActiveField::File => {
//                 let y = self.filename.get_cursor_pos();
//                 (chunks[2].x, chunks[2].y + y)
//             }
//             ActiveField::Folder => {
//                 let y = self.folder.get_cursor_pos();
//                 (chunks[3].x, chunks[3].y + y)
//             }
//             ActiveField::Submit => return,
//         };
//
//         frame.set_cursor_position((x, y));
//     }
//
//     pub fn handle_key(&mut self, key: KeyCode) -> Option<Pages> {
//         match key {
//             KeyCode::Enter if self.idx_pos == ActiveField::Submit => todo!(),
//
//             KeyCode::Up | KeyCode::BackTab | KeyCode::Enter => {
//                 self.idx_pos = self.idx_pos.prev();
//                 None
//             }
//
//             KeyCode::Down | KeyCode::Tab => {
//                 self.idx_pos = self.idx_pos.next();
//                 None
//             }
//
//             _ => {
//                 match self.idx_pos {
//                     ActiveField::Pass => {
//                         self.password.handle_key(key);
//                     }
//                     ActiveField::Phrase => {
//                         self.password.handle_key(key);
//                     }
//                     ActiveField::File => {
//                         self.password.handle_key(key);
//                     }
//                     ActiveField::Folder => {
//                         self.password.handle_key(key);
//                     }
//                     _ => (),
//                 }
//                 None
//             }
//         }
//     }
// }
//
// impl Default for NewStore {
//     fn default() -> Self {
//         Self {
//             folder: TextBox::new("Folder".to_string(), None),
//             password: SecretBox::new("Password".to_string(), None),
//             passphrase: SecretBox::new("PassPhrase".to_string(), Some(generate_pass_phrase(12))),
//             filename: TextBox::new("Filename".to_string(), Some("passwords".to_string())),
//             idx_pos: ActiveField::Pass,
//         }
//     }
// }
