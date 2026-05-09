use core::fmt;
use crossterm::event::DisableMouseCapture;
use crossterm::event::EnableMouseCapture;
use crossterm::event::KeyCode;
use crossterm::event::{self, Event, KeyEvent, MouseEvent};
use crossterm::{
    cursor::{Hide, Show},
    execute,
};
use gui::ptree::{ChildNode, PageTree, SectionNode, WidgetNode};
use ratatui::layout::Alignment;
use ratatui::layout::Rect;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Wrap;
use ratatui::{DefaultTerminal, Frame};
use std::io::stdout;
use std::rc::Rc;

struct CursorGuard;

impl CursorGuard {
    pub fn new() -> Result<Self, AppError> {
        execute!(stdout(), Hide)?;
        Ok(Self)
    }
}

impl Drop for CursorGuard {
    fn drop(&mut self) {
        let _ = execute!(stdout(), Show);
    }
}

fn enable_mouse() -> std::result::Result<(), AppError> {
    execute!(stdout(), EnableMouseCapture)?;
    Ok(())
}

fn disable_mouse() -> std::result::Result<(), AppError> {
    execute!(stdout(), DisableMouseCapture)?;
    Ok(())
}

#[derive(Debug)]
enum AppError {
    AnyError(String),
    IOError(std::io::Error),
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        AppError::IOError(value)
    }
}
impl From<String> for AppError {
    fn from(value: String) -> Self {
        AppError::AnyError(value)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::AnyError(err) => write!(f, "Error occured: {err}"),
            AppError::IOError(err) => write!(f, "IO Error: {err}"),
        }
    }
}
//
// trait TUIApp {
//     // Event Handlers
//     fn handle_key_event(&mut self, event: KeyEvent) -> Result<(), AppError>;
//     fn handle_mouse_event(&mut self, event: MouseEvent) -> Result<(), AppError>;
//     fn handle_paste_event(&mut self, content: String) -> Result<(), AppError>;
//     fn handle_focus_lost_event(&mut self) -> Result<(), AppError>;
//     fn handle_focus_gain_event(&mut self) -> Result<(), AppError>;
//     fn handle_resize_event(&mut self, x: u16, y: u16) -> Result<(), AppError>;
//
//     fn handle_events(&mut self) -> Result<(), AppError> {
//         let event = event::read()?;
//
//         match event {
//             Event::Key(event) => self.handle_key_event(event),
//             Event::Mouse(event) => self.handle_mouse_event(event),
//             Event::Paste(content) => self.handle_paste_event(content),
//             Event::Resize(x, y) => self.handle_resize_event(x, y),
//             Event::FocusLost => self.handle_focus_lost_event(),
//             Event::FocusGained => self.handle_focus_gain_event(),
//         }
//     }
//
//     /// If True the app will run; otherwise will close
//     fn is_running(&self) -> bool;
//
//     /// Main render block
//     fn render(&self, frame: &mut Frame);
//
//     fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), AppError> {
//         while self.is_running() {
//             terminal.draw(|frame| self.render(frame))?;
//             // It blocks the loop to listen for events
//             self.handle_events()? // blocks (waits)
//         }
//         Ok(())
//     }
// }

struct TextBlock {
    name: String,
    len: u16,
}
impl TextBlock {
    fn new(string: &str, len: u16) -> Self {
        TextBlock {
            name: string.to_string(),
            len,
        }
    }
}
impl WidgetNode for TextBlock {
    fn get_length(&self) -> u16 {
        self.len
    }
    fn render_widget(&self, area: Rect, frame: &mut Frame) {
        let p = Paragraph::new(self.name.clone())
            .block(Block::default().title("Text Block").borders(Borders::ALL))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(p, area);
    }
}
//
// struct MyApp {
//     is_alive: bool,
// }
//
// impl MyApp {
//     fn new() -> Self {
//         MyApp { is_alive: true }
//     }
// }

fn run(term: &mut DefaultTerminal, tree: &mut PageTree) -> Result<(), AppError> {
    loop {
        term.draw(|frame| {
            tree.render(frame);
        })?;

        if let Event::Key(e) = event::read()? {
            match e.code {
                KeyCode::Char('q') => break, // Exit the loop
                KeyCode::Tab => {
                    tree.move_to_next(true);
                }
                KeyCode::BackTab => tree.move_to_next(false),
                KeyCode::Enter => tree.enter(),
                KeyCode::Esc => tree.escape(),
                _ => continue,
            }
        }
    }
    Ok(())
}

fn main() {
    // let _ = disable_mouse();
    // let _ = enable_mouse();
    let l11 = ChildNode::new_widget(TextBlock::new("At First 1", 3));
    let l12 = ChildNode::new_widget(TextBlock::new("At First 2", 3));

    let l2s = ChildNode::new_section(Vec::from([
        ChildNode::new_widget(TextBlock::new("At second 1", 5)),
        ChildNode::new_widget(TextBlock::new("At second 2", 5)),
    ]));

    let l3s = ChildNode::new_section(Vec::from([
        ChildNode::new_widget(TextBlock::new("At Third 1", 4)),
        ChildNode::new_widget(TextBlock::new("At Third 2", 5)),
        ChildNode::new_widget(TextBlock::new("At Third 3", 4)),
    ]));

    let major1 = ChildNode::new_section(Vec::from([l11, l12, l2s]));
    let minor1 = ChildNode::new_section(Vec::from([
        l3s,
        ChildNode::new_widget(TextBlock::new("Ligma", 3)),
    ]));

    let page_root = SectionNode::new(Vec::from([
        ChildNode::new_widget(TextBlock::new("It Supposed to be the heading...", 3)),
        major1,
        minor1,
    ]));
    page_root.borrow_mut().set_show_section_outline(true);

    let mut tree = PageTree::new(Rc::clone(&page_root));
    // let _cur = CursorGuard::new();
    let _app_result = ratatui::run(|terminal| run(terminal, &mut tree));
}
