use crossterm::event::{KeyCode, MouseEvent};
use ratatui::widgets::{BorderType, List, Paragraph};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    widgets::Borders,
};
use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{Block, ListItem},
};

use crate::pages::Pages;

pub struct StoresList {
    stores: Vec<String>,
    idx_pos: usize,
}

impl StoresList {
    pub fn render(&mut self, frame: &mut Frame) {
        let items: Vec<ListItem> = self
            .stores
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                if idx == self.idx_pos {
                    ListItem::new(item.clone()).style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    ListItem::new(item.clone())
                }
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title("Select Store")
                    .borders(Borders::NONE)
                    .title_alignment(Alignment::Center)
                    .border_style(Style::default()),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ")
            .scroll_padding(3);

        let hors = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(30),
                    Constraint::Percentage(40),
                    Constraint::Percentage(30),
                ]
                .as_ref(),
            )
            .split(frame.area());

        let parts = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(40),
                    Constraint::Length(self.stores.len() as u16 + 2),
                    Constraint::Length(3),
                    Constraint::Percentage(20),
                ]
                .as_ref(),
            )
            .split(hors[1]);

        frame.render_widget(list, parts[1]);

        let is_cn_selected = self.idx_pos >= self.stores.len();
        let cn = Paragraph::new("Create New Store")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .alignment(Alignment::Center)
            .style(Style::default().fg(if is_cn_selected {
                Color::Yellow
            } else {
                Color::White
            }));
        frame.render_widget(cn, parts[2]);
    }

    pub fn handle_key(&mut self, key: KeyCode) -> Option<Pages> {
        match key {
            KeyCode::Up | KeyCode::BackTab | KeyCode::Backspace | KeyCode::Left
                if self.idx_pos > 0 =>
            {
                self.idx_pos -= 1;
                None
            }
            KeyCode::Down | KeyCode::Tab | KeyCode::Delete | KeyCode::Right
                if self.idx_pos < self.stores.len() =>
            {
                self.idx_pos += 1;
                None
            }
            KeyCode::Enter if self.idx_pos < self.stores.len() => {
                Some(Pages::Login(self.stores[self.idx_pos].clone()))
            }
            KeyCode::Enter if self.idx_pos >= self.stores.len() => Some(Pages::CreateNew),
            _ => None,
        }
    }

    pub fn handle_mouse(&mut self, _click: MouseEvent) -> Option<Pages> {
        todo!()
    }
}

impl Default for StoresList {
    fn default() -> Self {
        Self {
            stores: vec![String::from("path1/my.scrt"), String::from("path2/my.scrt")],
            idx_pos: 0,
        }
    }
}
