use core::fmt;
use crossterm::event::DisableMouseCapture;
use crossterm::event::EnableMouseCapture;
use crossterm::event::KeyCode;
use crossterm::event::{self, Event, KeyEvent, MouseEvent};
use crossterm::{
    cursor::{Hide, Show},
    execute,
};
use ratatui::layout::Alignment;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Wrap;
use ratatui::{DefaultTerminal, Frame};
use std::cell::RefCell;
use std::io::stdout;
use std::rc::Rc;
use std::rc::Weak;

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

enum ChildNode<'a> {
    Section(Rc<RefCell<SectionNode<'a>>>),
    Widget(Rc<RefCell<Box<dyn WidgetNode + 'a>>>),
}

impl ChildNode<'_> {
    fn get_section_length(&self) -> u16 {
        match self {
            ChildNode::Section(sec) => sec.borrow().get_total_length(),
            ChildNode::Widget(wid) => wid.borrow().get_length(),
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        match self {
            ChildNode::Section(sec) => sec.borrow().render_section(area, frame),
            ChildNode::Widget(wid) => wid.borrow().render_widget(area, frame),
        }
    }
}

impl<'a> ChildNode<'a> {
    fn new_widget<T: WidgetNode + 'a>(widget: T) -> Self {
        ChildNode::Widget(Rc::new(RefCell::new(Box::new(widget))))
    }
}

trait WidgetNode {
    fn render_widget(&self, area: Rect, frame: &mut Frame);
    fn get_length(&self) -> u16;
}

struct SectionNode<'a> {
    children: Vec<ChildNode<'a>>,
    parent: Option<Weak<RefCell<SectionNode<'a>>>>,
}

impl<'a> SectionNode<'a> {
    fn new(children: Vec<ChildNode<'a>>) -> Rc<RefCell<Self>> {
        let node = Rc::new(RefCell::new(Self {
            children: vec![],
            parent: None,
        }));
        // Add current node as parent for each children sections
        children.iter().for_each(|child| {
            if let ChildNode::Section(sec) = &child {
                sec.borrow_mut().parent = Some(Rc::downgrade(&node));
            }
        });
        node.borrow_mut().children.extend(children);
        node
    }
}

impl SectionNode<'_> {
    fn get_total_length(&self) -> u16 {
        self.children.iter().map(|c| c.get_section_length()).sum()
    }

    fn render_section(&self, area: Rect, frame: &mut Frame) {
        // Draw section outline
        let block = Block::default().borders(Borders::ALL);
        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Draw sections
        let constraints = self
            .children
            .iter()
            .map(|c| Constraint::Length(c.get_section_length()));

        Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints(constraints)
            .split(inner)
            .iter()
            .enumerate()
            .for_each(|(idx, sub_area)| self.children[idx].render(frame, *sub_area));
    }
}

struct PageTree<'a> {
    root: Rc<RefCell<SectionNode<'a>>>,
    // current_selection: Option<Weak<RefCell<SectionNode<'a>>>>,
}

impl<'a> PageTree<'a> {
    fn new(root: Rc<RefCell<SectionNode<'a>>>) -> Self {
        Self {
            root,
            // current_selection: None,
        }
    }
}

impl PageTree<'_> {
    fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        self.root.borrow().render_section(area, frame);
    }
}

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

fn run(term: &mut DefaultTerminal, tree: PageTree) -> Result<(), AppError> {
    loop {
        term.draw(|frame| {
            tree.render(frame);
        })?;

        if let Event::Key(e) = event::read()? {
            match e.code {
                KeyCode::Char('q') => break, // Exit the loop
                _ => {}
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

    let l2s = SectionNode::new(Vec::from([
        ChildNode::new_widget(TextBlock::new("At second 1", 5)),
        ChildNode::new_widget(TextBlock::new("At second 2", 5)),
    ]));

    let l3s = SectionNode::new(Vec::from([
        ChildNode::new_widget(TextBlock::new("At Third 1", 4)),
        ChildNode::new_widget(TextBlock::new("At Third 2", 5)),
        ChildNode::new_widget(TextBlock::new("At Third 3", 4)),
    ]));

    let major1 = SectionNode::new(Vec::from([l11, l12, ChildNode::Section(l2s)]));
    let minor1 = SectionNode::new(Vec::from([
        ChildNode::Section(l3s),
        ChildNode::new_widget(TextBlock::new("Ligma", 3)),
    ]));

    let page_root = SectionNode::new(Vec::from([
        ChildNode::new_widget(TextBlock::new("It Supposed to be the heading...", 3)),
        ChildNode::Section(major1),
        ChildNode::Section(minor1),
    ]));

    let tree = PageTree::new(Rc::clone(&page_root));
    // let _cur = CursorGuard::new();
    let _app_result = ratatui::run(|terminal| run(terminal, tree));
}
