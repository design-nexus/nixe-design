//! A command bar: one 44px row over a document, for the things you do to it
//! (File, undo, insert, play). Formatting lives in the [`crate::inspector`].
//!
//! ```text
//! File▾ │ ↶ ↷ │ [icon Label] [icon Label] … ⋯ │   Name.ext •   │ end… ◨
//! ```
//!
//! It never wraps or grows taller: when the window narrows, the tools first
//! drop their labels, then the least important ones move into the `⋯` menu.
//! Tools added with [`CommandBar::more`] always live in that menu.

use crate::widgets;
use gtk::glib;
use gtk::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// Width of a tool showing only its icon, with the gap after it.
const ICON_ONLY: i32 = 32;

struct Tool {
    widget: gtk::Widget,
    label: gtk::Label,
    keys: gtk::Label,
    priority: i32,
    /// Width with its label, worked out from the text.
    full: i32,
    /// Always in the menu.
    menu_only: bool,
    folded: Cell<bool>,
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
    tools: RefCell<Vec<Rc<Tool>>>,
    labels: Cell<bool>,
    width: Cell<i32>,
}

impl CommandBar {
    pub fn new() -> Rc<CommandBar> {
        let root = gtk::CenterBox::new();
        root.add_css_class("commandbar");
        let start = widgets::hbox(2);
        let lead = widgets::hbox(2);
        let tools_box = widgets::hbox(2);
        tools_box.add_css_class("commandbar-tools");
        let more = gtk::MenuButton::new();
        more.set_icon_name("view-more-horizontal-symbolic");
        more.add_css_class("commandbar-tool");
        more.add_css_class("icon-only");
        more.set_tooltip_text(Some("More"));
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
        // the tools fold instead.
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
            tools: RefCell::new(Vec::new()),
            labels: Cell::new(true),
            width: Cell::new(0),
        });
        // A menu row closes the menu once it has done its job.
        let pop2 = pop.clone();
        let click = gtk::GestureClick::new();
        click.set_propagation_phase(gtk::PropagationPhase::Bubble);
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

    /// At the start, before the tools: the File menu, undo and redo. A
    /// divider follows each call.
    pub fn lead(&self, items: &[&gtk::Widget]) {
        for w in items {
            if !w.has_css_class("commandbar-file") && (w.is::<gtk::Button>() || w.is::<gtk::MenuButton>()) {
                w.remove_css_class("bar-button");
                w.add_css_class("commandbar-tool");
                w.add_css_class("icon-only");
            }
            detach(w);
            self.lead.append(*w);
        }
        self.lead.append(&divider());
    }

    /// A tool on the bar: `widget` (a button or menu button) gets `icon` and
    /// `label`. Lower `priority` folds into the `⋯` menu first.
    pub fn tool(&self, widget: &impl IsA<gtk::Widget>, icon: &str, label: &str, priority: i32) {
        self.add(widget.upcast_ref(), icon, label, priority, false);
    }

    /// A tool that lives in the `⋯` menu.
    pub fn more(&self, widget: &impl IsA<gtk::Widget>, icon: &str, label: &str) {
        self.add(widget.upcast_ref(), icon, label, i32::MIN, true);
    }

    /// The bar's main action, as an accent pill (at most one per bar). It stays
    /// on the bar and keeps its label.
    pub fn primary(&self, widget: &impl IsA<gtk::Widget>, icon: &str, label: &str) {
        let w = widget.upcast_ref::<gtk::Widget>();
        let (content, _, _) = content(icon, label, &tooltip(w));
        set_content(w, &content);
        w.add_css_class("commandbar-primary");
        detach(w);
        self.lead.append(w);
    }

    /// A widget at the end of the bar (find, the inspector toggle).
    pub fn end(&self, w: &impl IsA<gtk::Widget>) {
        let w = w.upcast_ref::<gtk::Widget>();
        if w.is::<gtk::Button>() || w.is::<gtk::MenuButton>() {
            w.add_css_class("commandbar-tool");
            if !w.has_css_class("commandbar-labelled") {
                w.add_css_class("icon-only");
            }
        }
        detach(w);
        self.end.append(w);
    }

    /// A labelled tool at the end (Play). It keeps its label.
    pub fn end_tool(&self, widget: &impl IsA<gtk::Widget>, icon: &str, label: &str) {
        let w = widget.upcast_ref::<gtk::Widget>();
        let (content, _, _) = content(icon, label, &tooltip(w));
        set_content(w, &content);
        w.add_css_class("commandbar-labelled");
        self.end(w);
        self.end.append(&divider());
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

    fn add(&self, w: &gtk::Widget, icon: &str, label: &str, priority: i32, menu_only: bool) {
        let tip = tooltip(w);
        let (content, l, keys) = content(icon, label, &tip);
        set_content(w, &content);
        w.add_css_class("commandbar-tool");
        let text_width = l.create_pango_layout(Some(label)).pixel_size().0;
        let tool = Rc::new(Tool {
            widget: w.clone(),
            label: l,
            keys,
            priority,
            full: text_width + 16 + 6 + 18 + 2,
            menu_only,
            folded: Cell::new(false),
        });
        detach(w);
        if menu_only {
            self.put(&tool, true);
            self.more.set_visible(true);
        } else {
            self.tools_box.append(w);
        }
        self.tools.borrow_mut().push(tool);
    }

    /// Move a tool onto the bar or into the menu.
    fn put(&self, tool: &Tool, folded: bool) {
        if tool.folded.get() == folded && tool.widget.parent().is_some() {
            return;
        }
        tool.folded.set(folded);
        if let Some(p) = tool.widget.parent().and_downcast::<gtk::Box>() {
            p.remove(&tool.widget);
        }
        if folded {
            // Keep the menu in the order the tools were added.
            let tools = self.tools.borrow();
            let after = tools
                .iter()
                .take_while(|t| !std::ptr::eq(t.as_ref(), tool))
                .filter(|t| t.folded.get() && t.widget.parent().is_some())
                .last()
                .map(|t| t.widget.clone());
            drop(tools);
            self.menu_box.insert_child_after(&tool.widget, after.as_ref());
            tool.widget.add_css_class("in-menu");
            tool.label.set_visible(true);
            tool.keys.set_visible(!tool.keys.text().is_empty());
        } else {
            let tools = self.tools.borrow();
            let after = tools
                .iter()
                .take_while(|t| !std::ptr::eq(t.as_ref(), tool))
                .filter(|t| !t.folded.get() && t.widget.parent().is_some())
                .last()
                .map(|t| t.widget.clone());
            drop(tools);
            self.tools_box.insert_child_after(&tool.widget, after.as_ref());
            tool.widget.remove_css_class("in-menu");
            tool.label.set_visible(self.labels.get());
            tool.keys.set_visible(false);
        }
    }

    /// Labels if everything fits with them, else icons, else fold the least
    /// important tools into the menu until the rest fit.
    fn fit(&self) {
        let width = self.root.width();
        if width <= 0 {
            return;
        }
        let nat = |w: &gtk::Widget| if w.is_visible() { w.measure(gtk::Orientation::Horizontal, -1).1 } else { 0 };
        let title = nat(self.title.upcast_ref());
        // The title is centred while there's room, and moves aside to make more.
        let end = nat(self.end.upcast_ref());
        let available = width - title - end - nat(self.lead.upcast_ref()) - 48;
        let tools = self.tools.borrow().clone();
        let bar_tools: Vec<&Rc<Tool>> = tools.iter().filter(|t| !t.menu_only).collect();
        let more_always = tools.iter().any(|t| t.menu_only);
        let more_w = if more_always { ICON_ONLY } else { 0 };
        let full: i32 = bar_tools.iter().map(|t| t.full).sum();
        let labels = full + more_w <= available;
        self.labels.set(labels);
        let mut fold: Vec<bool> = vec![false; bar_tools.len()];
        if !labels {
            let mut total = bar_tools.len() as i32 * ICON_ONLY + more_w;
            let mut order: Vec<usize> = (0..bar_tools.len()).collect();
            order.sort_by_key(|&i| (bar_tools[i].priority, std::cmp::Reverse(i)));
            for i in order {
                if total <= available {
                    break;
                }
                if !more_always && !fold.iter().any(|f| *f) {
                    total += ICON_ONLY;
                }
                total -= ICON_ONLY;
                fold[i] = true;
            }
        }
        for (t, f) in bar_tools.iter().zip(&fold) {
            self.put(t, *f);
            if !*f {
                t.label.set_visible(labels);
                if labels {
                    t.widget.remove_css_class("icon-only");
                } else {
                    t.widget.add_css_class("icon-only");
                }
            } else {
                t.widget.remove_css_class("icon-only");
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

/// Take `w` out of the box it was made in (a toolbar group).
fn detach(w: &gtk::Widget) {
    if let Some(p) = w.parent().and_downcast::<gtk::Box>() {
        p.remove(w);
    }
    w.set_hexpand(false);
    w.set_valign(gtk::Align::Center);
}

fn tooltip(w: &gtk::Widget) -> String {
    w.tooltip_text().map(|t| t.to_string()).unwrap_or_default()
}

/// An icon, a label, and (shown in the menu) the keys from the tooltip.
fn content(icon: &str, label: &str, tip: &str) -> (gtk::Box, gtk::Label, gtk::Label) {
    let b = widgets::hbox(6);
    b.set_halign(gtk::Align::Start);
    b.append(&gtk::Image::from_icon_name(icon));
    let l = widgets::label(label, "commandbar-label");
    l.set_xalign(0.0);
    b.append(&l);
    let keys = widgets::label(&crate::palette::split_keys(tip).1, "commandbar-keys");
    keys.set_visible(false);
    b.append(&keys);
    (b, l, keys)
}

fn set_content(w: &gtk::Widget, content: &gtk::Box) {
    if let Some(m) = w.downcast_ref::<gtk::MenuButton>() {
        m.set_always_show_arrow(false);
        m.set_child(Some(content));
    } else if let Some(b) = w.downcast_ref::<gtk::Button>() {
        b.set_child(Some(content));
    }
    w.remove_css_class("bar-button");
    w.remove_css_class("tool-toggle");
    w.remove_css_class("flat");
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
