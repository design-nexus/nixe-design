//! The Inspector: a panel at the right of a document with the formatting for
//! what's selected, in labelled sections. A segmented switch at its top picks
//! a segment (Text · Page · Review). Contextual segments (a picture's, a
//! shape's) show only while they apply, in the accent colour, and are picked
//! while they do.
//!
//! [`Inspector::wrap`] puts it beside the document. In a narrow window it
//! floats over the document instead of squeezing it, and Esc closes it.

use crate::widgets;
use gtk::glib;
use gtk::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

/// Its width, docked.
const WIDTH: i32 = 288;

type OnOpen = Rc<dyn Fn(bool)>;

struct SegEntry {
    name: String,
    button: gtk::ToggleButton,
    contextual: bool,
}

pub struct Inspector {
    /// Slides in and out; holds the panel.
    pub root: gtk::Revealer,
    panel: gtk::Box,
    segs: gtk::Box,
    stack: gtk::Stack,
    entries: RefCell<Vec<SegEntry>>,
    current: RefCell<String>,
    /// The last ordinary segment, to go back to when a contextual one closes.
    last_plain: RefCell<String>,
    toggle: gtk::ToggleButton,
    open: Cell<bool>,
    /// Hidden for now (focus mode) without changing whether it's open.
    suspended: Cell<bool>,
    floating: Cell<bool>,
    dock: gtk::Box,
    float_slot: RefCell<Option<gtk::Overlay>>,
    on_open: RefCell<Vec<OnOpen>>,
    on_floating: RefCell<Vec<OnOpen>>,
    syncing: Cell<bool>,
}

/// One segment's page: sections, top to bottom.
pub struct Segment {
    page: gtk::Box,
}

/// A labelled section in a segment.
pub struct Section {
    pub root: gtk::Box,
}

thread_local! {
    static ALL: RefCell<Vec<Weak<Inspector>>> = const { RefCell::new(Vec::new()) };
}

impl Inspector {
    pub fn new() -> Rc<Inspector> {
        let panel = widgets::vbox(0);
        panel.add_css_class("inspector");
        let head = widgets::hbox(0);
        head.add_css_class("inspector-head");
        let segs = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        segs.add_css_class("inspector-segments");
        segs.set_homogeneous(true);
        segs.set_hexpand(true);
        head.append(&segs);
        let stack = gtk::Stack::new();
        stack.set_transition_type(gtk::StackTransitionType::None);
        stack.set_vhomogeneous(false);
        stack.set_hhomogeneous(true);
        let scroll = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .vexpand(true)
            .propagate_natural_height(true)
            .child(&stack)
            .build();
        panel.append(&head);
        panel.append(&scroll);
        let root = gtk::Revealer::new();
        root.set_transition_type(gtk::RevealerTransitionType::SlideLeft);
        root.set_transition_duration(150);
        root.set_child(Some(&panel));
        root.set_hexpand(false);
        let toggle = gtk::ToggleButton::new();
        toggle.set_icon_name("sidebar-show-right-symbolic");
        toggle.set_tooltip_text(Some("Inspector: formatting for the selection (Ctrl+Alt+I)"));
        let dock = widgets::hbox(0);
        let insp = Rc::new(Inspector {
            root,
            panel,
            segs,
            stack,
            entries: RefCell::new(Vec::new()),
            current: RefCell::new(String::new()),
            last_plain: RefCell::new(String::new()),
            toggle,
            open: Cell::new(false),
            suspended: Cell::new(false),
            floating: Cell::new(false),
            dock,
            float_slot: RefCell::new(None),
            on_open: RefCell::new(Vec::new()),
            on_floating: RefCell::new(Vec::new()),
            syncing: Cell::new(false),
        });
        let me = Rc::downgrade(&insp);
        insp.toggle.connect_toggled(move |b| {
            if let Some(i) = me.upgrade()
                && !i.syncing.get()
            {
                i.set_open(b.is_active());
            }
        });
        ALL.with(|a| a.borrow_mut().push(Rc::downgrade(&insp)));
        insp
    }

