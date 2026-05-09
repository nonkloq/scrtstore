use std::cell::RefCell;
use std::rc::Rc;
use std::rc::Weak;

use ratatui::Frame;

use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;

use ratatui::layout::Constraint;
use ratatui::layout::Layout;

type SectionPtr<'a> = Rc<RefCell<SectionNode<'a>>>;
type SectionPtrRef<'a> = Weak<RefCell<SectionNode<'a>>>;

type WidgetPtr<'a> = Rc<RefCell<Box<dyn WidgetNode + 'a>>>;

/// Node element
pub enum ChildNode<'a> {
    Section(SectionPtr<'a>),
    Widget(WidgetPtr<'a>),
}

/// Widget node interface
pub trait WidgetNode {
    fn render_widget(&self, area: Rect, frame: &mut Frame);
    fn get_length(&self) -> u16;
}

/// Section node that can have many nodes
pub struct SectionNode<'a> {
    show_section_outline: bool,

    children: Vec<ChildNode<'a>>,
    parent: Option<SectionPtrRef<'a>>,

    is_it_the_selected_section: bool,
    selected_child: usize,
}

/// Virtual TUI Tree
pub struct PageTree<'a> {
    root: Rc<RefCell<SectionNode<'a>>>,

    current_selection: Option<SectionPtrRef<'a>>,
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
    pub fn new_widget<T: WidgetNode + 'a>(widget: T) -> Self {
        ChildNode::Widget(Rc::new(RefCell::new(Box::new(widget))))
    }

    pub fn new_section(children: Vec<ChildNode<'a>>) -> Self {
        ChildNode::Section(SectionNode::new(children))
    }
}

impl<'a> SectionNode<'a> {
    pub fn new(children: Vec<ChildNode<'a>>) -> Rc<RefCell<Self>> {
        let node = Rc::new(RefCell::new(Self {
            show_section_outline: false,
            children: vec![],
            parent: None,
            is_it_the_selected_section: false,
            selected_child: 0,
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

    pub fn set_show_section_outline(&mut self, flag: bool) {
        self.show_section_outline = flag;
    }

    fn render_section(&self, area: Rect, frame: &mut Frame) {
        // Draw section outline
        let inner: Rect = if self.show_section_outline {
            let block = Block::default().borders(Borders::ALL);
            let inner = block.inner(area);
            frame.render_widget(block, area);
            inner
        } else {
            area
        };

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
            .for_each(|(idx, sub_area)| {
                if self.is_it_the_selected_section && idx == self.selected_child {
                    let block = Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Yellow));
                    frame.render_widget(block, *sub_area);
                }
                self.children[idx].render(frame, *sub_area)
            });
    }
}

impl<'a> PageTree<'a> {
    pub fn new(root: Rc<RefCell<SectionNode<'a>>>) -> Self {
        Self {
            root,
            current_selection: None,
        }
    }
}

impl PageTree<'_> {
    pub fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        self.root.borrow().render_section(area, frame);
    }
}

impl SectionNode<'_> {
    fn nxt_idx(&mut self, is_frwd: bool) {
        let idx = (self.selected_child as i32) + (2 * (is_frwd as i32) - 1);
        self.selected_child = idx.rem_euclid(self.children.len() as i32) as usize;
    }
}

impl<'a> SectionNode<'a> {
    fn get_section_ref(&self) -> Option<SectionPtrRef<'a>> {
        if let Some(ChildNode::Section(section)) = self.children.get(self.selected_child) {
            Some(Rc::downgrade(section))
        } else {
            None
        }
    }
}

impl<'a> PageTree<'a> {
    fn set_selected_section(&mut self, section: Option<SectionPtrRef<'a>>) {
        // Unset the selection locally to the section
        if let Some(secref) = self.current_selection.as_ref() {
            if let Some(sec) = secref.upgrade() {
                sec.borrow_mut().is_it_the_selected_section = false;
            }
        }
        if let Some(secref) = section.as_ref() {
            if let Some(sec) = secref.upgrade() {
                sec.borrow_mut().is_it_the_selected_section = true;
            }
        }

        self.current_selection = section;
    }

    fn get_selected_section(&mut self) -> Option<SectionPtr<'a>> {
        if let Some(csec) = self.current_selection.as_ref() {
            csec.upgrade()
        } else {
            None
        }
    }
}

impl PageTree<'_> {
    pub fn move_to_next(&mut self, is_frwd: bool) {
        // If nothing is selected, start from the root
        if self.current_selection.is_none() {
            return self.set_selected_section(Some(Rc::downgrade(&self.root)));
        }

        if let Some(section) = self.get_selected_section() {
            section.borrow_mut().nxt_idx(is_frwd);
        }
    }

    pub fn enter(&mut self) {
        if let Some(section) = self.get_selected_section() {
            let new_section = section.borrow().get_section_ref();
            self.set_selected_section(new_section);
        }
    }

    pub fn escape(&mut self) {
        if let Some(section) = self.get_selected_section() {
            let parent_ref = if let Some(parent) = section.borrow().parent.as_ref() {
                Some(Weak::clone(parent))
            } else {
                None
            };
            self.set_selected_section(parent_ref);
        }
    }
}
