use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};

pub fn render_button(
    text: &str,
    area: Rect,
    frame: &mut ratatui::prelude::Frame,
    is_selected: bool,
) {
    let button_style = if is_selected {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Reset)
    };

    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .style(button_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );

    frame.render_widget(paragraph, area);
}