    /// The button that shows and hides it, for the command bar.
    pub fn toggle_button(&self) -> &gtk::ToggleButton {
        &self.toggle
    }

    /// `content` with the inspector beside it (or floating over it when narrow).
    pub fn wrap(self: &Rc<Self>, content: &impl IsA<gtk::Widget>) -> gtk::Overlay {
        let overlay = gtk::Overlay::new();
        overlay.add_css_class("inspector-host");
        let row = widgets::hbox(0);
        content.set_hexpand(true);
        row.append(content);
        row.append(&self.dock);
        overlay.set_child(Some(&row));
        overlay.set_vexpand(true);
        *self.float_slot.borrow_mut() = Some(overlay.clone());
        // It starts floating, so it never makes the window wider than asked;
        // it docks once the document leaves room for it.
        self.floating.set(false);
        self.dock.append(&self.root);
        self.set_floating(true);
        let me = Rc::downgrade(self);
        let content = content.clone().upcast::<gtk::Widget>();
        // The first measure is always reported: it may float from the start.
        let first = Cell::new(true);
        overlay.add_tick_callback(move |o, _| {
            let Some(i) = me.upgrade() else { return glib::ControlFlow::Break };
            let width = o.width();
            if width > 0 {
                let least = content.measure(gtk::Orientation::Horizontal, -1).0;
                if i.set_floating(width - least < WIDTH) | first.replace(false) {
                    let hooks = i.on_floating.borrow().clone();
                    for f in hooks {
                        f(i.floating.get());
                    }
                }
            }
            glib::ControlFlow::Continue
        });
        overlay
    }

    /// Returns whether it changed.
    fn set_floating(&self, floating: bool) -> bool {
        if self.floating.get() == floating {
            return false;
        }
        self.floating.set(floating);
        let Some(overlay) = self.float_slot.borrow().clone() else { return true };
        if floating {
            self.dock.remove(&self.root);
            // As tall as what it holds, up to the document's height.
            self.root.set_halign(gtk::Align::End);
            self.root.set_valign(gtk::Align::Start);
            self.panel.add_css_class("floating");
            overlay.add_overlay(&self.root);
        } else {
            overlay.remove_overlay(&self.root);
            self.panel.remove_css_class("floating");
            self.root.set_valign(gtk::Align::Fill);
            self.dock.append(&self.root);
        }
        true
    }

    /// Called when it starts or stops floating as the space beside the
    /// document changes, so the window can make room (a narrower sidebar).
    pub fn connect_floating(&self, f: impl Fn(bool) + 'static) {
        self.on_floating.borrow_mut().push(Rc::new(f));
    }

    pub fn is_floating(&self) -> bool {
        self.floating.get()
    }

    /// Called with the new state whenever it opens or closes.
    pub fn connect_open(&self, f: impl Fn(bool) + 'static) {
        self.on_open.borrow_mut().push(Rc::new(f));
    }

    pub fn is_open(&self) -> bool {
        self.open.get()
    }

    pub fn set_open(&self, open: bool) {
        let changed = self.open.get() != open;
        self.open.set(open);
        self.refresh();
        if changed {
            let hooks = self.on_open.borrow().clone();
            for f in hooks {
                f(open);
            }
        }
    }

    pub fn toggle(&self) {
        self.set_open(!self.open.get());
    }

    /// Hide it for now (focus mode), or bring it back as it was.
    pub fn suspend(&self, on: bool) {
        self.suspended.set(on);
        self.refresh();
    }

    fn refresh(&self) {
        let shown = self.open.get() && !self.suspended.get();
        self.root.set_reveal_child(shown);
        self.root.set_can_target(shown);
        self.syncing.set(true);
        self.toggle.set_active(self.open.get());
        self.syncing.set(false);
    }

    /// Esc: a floating inspector closes. Returns whether it did.
    pub fn escape(&self) -> bool {
        if self.floating.get() && self.open.get() && !self.suspended.get() {
            self.set_open(false);
            return true;
        }
        false
    }

