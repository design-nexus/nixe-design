//! A ribbon: a strip of tabs, each showing a fixed-height panel of labelled
//! groups. It never grows taller as the window narrows: the least important
//! groups fold into a button each that opens them in a popover. Contextual
//! tabs (a picture's, a shape's) show only while they apply. The panel can be
//! collapsed to the strip (Ctrl+F1, or a double click on a tab); then a click
//! on a tab shows its panel until the next click.

use crate::widgets;
use gtk::glib;
use gtk::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

type OnCollapse = Box<dyn Fn(bool)>;

struct TabEntry {
    name: String,
    button: gtk::ToggleButton,
    page: gtk::Box,
    contextual: bool,
    groups: Vec<Rc<Group>>,
}

pub struct Ribbon {
    pub root: gtk::Box,
    start: gtk::Box,
    tabs_box: gtk::Box,
    end: gtk::Box,
    panel: gtk::Stack,
    tabs: RefCell<Vec<TabEntry>>,
    current: RefCell<String>,
    /// The last ordinary tab, to go back to when a contextual one closes.
    last_plain: RefCell<String>,
    collapsed: Cell<bool>,
    /// Collapsed, but showing a tab's panel for a moment.
    peeking: Cell<bool>,
    chevron: gtk::Button,
    width: Cell<i32>,
    on_collapse: RefCell<Option<OnCollapse>>,
}

/// One tab's panel: groups side by side.
pub struct Tab {
    ribbon: Weak<Ribbon>,
    index: usize,
}

/// A labelled group of controls in one or two rows. `priority` decides which
/// groups fold first when the window is narrow (lowest first).
pub struct Group {
    root: gtk::Box,
    content: gtk::Box,
    label: gtk::Label,
    fold: gtk::MenuButton,
    popover_box: gtk::Box,
    priority: i32,
    folded: Cell<bool>,
    /// The width the group takes unfolded, measured while it was.
    natural: Cell<i32>,
}

impl Ribbon {
    pub fn new() -> Rc<Ribbon> {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
        root.add_css_class("ribbon");
        let strip = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        strip.add_css_class("ribbon-strip");
        let start = widgets::hbox(2);
        let tabs_box = widgets::hbox(2);
        tabs_box.add_css_class("ribbon-tabs");
        let spacer = widgets::hbox(0);
        spacer.set_hexpand(true);
        let end = widgets::hbox(2);
        let chevron = widgets::bar_button("pan-up-symbolic", "Collapse the ribbon (Ctrl+F1)");
        strip.append(&start);
        strip.append(&tabs_box);
        strip.append(&spacer);
        strip.append(&end);
        strip.append(&chevron);
        let panel = gtk::Stack::new();
        panel.add_css_class("ribbon-panel");
        panel.set_transition_type(gtk::StackTransitionType::None);
        panel.set_hhomogeneous(false);
        panel.set_vhomogeneous(true);
        root.set_vexpand(false);
        panel.set_vexpand(false);
        root.append(&strip);
        root.append(&panel);
        let ribbon = Rc::new(Ribbon {
            root,
            start,
            tabs_box,
            end,
            panel,
            tabs: RefCell::new(Vec::new()),
            current: RefCell::new(String::new()),
            last_plain: RefCell::new(String::new()),
            collapsed: Cell::new(false),
            peeking: Cell::new(false),
            chevron,
            width: Cell::new(0),
            on_collapse: RefCell::new(None),
        });
        let me = Rc::downgrade(&ribbon);
        ribbon.chevron.connect_clicked(move |_| {
            if let Some(r) = me.upgrade() {
                r.toggle_collapsed();
            }
        });
        // Refit when the width changes: a tick is cheap, and catches every resize.
        let me = Rc::downgrade(&ribbon);
        ribbon.root.add_tick_callback(move |w, _| {
            let Some(r) = me.upgrade() else { return glib::ControlFlow::Break };
            let width = w.width();
            if width > 0 && width != r.width.get() {
                r.width.set(width);
                r.fit();
            }
            glib::ControlFlow::Continue
        });
        ribbon
    }

    /// A widget at the start of the strip, before the tabs (the File menu).
    pub fn start(&self, w: &impl IsA<gtk::Widget>) {
        self.start.append(w);
    }

    /// A widget at the end of the strip (find, the document's name).
    pub fn end(&self, w: &impl IsA<gtk::Widget>) {
        self.end.append(w);
    }

