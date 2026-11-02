use std::{cell::RefCell, path::PathBuf};

use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatree::{FilePicker as Fpw, FilePickerState, FilePickerTheme, PickerMode, PickerResult};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, BorderType, Borders, Paragraph},
};

pub struct FilePicker<'a> {
    title: &'a str,
    path: PathBuf,
    mode: PickerMode,
    state: RefCell<Option<FilePickerState>>,
}

impl<'a> FilePicker<'a> {
    pub fn new(title: &'a str, path: PathBuf, mode: PickerMode) -> FilePicker<'a> {
        FilePicker {
            title,
            path,
            state: RefCell::new(None),
            mode,
        }
    }

    pub fn get_content(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn render_widget(
        &self,
        area: Rect,
        frame: &mut ratatui::prelude::Frame,
        is_selected: bool,
    ) {
        let mut fp_open = false;
        if let Some(state) = self.state.borrow_mut().as_mut() {
            let above_box = Rect {
                x: area.x,
                y: area.y - area.height * 3,
                height: area.height * 3,
                width: area.width,
            };
            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::LightTripleDashed)
                .fg(Color::Yellow);
            let inner = block.inner(above_box);
            frame.render_widget(block, above_box);
            frame.render_stateful_widget(Fpw::default(), inner, state);
            fp_open = true;
        }

        let widstyle = if is_selected {
            Style::default()
                .fg(if fp_open {
                    Color::DarkGray
                } else {
                    Color::Yellow
                })
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Reset)
        };
        let path_widget = Paragraph::new(self.path.to_str().unwrap())
            .block(
                Block::default()
                    .title_top(self.title)
                    .title_alignment(Alignment::Left)
                    .borders(Borders::ALL)
                    .border_type(BorderType::HeavyDoubleDashed),
            )
            .style(widstyle);

        frame.render_widget(path_widget, area);
    }

    pub fn handle_event(&mut self, event: KeyEvent) -> bool {
        if event.code == KeyCode::Enter && self.state.borrow().is_none() {
            let theme = FilePickerTheme {
                status_bar: Style::default().fg(Color::Black),
                ..FilePickerTheme::default()
            };
            _ = self.state.replace(Some(
                FilePickerState::builder()
                    .start_dir(self.path.to_str().unwrap())
                    .mode(self.mode)
                    .theme(theme)
                    .build(),
            ));
            return true;
        }

        let mut state_obj = self.state.borrow_mut();
        if let Some(state) = state_obj.as_mut() {
            state.handle_event(Event::Key(event));

            match state.result() {
                PickerResult::Selected(paths) => {
                    self.path = paths[0].clone();
                    *state_obj = None;
                }
                PickerResult::Cancelled => {
                    *state_obj = None;
                }
                PickerResult::Pending => {}
            }

            return true;
        }

        false
    }
}
