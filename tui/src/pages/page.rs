use crossterm::event::Event;
use ratatui::Frame;

pub type PageObject = Box<dyn Page>;

pub enum EventResult {
    NotConsumed,          // When event not used in page
    Consumed,             // After exhausting the event
    NxtState(PageObject), // State change triggered by event
}
pub trait Page {
    fn render(&self, frame: &mut Frame);

    #[allow(unused_variables)]
    fn handle_event(&mut self, event: Event) -> EventResult {
        EventResult::NotConsumed // Do not consume by default
    }
}
