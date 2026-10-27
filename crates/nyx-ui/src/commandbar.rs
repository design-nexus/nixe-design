//! A command bar: one 44px row of icons over a document, for what you use most
//! (File, undo, the everyday formatting, inserting). Everything else lives in
//! the [`crate::inspector`].
//!
//! ```text
//! File▾ ↶ ↷ │ B I U │ ≡ ≡ ≡ │ ▣ ⊞ ⋯        Name.ext •        end… ◨
//! ```
//!
//! Tools come in groups ([`CommandBar::group`]), split by short dividers. It
//! never wraps or grows taller: when the window narrows, whole groups fold into
//! the `⋯` menu, least important first, where each tool shows its name and
//! keys. Tools added with [`CommandBar::more`] always live in that menu.
//!
//! A control that also sits in the Inspector is mirrored onto the bar
//! ([`CommandBar::toggle_of`], [`CommandBar::button_of`]): the bar's copy
//! shows the Inspector one's state and clicks it.

use crate::widgets;
use gtk::glib;
use gtk::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// An icon tool's width, with the gap after it.
const ICON: i32 = 32;
/// A divider with its margins.
const DIVIDER: i32 = 14;

struct Tool {
    widget: gtk::Widget,
    label: gtk::Label,
    keys: gtk::Label,
    /// Its tooltip, shown on the bar (a menu row has its name instead).
    tip: String,
}

struct Group {
    priority: i32,
    menu_only: bool,
    /// On the bar: a divider, then the tools.
    bar_part: gtk::Box,
    /// In the menu: a separator, then the tools as rows.
    menu_part: gtk::Box,
    tools: RefCell<Vec<Tool>>,
    folded: Cell<bool>,
}

impl Group {
    fn width(&self) -> i32 {
        DIVIDER + self.tools.borrow().len() as i32 * ICON
    }

    fn set_folded(&self, folded: bool) {
        if self.folded.get() == folded {
            return;
        }
        self.folded.set(folded);
        let (from, to) = if folded { (&self.bar_part, &self.menu_part) } else { (&self.menu_part, &self.bar_part) };
        for t in self.tools.borrow().iter() {
            from.remove(&t.widget);
            to.append(&t.widget);
            show_as_row(t, folded);
        }
        self.bar_part.set_visible(!folded);
        self.menu_part.set_visible(folded);
    }
}

pub struct CommandBar {
    pub root: gtk::CenterBox,
    lead: gtk::Box,
    tools_box: gtk::Box,
    more: gtk::MenuButton,
    menu_box: gtk::Box,
    end: gtk::Box,
    title: gtk::Button,
    name: gtk::Label,
    ext: gtk::Label,
    dot: gtk::Box,
    groups: RefCell<Vec<Rc<Group>>>,
    width: Cell<i32>,
}

impl CommandBar {
    pub fn new() -> Rc<CommandBar> {
        let root = gtk::CenterBox::new();
        root.add_css_class("commandbar");
        let start = widgets::hbox(0);
        let lead = widgets::hbox(2);
        let tools_box = widgets::hbox(0);
        tools_box.add_css_class("commandbar-tools");
        let more = gtk::MenuButton::new();
        more.set_icon_name("view-more-horizontal-symbolic");
        more.add_css_class("commandbar-tool");
        more.add_css_class("icon-only");
        more.set_tooltip_text(Some("More"));
        more.set_valign(gtk::Align::Center);
        more.set_margin_start(2);
        more.set_visible(false);
        let pop = gtk::Popover::new();
        pop.add_css_class("commandbar-menu");
        pop.set_has_arrow(false);
        let menu_box = widgets::vbox(0);
        pop.set_child(Some(&menu_box));
        more.set_popover(Some(&pop));
        start.append(&lead);
        start.append(&tools_box);
        start.append(&more);

        let title = gtk::Button::new();
        title.add_css_class("commandbar-title");
        let t = widgets::hbox(0);
        let name = widgets::label("", "commandbar-name");
        name.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
        name.set_max_width_chars(28);
        let ext = widgets::label("", "commandbar-ext");
        let dot = widgets::hbox(0);
        dot.add_css_class("commandbar-dot");
        dot.set_valign(gtk::Align::Center);
        dot.set_visible(false);
        t.append(&name);
        t.append(&ext);
        t.append(&dot);
        title.set_child(Some(&t));
        title.set_visible(false);

        let end = widgets::hbox(2);
        // The start asks for no width of its own, so the window can narrow and
        // the groups fold instead.
        let holder = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::External)
            .vscrollbar_policy(gtk::PolicyType::Never)
            .propagate_natural_width(true)
            .child(&start)
            .build();
        root.set_start_widget(Some(&holder));
        root.set_center_widget(Some(&title));
        root.set_end_widget(Some(&end));

