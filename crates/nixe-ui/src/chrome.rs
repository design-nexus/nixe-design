//! The window's frame without a top bar. Where you are shows in the sidebar
//! (and the window title); the sidebar's head holds its toggle and search;
//! the settings gear and app actions sit at the right of the status bar.

use crate::widgets;
use gtk::prelude::*;

/// The sidebar's first row: the collapse toggle, the app's name, and a search
/// button when the app has one. In the icon-only sidebar the name hides and
/// the buttons stack.
pub struct SidebarHead {
    pub root: gtk::Box,
    pub toggle: gtk::Button,
    pub search: Option<gtk::Button>,
}

pub fn sidebar_head(app_name: &str, search_tooltip: Option<&str>) -> SidebarHead {
    let root = gtk::Box::new(gtk::Orientation::Horizontal, 2);
    root.add_css_class("sidebar-head");
    let toggle = widgets::bar_button("sidebar-show-symbolic", "Collapse or expand the sidebar (Ctrl+B)");
    root.append(&toggle);
    let name = widgets::label(app_name, "sidebar-app-name");
    name.add_css_class("compact-hide");
    name.set_hexpand(true);
    name.set_xalign(0.0);
    root.append(&name);
    let search = search_tooltip.map(|tip| {
        let b = widgets::bar_button("system-search-symbolic", tip);
        root.append(&b);
        b
    });
    SidebarHead { root, toggle, search }
}

/// Lay the head out for the icon-only sidebar (stacked) or the full one (a row).
pub fn set_head_compact(head: &gtk::Box, compact: bool) {
    head.set_orientation(if compact { gtk::Orientation::Vertical } else { gtk::Orientation::Horizontal });
    head.set_halign(if compact { gtk::Align::Center } else { gtk::Align::Fill });
    let mut child = head.first_child();
    while let Some(c) = child {
        if c.has_css_class("sidebar-app-name") {
            c.set_visible(!compact);
        }
        child = c.next_sibling();
    }
}

/// A slim strip over the page that holds `entry`, shown only while the entry is.
/// Apps keep showing and hiding the entry itself.
pub fn search_strip(entry: &gtk::SearchEntry) -> gtk::Box {
    let strip = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    strip.add_css_class("search-strip");
    entry.set_hexpand(true);
    if let Some(parent) = entry.parent().and_downcast::<gtk::Box>() {
        parent.remove(entry);
    }
    strip.append(entry);
    strip.set_visible(entry.get_visible());
    let s = strip.clone();
    // Effective visibility includes the hidden strip itself; use the entry’s
    // requested visibility so showing it can reveal its parent.
    entry.connect_visible_notify(move |e| s.set_visible(e.get_visible()));
    strip
}

/// The settings gear for the end of the status bar.
pub fn gear(open: impl Fn() + 'static) -> gtk::Button {
    let b = widgets::bar_button("emblem-system-symbolic", "Settings (Ctrl+,)");
    b.add_css_class("status-action");
    b.connect_clicked(move |_| open());
    b
}

/// A small icon button for the status bar (an app action that used to sit in
/// the top bar).
pub fn status_action(icon: &str, tooltip: &str, run: impl Fn() + 'static) -> gtk::Button {
    let b = widgets::bar_button(icon, tooltip);
    b.add_css_class("status-action");
    b.connect_clicked(move |_| run());
    b
}

/// The window title for a page: "Music — Albums".
pub fn set_title(window: &impl IsA<gtk::Window>, app_name: &str, page: &str) {
    let title = if page.is_empty() || page == app_name { app_name.to_string() } else { format!("{app_name} — {page}") };
    window.set_title(Some(&title));
}

/// The suite's F1 control: the key is highlighted separately from its label.
pub fn shortcuts_button(open: impl Fn() + 'static) -> gtk::Button {
    let help = gtk::Button::new();
    help.add_css_class("status-help");
    let content = widgets::hbox(10);
    content.append(&widgets::label("F1", "status-key"));
    content.append(&widgets::label("Shortcuts", ""));
    help.set_child(Some(&content));
    help.set_tooltip_text(Some("Show the keyboard shortcuts"));
    help.connect_clicked(move |_| open());
    help
}