    pub fn segment(self: &Rc<Self>, name: &str) -> Segment {
        self.add_segment(name, false)
    }

    /// A segment shown only while [`Inspector::show_contextual`] says so.
    pub fn contextual_segment(self: &Rc<Self>, name: &str) -> Segment {
        self.add_segment(name, true)
    }

    fn add_segment(self: &Rc<Self>, name: &str, contextual: bool) -> Segment {
        let button = gtk::ToggleButton::with_label(name);
        button.add_css_class("inspector-segment");
        if contextual {
            button.add_css_class("contextual");
            button.set_visible(false);
        }
        let n = name.to_string();
        let me = Rc::downgrade(self);
        button.connect_clicked(move |b| {
            if let Some(i) = me.upgrade() {
                i.select(&n);
                b.set_active(true);
            }
        });
        self.segs.append(&button);
        let page = widgets::vbox(0);
        page.add_css_class("inspector-page");
        if contextual {
            // Hidden with its segment, so the palette leaves its controls out.
            page.add_css_class("contextual-page");
            page.set_visible(false);
        }
        self.stack.add_named(&page, Some(name));
        self.entries.borrow_mut().push(SegEntry { name: name.to_string(), button, contextual });
        if self.current.borrow().is_empty() && !contextual {
            self.select(name);
        }
        Segment { page }
    }

    pub fn select(&self, name: &str) {
        *self.current.borrow_mut() = name.to_string();
        self.stack.set_visible_child_name(name);
        for e in self.entries.borrow().iter() {
            e.button.set_active(e.name == name);
            if e.name == name && !e.contextual {
                *self.last_plain.borrow_mut() = name.to_string();
            }
        }
    }

    /// Show or hide a contextual segment. Showing one picks it; hiding the one
    /// on screen goes back to the last ordinary segment.
    pub fn show_contextual(&self, name: &str, on: bool) {
        let changed = {
            let entries = self.entries.borrow();
            let Some(e) = entries.iter().find(|e| e.name == name && e.contextual) else { return };
            let changed = e.button.is_visible() != on;
            e.button.set_visible(on);
            if let Some(page) = self.stack.child_by_name(name) {
                page.set_visible(on);
            }
            changed
        };
        if !changed {
            return;
        }
        if on {
            self.select(name);
        } else if *self.current.borrow() == name {
            let back = self.last_plain.borrow().clone();
            self.select(&back);
        }
    }

    /// Open it at the segment holding `w`, if `w` is in it.
    fn reveal(&self, w: &gtk::Widget) -> bool {
        if !w.is_ancestor(&self.panel) {
            return false;
        }
        let mut page = None;
        let mut cur = Some(w.clone());
        while let Some(c) = cur {
            if c.parent().is_some_and(|p| p == *self.stack.upcast_ref::<gtk::Widget>()) {
                page = Some(c);
                break;
            }
            cur = c.parent();
        }
        if let Some(name) = page.and_then(|p| self.stack.page(&p).name()) {
            self.select(&name);
        }
        if !self.open.get() {
            self.set_open(true);
        }
        self.suspend(false);
        true
    }
}

/// Before the palette presses `w`: open the inspector it's in, at its segment.
pub fn reveal(w: &gtk::Widget) {
    ALL.with(|a| {
        a.borrow_mut().retain(|i| i.strong_count() > 0);
        for i in a.borrow().iter().filter_map(Weak::upgrade) {
            if i.reveal(w) {
                break;
            }
        }
    });
}

impl Segment {
    pub fn section(&self, title: &str) -> Section {
        let root = widgets::vbox(8);
        root.add_css_class("inspector-section");
        root.append(&widgets::label(&title.to_uppercase(), "inspector-heading"));
        self.page.append(&root);
        Section { root }
    }
}

