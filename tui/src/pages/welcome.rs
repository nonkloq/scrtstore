use crossterm::event::{Event, KeyCode};
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    widgets::Paragraph,
};

use crate::{
    pages::{
        page::{EventResult, Page},
        select_store::SelectStore,
    },
    svc::utils::{get_known_stores, get_project_dirs},
};

pub struct Welcome;

const BANNER: &str = r#"
                                                            
 ▄▄▄▄▄▄▄                    ▄▄▄▄▄▄▄                         
█████▀▀▀              ██   █████▀▀▀  ██                     
 ▀████▄  ▄████ ████▄ ▀██▀▀  ▀████▄  ▀██▀▀ ▄███▄ ████▄ ▄█▀█▄ 
   ▀████ ██    ██ ▀▀  ██      ▀████  ██   ██ ██ ██ ▀▀ ██▄█▀ 
███████▀ ▀████ ██     ██   ███████▀  ██   ▀███▀ ██    ▀█▄▄▄ 
                                                            
                                                            
"#;

impl Page for Welcome {
    fn render(&self, frame: &mut ratatui::prelude::Frame) {
        let area = frame.area();
        let chunks = Layout::vertical([
            Constraint::Percentage(30),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(30),
        ])
        .split(area);

        frame.render_widget(
            Paragraph::new(BANNER).alignment(Alignment::Center),
            chunks[1],
        );
        frame.render_widget(
            Paragraph::new("\"q\" to Exit. Anything to Enter.").alignment(Alignment::Center),
            chunks[2],
        );
    }

    fn handle_event(&self, event: crossterm::event::Event) -> EventResult {
        if matches!(event, Event::Key(e) if e.code == KeyCode::Char('q')) {
            return EventResult::NotConsumed;
        }

        EventResult::NxtState(SelectStore::new(get_known_stores(&get_project_dirs())))
    }
}

impl Welcome {
    pub fn new() -> Box<Self> {
        Box::new(Welcome)
    }
}
