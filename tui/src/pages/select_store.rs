use std::str::FromStr;

use crate::{
    pages::{
        create_new_store::CreateNewStore,
        login::Login,
        page::{EventResult, Page},
    },
    svc::utils::StoreMetaData,
};

use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, HighlightSpacing, List, ListItem, ListState, Paragraph,
        Scrollbar, ScrollbarOrientation, ScrollbarState,
    },
};

pub struct SelectStore {
    stores: Vec<StoreMetaData>,
    selected_option: usize,
}

const BANNER: &str = r#"
                                                            
 ▄▄▄▄▄▄▄                    ▄▄▄▄▄▄▄                         
█████▀▀▀              ██   █████▀▀▀  ██                     
 ▀████▄  ▄████ ████▄ ▀██▀▀  ▀████▄  ▀██▀▀ ▄███▄ ████▄ ▄█▀█▄ 
   ▀████ ██    ██ ▀▀  ██      ▀████  ██   ██ ██ ██ ▀▀ ██▄█▀ 
███████▀ ▀████ ██     ██   ███████▀  ██   ▀███▀ ██    ▀█▄▄▄ 
                                                            
                                                            
"#;

impl Page for SelectStore {
    fn render(&self, frame: &mut ratatui::prelude::Frame) {
        let area = frame.area();
        // Main vertical layout
        let [_head_padding, banner_area, content_area, _footer_area] = Layout::vertical([
            Constraint::Percentage(10),
            Constraint::Percentage(25),
            Constraint::Percentage(55),
            Constraint::Percentage(10),
        ])
        .areas(area);

        // Banner
        frame.render_widget(
            Paragraph::new(BANNER)
                .alignment(Alignment::Center)
                .light_blue(),
            banner_area,
        );

        // Center the content horizontally
        let [_, content_area, _] = Layout::horizontal([
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ])
        .areas(content_area);

        // Store list + Create New Store button
        let [list_area, create_area] =
            Layout::vertical([Constraint::Min(5), Constraint::Length(3)])
                .spacing(1)
                .areas(content_area);

        // -------------------------
        // Store list
        // -------------------------

        if !self.stores.is_empty() {
            let items: Vec<ListItem> = self
                .stores
                .iter()
                .map(|store| {
                    ListItem::new(vec![
                        Line::from(store.get_display_path()),
                        Line::from(vec![
                            Span::styled(
                                "Last accessed: ",
                                Style::default()
                                    .fg(Color::DarkGray)
                                    .add_modifier(Modifier::DIM),
                            ),
                            Span::styled(
                                store.get_last_accessed_timestamp().to_string(),
                                Style::default().fg(Color::Gray).add_modifier(Modifier::DIM),
                            ),
                        ]),
                        Line::from(""),
                    ])
                })
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .title(" Select Store ")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title_alignment(Alignment::Center),
                )
                .highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("▶ ")
                .highlight_spacing(HighlightSpacing::Always)
                .scroll_padding(3);

            let mut list_state = ListState::default();
            if self.selected_option < self.stores.len() {
                list_state = list_state.with_selected(Some(self.selected_option));
            }

            frame.render_stateful_widget(list, list_area, &mut list_state);
            let mut scrollbar_state =
                ScrollbarState::new(self.stores.len()).position(self.selected_option);

            let scrollbar = Scrollbar::default().orientation(ScrollbarOrientation::VerticalRight);

            frame.render_stateful_widget(scrollbar, list_area, &mut scrollbar_state);
        }

        // -------------------------
        // Create New Store
        // -------------------------

        let create_selected = self.selected_option == self.stores.len();

        let create_style = if create_selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let create_new = Paragraph::new("Create New Store")
            .alignment(Alignment::Center)
            .style(create_style)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            );

        frame.render_widget(create_new, create_area);
    }

    fn handle_event(&mut self, event: crossterm::event::Event) -> super::page::EventResult {
        match event {
            Event::Key(key) => self.handle_key(key),
            _ => EventResult::NotConsumed,
        }
    }
}

impl SelectStore {
    pub fn new(stores: Vec<StoreMetaData>) -> Box<Self> {
        Box::new(Self {
            stores,
            selected_option: 0,
        })
    }

    fn handle_key(&mut self, key: KeyEvent) -> EventResult {
        match key.code {
            KeyCode::Up | KeyCode::BackTab | KeyCode::Backspace | KeyCode::Left
                if self.selected_option > 0 =>
            {
                self.selected_option -= 1;
                EventResult::Consumed
            }
            KeyCode::Down | KeyCode::Tab | KeyCode::Delete | KeyCode::Right
                if self.selected_option < self.stores.len() =>
            {
                self.selected_option += 1;
                EventResult::Consumed
            }
            KeyCode::Enter if self.selected_option < self.stores.len() => {
                EventResult::NxtState(Login::new())
            }
            KeyCode::Enter if self.selected_option >= self.stores.len() => {
                EventResult::NxtState(CreateNewStore::new())
            }
            _ => EventResult::NotConsumed,
        }
    }
}