impl Section {
    /// A control the width of the section.
    pub fn add(&self, w: &impl IsA<gtk::Widget>) -> &Self {
        let w = w.upcast_ref::<gtk::Widget>();
        detach(w);
        fill(w);
        self.root.append(w);
        self
    }

    /// Controls side by side, sharing the width equally.
    pub fn row(&self, items: &[&gtk::Widget]) -> gtk::Box {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        row.set_homogeneous(true);
        for w in items {
            detach(w);
            fill(w);
            row.append(*w);
        }
        self.root.append(&row);
        row
    }

    /// Controls side by side, the first taking what the others leave.
    pub fn row_wide_first(&self, items: &[&gtk::Widget]) -> gtk::Box {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        for (i, w) in items.iter().enumerate() {
            detach(w);
            if i == 0 {
                fill(w);
            } else {
                w.set_hexpand(false);
                tidy(w);
            }
            row.append(*w);
        }
        self.root.append(&row);
        row
    }

    /// Buttons joined into one segmented row (B I U, the alignments).
    pub fn segmented(&self, items: &[&gtk::Widget]) -> gtk::Box {
        let row = segmented(items);
        self.root.append(&row);
        row
    }

    /// A label on the left and a control filling the rest of the row.
    pub fn field(&self, label: &str, w: &impl IsA<gtk::Widget>) -> &Self {
        let w = w.upcast_ref::<gtk::Widget>();
        detach(w);
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        row.add_css_class("inspector-field");
        let l = widgets::label(label, "inspector-field-label");
        l.set_xalign(0.0);
        l.set_size_request(84, -1);
        row.append(&l);
        fill(w);
        row.append(w);
        self.root.append(&row);
        self
    }

    /// A line of dim text under the controls.
    pub fn note(&self, text: &str) -> gtk::Label {
        let l = widgets::label(text, "inspector-note");
        l.set_wrap(true);
        l.set_xalign(0.0);
        self.root.append(&l);
        l
    }
}

/// Buttons joined into one segmented row.
pub fn segmented(items: &[&gtk::Widget]) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    row.add_css_class("inspector-linked");
    row.set_homogeneous(true);
    row.set_hexpand(true);
    for w in items {
        detach(w);
        tidy(w);
        w.set_hexpand(true);
        row.append(*w);
    }
    row
}

/// Take `w` out of the box it was made in (a toolbar group).
fn detach(w: &gtk::Widget) {
    if let Some(p) = w.parent().and_downcast::<gtk::Box>() {
        p.remove(w);
    }
}

fn fill(w: &gtk::Widget) {
    w.set_hexpand(true);
    w.set_halign(gtk::Align::Fill);
    tidy(w);
    if w.is::<gtk::Button>() || w.is::<gtk::MenuButton>() {
        w.add_css_class("inspector-button");
    }
}

/// Drop the toolbar looks a control was made with.
fn tidy(w: &gtk::Widget) {
    for c in ["bar-button", "tool-toggle", "tool-group", "flat"] {
        w.remove_css_class(c);
    }
    w.set_valign(gtk::Align::Fill);
}

/// Give a toolbar button or menu button a label beside its icon, centred.
pub fn labelled(w: &impl IsA<gtk::Widget>, icon: &str, label: &str) {
    let w = w.upcast_ref::<gtk::Widget>();
    let b = widgets::hbox(7);
    b.set_halign(gtk::Align::Center);
    b.append(&gtk::Image::from_icon_name(icon));
    b.append(&widgets::label(label, ""));
    if let Some(m) = w.downcast_ref::<gtk::MenuButton>() {
        m.set_always_show_arrow(false);
        m.set_child(Some(&b));
    } else if let Some(x) = w.downcast_ref::<gtk::Button>() {
        x.set_child(Some(&b));
    }
}

/// A button with an icon and a label, for a section.
pub fn button(icon: &str, label: &str, tooltip: &str) -> gtk::Button {
    let b = gtk::Button::new();
    labelled(&b, icon, label);
    b.set_tooltip_text(Some(tooltip));
    b
}