        let bar = Rc::new(CommandBar {
            root,
            lead,
            tools_box,
            more,
            menu_box,
            end,
            title,
            name,
            ext,
            dot,
            groups: RefCell::new(Vec::new()),
            width: Cell::new(0),
        });
        // A menu row closes the menu once it has done its job.
        let pop2 = pop.clone();
        let click = gtk::GestureClick::new();
        click.connect_released(move |_, _, _, _| {
            let p = pop2.clone();
            glib::idle_add_local_once(move || {
                // A row that opened a menu of its own keeps this one open.
                if !has_open_popover(p.upcast_ref()) {
                    p.popdown();
                }
            });
        });
        bar.menu_box.add_controller(click);
        let me = Rc::downgrade(&bar);
        bar.root.add_tick_callback(move |w, _| {
            let Some(b) = me.upgrade() else { return glib::ControlFlow::Break };
            let width = w.width();
            if width > 0 && width != b.width.get() {
                b.width.set(width);
                b.fit();
            }
            glib::ControlFlow::Continue
        });
        bar
    }

    /// At the start, before the groups: the File menu, undo and redo.
    pub fn lead(&self, items: &[&gtk::Widget]) {
        for w in items {
            detach(w);
            if !w.has_css_class("commandbar-file") && (w.is::<gtk::Button>() || w.is::<gtk::MenuButton>()) {
                tidy(w);
                w.add_css_class("commandbar-tool");
                w.add_css_class("icon-only");
            }
            self.lead.append(*w);
        }
    }

    /// The bar's main action (at most one): an accent square after the lead.
    pub fn primary(&self, widget: &impl IsA<gtk::Widget>, icon: &str) {
        let w = widget.upcast_ref::<gtk::Widget>();
        detach(w);
        tidy(w);
        set_icon(w, icon);
        w.add_css_class("commandbar-primary");
        w.set_margin_start(DIVIDER / 2);
        self.lead.append(w);
    }

    /// Start a group of tools. Lower `priority` folds into `⋯` first.
    pub fn group(&self, priority: i32) {
        self.add_group(priority, false);
    }

    fn add_group(&self, priority: i32, menu_only: bool) -> Rc<Group> {
        let bar_part = widgets::hbox(2);
        bar_part.append(&divider());
        let menu_part = widgets::vbox(0);
        menu_part.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        menu_part.set_visible(menu_only);
        self.menu_box.append(&menu_part);
        if !menu_only {
            self.tools_box.append(&bar_part);
        }
        let g = Rc::new(Group {
            priority,
            menu_only,
            bar_part,
            menu_part,
            tools: RefCell::new(Vec::new()),
            folded: Cell::new(menu_only),
        });
        self.groups.borrow_mut().push(g.clone());
        g
    }

    /// A tool in the current group: `widget` (a button or menu button) shows
    /// `icon`, and `label` in the `⋯` menu.
    pub fn tool(&self, widget: &impl IsA<gtk::Widget>, icon: &str, label: &str) {
        let g = self.groups.borrow().iter().rev().find(|g| !g.menu_only).cloned();
        let g = g.unwrap_or_else(|| self.add_group(0, false));
        self.add(&g, widget.upcast_ref(), icon, label);
    }

    /// A tool that always lives in the `⋯` menu.
    pub fn more(&self, widget: &impl IsA<gtk::Widget>, icon: &str, label: &str) {
        let g = self.groups.borrow().iter().find(|g| g.menu_only).cloned();
        let g = g.unwrap_or_else(|| self.add_group(i32::MAX, true));
        self.add(&g, widget.upcast_ref(), icon, label);
        self.more.set_visible(true);
    }

    /// A copy of the Inspector's `src` toggle in the current group: it shows
    /// `src`'s state and clicking it clicks `src`.
    pub fn toggle_of(&self, src: &gtk::ToggleButton, icon: &str, label: &str) -> gtk::ToggleButton {
        let b = gtk::ToggleButton::new();
        b.set_tooltip_text(src.tooltip_text().as_deref());
        src.bind_property("active", &b, "active").sync_create().build();
        src.bind_property("sensitive", &b, "sensitive").sync_create().build();
        let s = src.clone();
        // The click flips this copy; clicking `src` flips it, and the binding
        // then sets this one to match.
        b.connect_clicked(move |_| s.emit_clicked());
        self.tool(&b, icon, label);
        b
    }

    /// A copy of the Inspector's `src` button in the current group.
    pub fn button_of(&self, src: &gtk::Button, icon: &str, label: &str) -> gtk::Button {
        let b = gtk::Button::new();
        b.set_tooltip_text(src.tooltip_text().as_deref());
        src.bind_property("sensitive", &b, "sensitive").sync_create().build();
        let s = src.clone();
        b.connect_clicked(move |_| s.emit_clicked());
        self.tool(&b, icon, label);
        b
    }

    /// A widget at the end of the bar (Play, Find, the Inspector toggle).
    pub fn end(&self, w: &impl IsA<gtk::Widget>) {
        let w = w.upcast_ref::<gtk::Widget>();
        detach(w);
        if (w.is::<gtk::Button>() || w.is::<gtk::MenuButton>()) && !w.has_css_class("commandbar-split") {
            tidy(w);
            w.add_css_class("commandbar-tool");
            w.add_css_class("icon-only");
        }
        self.end.append(w);
    }

    /// The document's name in the middle: `name` with its extension dimmed and
    /// a dot while there are unsaved changes. Hidden while `name` is empty.
    pub fn set_title(&self, name: &str, ext: &str, dirty: bool) {
        self.name.set_text(name);
        self.ext.set_text(ext);
        self.dot.set_visible(dirty);
        self.dot.set_tooltip_text(dirty.then_some("Unsaved changes"));
        self.title.set_visible(!name.is_empty());
        self.fit();
    }

    /// The title button, to give it a click (Save as) and a tooltip.
    pub fn title_button(&self) -> &gtk::Button {
        &self.title
    }

    fn add(&self, g: &Group, w: &gtk::Widget, icon: &str, label: &str) {
        detach(w);
        tidy(w);
        let tip = w.tooltip_text().map(|t| t.to_string()).unwrap_or_default();
        let b = widgets::hbox(10);
        b.set_halign(gtk::Align::Fill);
        let image = gtk::Image::from_icon_name(icon);
        image.set_halign(gtk::Align::Center);
        b.append(&image);
        let l = widgets::label(label, "commandbar-label");
        l.set_xalign(0.0);
        l.set_hexpand(true);
        b.append(&l);
        let keys = widgets::label(&crate::palette::split_keys(&tip).1, "commandbar-keys");
        b.append(&keys);
        set_child(w, &b);
        w.add_css_class("commandbar-tool");
        let tool = Tool { widget: w.clone(), label: l, keys, tip };
        if g.folded.get() {
            g.menu_part.append(w);
        } else {
            g.bar_part.append(w);
        }
        show_as_row(&tool, g.folded.get());
        g.tools.borrow_mut().push(tool);
    }

    /// Fold the least important groups into `⋯` until the rest fit.
    fn fit(&self) {
        let width = self.root.width();
        if width <= 0 {
            return;
        }
        let nat = |w: &gtk::Widget| if w.is_visible() { w.measure(gtk::Orientation::Horizontal, -1).1 } else { 0 };
        let title = nat(self.title.upcast_ref());
        let end = nat(self.end.upcast_ref());
        // The title is centred while there's room, and moves aside rather
        // than push tools into the menu.
        let available = (width - title - end - 40).max(0) - nat(self.lead.upcast_ref());
        let groups = self.groups.borrow().clone();
        let bar_groups: Vec<&Rc<Group>> = groups.iter().filter(|g| !g.menu_only).collect();
        let more_always = groups.iter().any(|g| g.menu_only && !g.tools.borrow().is_empty());
        let mut total: i32 = bar_groups.iter().map(|g| g.width()).sum::<i32>() + if more_always { ICON } else { 0 };
        let mut order: Vec<usize> = (0..bar_groups.len()).collect();
        // Least important first; among equals, the rightmost.
        order.sort_by_key(|&i| (bar_groups[i].priority, std::cmp::Reverse(i)));
        let mut fold = vec![false; bar_groups.len()];
        for i in order {
            if total <= available {
                break;
            }
            if !more_always && !fold.iter().any(|f| *f) {
                total += ICON;
            }
            total -= bar_groups[i].width();
            fold[i] = true;
        }
        for (g, f) in bar_groups.iter().zip(&fold) {
            g.set_folded(*f);
        }
        // The menu's first section needs no separator above it.
        let mut first = true;
        for g in &groups {
            if let Some(sep) = g.menu_part.first_child() {
                sep.set_visible(!first);
            }
            if g.folded.get() && !g.tools.borrow().is_empty() {
                first = false;
            }
        }
        self.more.set_visible(more_always || fold.iter().any(|f| *f));
    }
}

