use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub struct InputField<'a> {
    title: &'a str,
    placeholder: Option<&'a str>,
    text: String,
    cursor_pos: usize,
}

// pub struct SecretInputField {
//     tbox: InputField,
//     is_visible: bool,
// }

impl<'a> InputField<'a> {
    pub fn new(title: &'a str, text: Option<&'a str>, placeholder: Option<&'a str>) -> Self {
        let text = if let Some(val) = text {
            String::from(val)
        } else {
            "".to_string()
        };

        InputField {
            title,
            text,
            placeholder,
            cursor_pos: 0,
        }
    }

    pub fn set_content(&mut self, new_txt: String) {
        self.text = new_txt
    }

    pub fn get_content(&self) -> String {
        self.text.clone()
    }

    pub fn get_cursor_pos(&self) -> u16 {
        self.cursor_pos as u16
    }

    fn remove_char(&mut self, is_prev: bool) {
        if (is_prev && self.cursor_pos == 0) || (!is_prev && self.cursor_pos >= self.text.len()) {
            return;
        }

        // Delete condition: Remove after cursor (exact index of cursor)
        let mut remove_idx: usize = self.cursor_pos;
        if is_prev {
            remove_idx -= 1;
        }

        if remove_idx == self.text.len() - 1 {
            self.text.pop().unwrap_or(' ')
        } else {
            self.text.remove(remove_idx)
        };

        if is_prev {
            self.cursor_pos -= 1;
        }
    }

    fn move_cursor(&mut self, is_left: bool) {
        if is_left && self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        } else if !is_left && self.cursor_pos < self.text.len() {
            self.cursor_pos += 1;
        }
    }

    fn add_char(&mut self, word: char) {
        if self.cursor_pos >= self.text.len() {
            self.text.push(word);
        } else {
            self.text.insert(self.cursor_pos, word);
        }
        self.cursor_pos += 1;
    }
}

impl<'a> InputField<'a> {
    pub fn handle_event(&mut self, event: KeyEvent) -> bool {
        match event.code {
            KeyCode::Backspace => self.remove_char(true),
            KeyCode::Left => self.move_cursor(true),
            KeyCode::Right => self.move_cursor(false),
            KeyCode::Delete => self.remove_char(false),
            KeyCode::Char(c) => self.add_char(c),
            _ => return false,
        }
        true
    }

    pub fn handle_paste(&mut self, val: String) -> bool {
        if val.is_empty() {
            return false;
        }

        self.text.insert_str(self.cursor_pos, &val);

        self.cursor_pos += val.len();
        true
    }

    pub fn render_widget(
        &self,
        area: Rect,
        frame: &mut ratatui::prelude::Frame,
        is_selected: bool,
    ) {
        let block = Block::default()
            .title_top(self.title)
            .title_alignment(Alignment::Left)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if is_selected {
                Color::Yellow
            } else {
                Color::Reset
            }));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let use_placeholder = self.text.is_empty();

        let text = if use_placeholder && let Some(plc) = self.placeholder {
            plc
        } else {
            self.text.as_str()
        };

        let text = Paragraph::new(text)
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left)
            .fg(if use_placeholder {
                Color::DarkGray
            } else {
                Color::Reset
            });

        if is_selected {
            frame.set_cursor_position((inner.x + self.get_cursor_pos(), inner.y));
        }

        frame.render_widget(text, inner);
    }
}

// impl SecretInputField {
//     pub fn new(title: String, text: Option<String>) -> Self {
//         SecretInputField {
//             tbox: InputField::new(title, text),
//             is_visible: false,
//         }
//     }
//
//     pub fn get_content(&self) -> String {
//         self.tbox.get_content()
//     }
//
//     pub fn get_cursor_pos(&self) -> u16 {
//         self.tbox.get_cursor_pos()
//     }
// }

// impl WidgetNode for SecretInputField {
//     fn handle_key_event(&mut self, event: KeyEvent) -> bool {
//         if event.code == KeyCode::Char('h') && event.modifiers.contains(KeyModifiers::ALT) {
//             self.is_visible = !self.is_visible;
//             return true;
//         }
//         self.tbox.handle_key_event(event)
//     }
//
//     fn get_length(&self) -> u16 {
//         self.tbox.get_length()
//     }
//
//     fn render_widget(&self, area: Rect, frame: &mut ratatui::prelude::Frame, is_selected: bool) {
//         let content = if self.is_visible {
//             self.tbox.text.clone()
//         } else {
//             "*".repeat(self.tbox.text.len())
//         };
//
//         let eye = if self.is_visible {
//             "🙉 (Alt+h)"
//         } else {
//             "🙈 (Alt+h)"
//         };
//
//         let block = Block::default()
//             .title_top(self.tbox.title.clone())
//             .title_alignment(Alignment::Left)
//             .borders(Borders::ALL);
//
//         let inner = block.inner(area);
//         frame.render_widget(block, area);
//
//         let chunks = Layout::default()
//             .direction(Direction::Horizontal)
//             .constraints([Constraint::Percentage(94), Constraint::Percentage(6)])
//             .split(inner);
//
//         let text_widget = Paragraph::new(content).alignment(Alignment::Left);
//         frame.render_widget(text_widget, chunks[0]);
//         let eye_widget = Paragraph::new(eye).alignment(Alignment::Center);
//         frame.render_widget(eye_widget, chunks[1]);
//
//         if is_selected {
//             frame.set_cursor_position((chunks[0].x + self.get_cursor_pos(), chunks[0].y));
//         }
//     }
// }