    /// Called with the new state when the ribbon collapses or expands.
    pub fn connect_collapsed(&self, f: impl Fn(bool) + 'static) {
        *self.on_collapse.borrow_mut() = Some(Box::new(f));
    }

    pub fn tab(self: &Rc<Self>, name: &str) -> Tab {
        self.add_tab(name, false)
    }

    /// A tab shown only while [`Ribbon::show_contextual`] says so.
    pub fn contextual_tab(self: &Rc<Self>, name: &str) -> Tab {
        self.add_tab(name, true)
    }

    fn add_tab(self: &Rc<Self>, name: &str, contextual: bool) -> Tab {
        let button = gtk::ToggleButton::with_label(name);
        button.add_css_class("ribbon-tab");
        if contextual {
            button.add_css_class("contextual");
            button.set_visible(false);
        }
        let page = widgets::hbox(0);
        page.add_css_class("ribbon-page");
        // The panel asks for no width of its own, so the window can narrow
        // and the groups fold instead.
        let holder = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::External)
            .vscrollbar_policy(gtk::PolicyType::Never)
            .child(&page)
            .build();
        self.panel.add_named(&holder, Some(name));
        let n = name.to_string();
        let me = Rc::downgrade(self);
        button.connect_clicked(move |b| {
            let Some(r) = me.upgrade() else { return };
            if r.collapsed.get() {
                // Collapsed: a click shows this tab's panel, a second hides it.
                let same = *r.current.borrow() == n;
                r.peeking.set(!(same && r.peeking.get()));
                r.panel.set_visible(r.peeking.get());
            }
            r.select(&n);
            b.set_active(true);
        });
        // A double click collapses or expands.
        let click = gtk::GestureClick::new();
        let me = Rc::downgrade(self);
        click.connect_pressed(move |_, n_press, _, _| {
            if n_press == 2
                && let Some(r) = me.upgrade()
            {
                r.toggle_collapsed();
            }
        });
        button.add_controller(click);
        self.tabs_box.append(&button);
        let index = {
            let mut tabs = self.tabs.borrow_mut();
            tabs.push(TabEntry { name: name.to_string(), button, page, contextual, groups: Vec::new() });
            tabs.len() - 1
        };
        if self.current.borrow().is_empty() && !contextual {
            self.select(name);
        }
        Tab { ribbon: Rc::downgrade(self), index }
    }

    /// Show a tab's panel.
    pub fn select(&self, name: &str) {
        *self.current.borrow_mut() = name.to_string();
        self.panel.set_visible_child_name(name);
        for t in self.tabs.borrow().iter() {
            t.button.set_active(t.name == name);
            if t.name == name && !t.contextual {
                *self.last_plain.borrow_mut() = name.to_string();
            }
        }
        self.fit();
    }

    /// Show or hide a contextual tab. Showing one switches to it; hiding the
    /// one on screen goes back to the last ordinary tab.
    pub fn show_contextual(&self, name: &str, on: bool) {
        let changed = {
            let tabs = self.tabs.borrow();
            let Some(t) = tabs.iter().find(|t| t.name == name && t.contextual) else { return };
            let changed = t.button.is_visible() != on;
            t.button.set_visible(on);
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

    pub fn is_collapsed(&self) -> bool {
        self.collapsed.get()
    }

    pub fn set_collapsed(&self, collapsed: bool) {
        self.collapsed.set(collapsed);
        self.peeking.set(false);
        self.panel.set_visible(!collapsed);
        if collapsed {
            self.root.add_css_class("collapsed");
        } else {
            self.root.remove_css_class("collapsed");
        }
        self.chevron.set_icon_name(if collapsed { "pan-down-symbolic" } else { "pan-up-symbolic" });
        self.chevron.set_tooltip_text(Some(if collapsed {
            "Expand the ribbon (Ctrl+F1)"
        } else {
            "Collapse the ribbon (Ctrl+F1)"
        }));
        if let Some(f) = self.on_collapse.borrow().as_ref() {
            f(collapsed);
        }
    }

    pub fn toggle_collapsed(&self) {
        self.set_collapsed(!self.collapsed.get());
    }

    /// While collapsed, hide a panel shown by a click on its tab.
    pub fn end_peek(&self) {
        if self.collapsed.get() && self.peeking.get() {
            self.peeking.set(false);
            self.panel.set_visible(false);
        }
    }

    /// Fold the current tab's groups, least important first, until they fit.
    fn fit(&self) {
        let width = self.root.width();
        if width <= 0 {
            return;
        }
        let tabs = self.tabs.borrow();
        let Some(tab) = tabs.iter().find(|t| t.name == *self.current.borrow()) else { return };
        let mut groups: Vec<&Rc<Group>> = tab.groups.iter().filter(|g| g.root.is_visible()).collect();
        // Measure unfolded groups now; folded ones keep their last measure.
        for g in &groups {
            if !g.folded.get() {
                g.natural.set(g.root.measure(gtk::Orientation::Horizontal, -1).1);
            }
        }
        // A hidden fold button measures nothing, so allow for its label.
        let folded_width = |g: &Group| g.fold.measure(gtk::Orientation::Horizontal, -1).1.max(64) + 18;
        let available = width - 32;
        groups.sort_by_key(|g| g.priority);
        let mut total: i32 = groups.iter().map(|g| g.natural.get()).sum();
        let mut fold = vec![false; groups.len()];
        for (i, g) in groups.iter().enumerate() {
            if total <= available {
                break;
            }
            total -= g.natural.get() - folded_width(g);
            fold[i] = true;
        }
        for (g, f) in groups.iter().zip(fold) {
            g.set_folded(f);
        }
    }
}

impl Tab {
    /// A new group at the end of this tab, called `label`.
    pub fn group(&self, label: &str, icon: &str, priority: i32) -> Rc<Group> {
        let Some(ribbon) = self.ribbon.upgrade() else { panic!("the ribbon is gone") };
        let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
        root.add_css_class("ribbon-group");
        let content = gtk::Box::new(gtk::Orientation::Vertical, 2);
        content.add_css_class("ribbon-group-content");
        content.set_valign(gtk::Align::Center);
        let name = widgets::label(label, "ribbon-group-label");
        name.set_xalign(0.5);
        let fold = gtk::MenuButton::new();
        fold.add_css_class("ribbon-fold");
        let fold_content = widgets::vbox(2);
        fold_content.append(&gtk::Image::from_icon_name(icon));
        fold_content.append(&widgets::label(label, "ribbon-fold-label"));
        fold.set_child(Some(&fold_content));
        fold.set_tooltip_text(Some(label));
        fold.set_visible(false);
        let popover = gtk::Popover::new();
        popover.add_css_class("ribbon-popover");
        let popover_box = widgets::vbox(0);
        popover.set_child(Some(&popover_box));
        fold.set_popover(Some(&popover));
        root.append(&content);
        root.append(&fold);
        root.append(&name);
        let group = Rc::new(Group {
            root,
            content,
            label: name,
            fold,
            popover_box,
            priority,
            folded: Cell::new(false),
            natural: Cell::new(0),
        });
        let mut tabs = ribbon.tabs.borrow_mut();
        let tab = &mut tabs[self.index];
        tab.page.append(&group.root);
        tab.groups.push(group.clone());
        group
    }
}

impl Group {
    /// A row of controls in this group (a group has one or two).
    pub fn row(&self, items: &[&gtk::Widget]) -> gtk::Box {
        let row = widgets::hbox(2);
        row.add_css_class("ribbon-row");
        for w in items {
            if let Some(parent) = w.parent().and_downcast::<gtk::Box>() {
                parent.remove(*w);
            }
            row.append(*w);
        }
        self.content.append(&row);
        row
    }

    /// The whole group, to show or hide it (a contextual group within a tab).
    pub fn widget(&self) -> &gtk::Box {
        &self.root
    }

    fn set_folded(&self, folded: bool) {
        if self.folded.get() == folded {
            return;
        }
        self.folded.set(folded);
        // The controls move into the popover, and back.
        if folded {
            self.root.remove(&self.content);
            self.popover_box.append(&self.content);
        } else {
            self.popover_box.remove(&self.content);
            self.root.prepend(&self.content);
        }
        self.fold.set_visible(folded);
        self.label.set_visible(!folded);
    }
}

/// A large button: an icon over its name, for a group's main action.
pub fn large_button(icon: &str, label: &str, tooltip: &str) -> gtk::Button {
    let b = gtk::Button::new();
    b.add_css_class("ribbon-large");
    let content = widgets::vbox(2);
    let image = gtk::Image::from_icon_name(icon);
    image.set_pixel_size(20);
    content.append(&image);
    content.append(&widgets::label(label, "ribbon-large-label"));
    b.set_child(Some(&content));
    b.set_tooltip_text(Some(tooltip));
    b
}