fn divider() -> gtk::Box {
    let d = widgets::hbox(0);
    d.add_css_class("commandbar-divider");
    d.set_valign(gtk::Align::Center);
    d
}

/// On the bar a tool shows only its icon; in the menu, its name and keys too.
fn show_as_row(t: &Tool, row: bool) {
    t.label.set_visible(row);
    t.keys.set_visible(row && !t.keys.text().is_empty());
    if row {
        t.widget.remove_css_class("icon-only");
        t.widget.add_css_class("in-menu");
        t.widget.set_tooltip_text(None);
    } else {
        t.widget.add_css_class("icon-only");
        t.widget.remove_css_class("in-menu");
        t.widget.set_tooltip_text((!t.tip.is_empty()).then_some(t.tip.as_str()));
    }
    if let Some(b) = t
        .widget
        .downcast_ref::<gtk::Button>()
        .and_then(|b| b.child())
        .or_else(|| t.widget.downcast_ref::<gtk::MenuButton>().and_then(|m| m.child()))
    {
        b.set_halign(if row { gtk::Align::Fill } else { gtk::Align::Center });
    }
}

/// Take `w` out of the box it was made in (a toolbar group).
fn detach(w: &gtk::Widget) {
    if let Some(p) = w.parent().and_downcast::<gtk::Box>() {
        p.remove(w);
    }
    w.set_hexpand(false);
    w.set_valign(gtk::Align::Center);
}

