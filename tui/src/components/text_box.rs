use crossterm::event::KeyCode;
use ratatui::{
    style::Style,
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget, Wrap},
};

use crate::components::input_widget::InputWidget;

pub struct TextBox {
    text: String,
    title: String,
    cursor_pos: usize,
    line_number: usize,
}

impl TextBox {
    pub fn new(title: String) -> Self {
        TextBox {
            text: String::from(""),
            title,
            cursor_pos: 0,
            line_number: 0,
        }
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

        let remchar = if remove_idx == self.text.len() - 1 {
            self.text.pop().unwrap_or(' ')
        } else {
            self.text.remove(remove_idx)
        };

        if remchar == '\n' && self.line_number > 0 {
            self.line_number -= 1;
        }

        if is_prev {
            self.cursor_pos -= 1;
        }
    }
    fn add_char(&mut self, word: char) {
        if self.cursor_pos >= self.text.len() {
            self.text.push(word);
        } else {
            self.text.insert(self.cursor_pos, word);
        }
        self.cursor_pos += 1;

        if word == '\n' {
            self.line_number += 1;
        }
    }

    fn is_new_line(&self, idx: usize) -> bool {
        self.text
            .get(idx..=idx)
            .map(|s| s.chars().any(|c| c == '\n'))
            .unwrap_or(false)
    }

    fn move_cursor(&mut self, is_left: bool) {
        if is_left && self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            if self.is_new_line(self.cursor_pos) {
                self.line_number -= 1
            }
        } else if !is_left && self.cursor_pos < self.text.len() {
            if self.is_new_line(self.cursor_pos) {
                self.line_number += 1
            }
            self.cursor_pos += 1;
        }
    }
}

impl InputWidget for TextBox {
    type UIElem = TextBoxW;
    fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Backspace => self.remove_char(true),
            KeyCode::Left => self.move_cursor(true),
            KeyCode::Right => self.move_cursor(false),
            KeyCode::Delete => self.remove_char(false),
            KeyCode::Char(c) => self.add_char(c),
            KeyCode::Enter => self.add_char('\n'),
            _ => {}
        }
    }
    fn handle_paste(&mut self, _data: String) {
        if _data.is_empty() {
            return;
        }
        for ch in _data.chars() {
            self.add_char(ch);
        }
    }

    fn get_cursor_position(&self) -> Option<(u16, u16)> {
        let n = self.line_number;
        let idx = if n == 0 {
            0
        } else {
            self.text
                .char_indices()
                .filter(|(_, c)| *c == '\n') // only newline chars
                .nth(n - 1) // nth newline (1-based)
                .map(|(idx, _)| idx + 1)
                .unwrap_or(0)
        };
        Some(((self.cursor_pos - idx) as u16, self.line_number as u16))
    }
}

#[derive(Default)]
pub struct TextBoxW;

impl StatefulWidget for TextBoxW {
    type State = TextBox;
    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let p = Paragraph::new(state.text.clone())
            .style(Style::default().fg(ratatui::style::Color::White))
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .title_top(state.title.clone())
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded),
            );
        p.render(area, buf);
    }
}
