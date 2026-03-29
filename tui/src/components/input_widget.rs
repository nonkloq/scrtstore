/// Widgets that are controlled by user inputs
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{Frame, layout::Rect, widgets::StatefulWidget};

pub trait InputWidget {
    type UIElem: StatefulWidget<State = Self> + Default;
    /// Handle Keyboard events
    fn handle_key(&mut self, key: KeyEvent);

    /// Get Cursor position relative to the widget
    /// None if widget doesn't have a cursor
    fn get_cursor_position(&self) -> Option<(u16, u16)>;

    /// Handle Mouse events
    fn handle_mouse(&mut self, _key: MouseEvent) {}

    /// Handle paste from clipboard
    fn handle_paste(&mut self, _data: String) {}

    fn set_active(&mut self) {}
    fn set_inactive(&mut self) {}

    /// Create new stateful widget and render it
    fn render_widget(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_stateful_widget(Self::UIElem::default(), area, self);
    }

    /// Set cursor
    fn set_cursor(&mut self, frame: &mut Frame, area: Rect) {
        if let Some((col, line)) = self.get_cursor_position() {
            let x = area.x + 1 + col; // +1 for left border
            let y = area.y + 1 + line; // inside the box
            frame.set_cursor_position((x, y));
        };
    }
}