/// Drop the toolbar looks a control was made with.
fn tidy(w: &gtk::Widget) {
    for c in ["bar-button", "tool-toggle", "flat"] {
        w.remove_css_class(c);
    }
}

fn set_child(w: &gtk::Widget, child: &gtk::Box) {
    if let Some(m) = w.downcast_ref::<gtk::MenuButton>() {
        m.set_always_show_arrow(false);
        m.set_child(Some(child));
    } else if let Some(b) = w.downcast_ref::<gtk::Button>() {
        b.set_child(Some(child));
    }
}

fn set_icon(w: &gtk::Widget, icon: &str) {
    if let Some(m) = w.downcast_ref::<gtk::MenuButton>() {
        m.set_icon_name(icon);
    } else if let Some(b) = w.downcast_ref::<gtk::Button>() {
        b.set_icon_name(icon);
    }
}

/// Whether a popover inside `w` (another menu) is showing.
fn has_open_popover(w: &gtk::Widget) -> bool {
    let mut child = w.first_child();
    while let Some(c) = child {
        if c.downcast_ref::<gtk::MenuButton>().is_some_and(|m| m.popover().is_some_and(|p| p.is_visible())) {
            return true;
        }
        if has_open_popover(&c) {
            return true;
        }
        child = c.next_sibling();
    }
    false
}
