use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::ptree::WidgetNode;

pub struct TextBox {
    title: String,
    text: String,
    cursor: usize,
}

pub struct SecretBox {
    tbox: TextBox,
    is_visible: bool,
}

impl TextBox {
    pub fn new(title: String, text: Option<String>) -> Self {
        TextBox {
            title,
            text: text.unwrap_or("".to_string()),
            cursor: 0,
        }
    }

    pub fn get_content(&self) -> String {
        self.text.clone()
    }

    pub fn get_cursor_pos(&self) -> u16 {
        self.cursor as u16
    }

    fn remove_char(&mut self, is_prev: bool) {
        if (is_prev && self.cursor == 0) || (!is_prev && self.cursor >= self.text.len()) {
            return;
        }

        // Delete condition: Remove after cursor (exact index of cursor)
        let mut remove_idx: usize = self.cursor;
        if is_prev {
            remove_idx -= 1;
        }

        if remove_idx == self.text.len() - 1 {
            self.text.pop().unwrap_or(' ')
        } else {
            self.text.remove(remove_idx)
        };

        if is_prev {
            self.cursor -= 1;
        }
    }

    fn move_cursor(&mut self, is_left: bool) {
        if is_left && self.cursor > 0 {
            self.cursor -= 1;
        } else if !is_left && self.cursor < self.text.len() {
            self.cursor += 1;
        }
    }

    fn add_char(&mut self, word: char) {
        if self.cursor >= self.text.len() {
            self.text.push(word);
        } else {
            self.text.insert(self.cursor, word);
        }
        self.cursor += 1;
    }
}

impl WidgetNode for TextBox {
    fn handle_key_event(&mut self, event: KeyEvent) -> bool {
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

    fn get_length(&self) -> u16 {
        // 3 is fixed input field height per line, later can be added as struct field
        (self.text.matches("\n").count() as u16 + 1) * 3
    }

    fn render_widget(&self, area: Rect, frame: &mut ratatui::prelude::Frame, is_selected: bool) {
        let block = Block::default()
            .title_top(self.title.clone())
            .title_alignment(Alignment::Left)
            .borders(Borders::ALL);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let text = Paragraph::new(self.text.clone())
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left);

        if is_selected {
            frame.set_cursor_position((inner.x + self.get_cursor_pos(), inner.y));
        }

        frame.render_widget(text, inner);
    }
}

impl SecretBox {
    pub fn new(title: String, text: Option<String>) -> Self {
        SecretBox {
            tbox: TextBox::new(title, text),
            is_visible: false,
        }
    }

    pub fn get_content(&self) -> String {
        self.tbox.get_content()
    }

    pub fn get_cursor_pos(&self) -> u16 {
        self.tbox.get_cursor_pos()
    }
}

impl WidgetNode for SecretBox {
    fn handle_key_event(&mut self, event: KeyEvent) -> bool {
        if event.code == KeyCode::Char('h') && event.modifiers.contains(KeyModifiers::ALT) {
            self.is_visible = !self.is_visible;
            return true;
        }
        self.tbox.handle_key_event(event)
    }

    fn get_length(&self) -> u16 {
        self.tbox.get_length()
    }

    fn render_widget(&self, area: Rect, frame: &mut ratatui::prelude::Frame, is_selected: bool) {
        let content = if self.is_visible {
            self.tbox.text.clone()
        } else {
            "*".repeat(self.tbox.text.len())
        };

        let eye = if self.is_visible {
            "🙉 (Alt+h)"
        } else {
            "🙈 (Alt+h)"
        };

        let block = Block::default()
            .title_top(self.tbox.title.clone())
            .title_alignment(Alignment::Left)
            .borders(Borders::ALL);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(94), Constraint::Percentage(6)])
            .split(inner);

        let text_widget = Paragraph::new(content).alignment(Alignment::Left);
        frame.render_widget(text_widget, chunks[0]);
        let eye_widget = Paragraph::new(eye).alignment(Alignment::Center);
        frame.render_widget(eye_widget, chunks[1]);

        if is_selected {
            frame.set_cursor_position((chunks[0].x + self.get_cursor_pos(), chunks[0].y));
        }
    }
}
