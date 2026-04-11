use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text,
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget, Wrap},
};

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

    pub fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Backspace => self.remove_char(true),
            KeyCode::Left => self.move_cursor(true),
            KeyCode::Right => self.move_cursor(false),
            KeyCode::Delete => self.remove_char(false),
            KeyCode::Char(c) => self.add_char(c),
            _ => {}
        }
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

    pub fn handle_key(&mut self, key: KeyCode) {
        self.tbox.handle_key(key);
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
        let text = Paragraph::new(state.text.clone())
            .block(
                Block::default()
                    .title_top(state.title.clone())
                    .title_alignment(Alignment::Left)
                    .borders(Borders::ALL),
            )
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left);
        text.render(area, buf);
    }
}
#[derive(Default)]
pub struct SecretBoxW;

impl StatefulWidget for SecretBoxW {
    type State = SecretBox;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let content = if state.is_visible {
            state.tbox.text.clone()
        } else {
            "*".repeat(state.tbox.text.len())
        };

        let eye = if state.is_visible { "👁" } else { "🙈" };

        let block = Block::default()
            .title_top(state.tbox.title.clone())
            .title_alignment(Alignment::Left)
            .borders(Borders::ALL);

        let inner = block.inner(area);
        block.render(area, buf);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(85), Constraint::Percentage(15)])
            .split(inner);

        let text_widget = Paragraph::new(content).alignment(Alignment::Left);
        text_widget.render(chunks[0], buf);
        let eye_widget = Paragraph::new(eye).alignment(Alignment::Center);
        eye_widget.render(chunks[1], buf);
    }
}
