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
    let b = settings_button(open);
    b.add_css_class("status-action");
    b
}

/// The settings button every Nyx app uses: two sliders. In 2.0 it sits at the
/// right end of the header.
pub fn settings_button(open: impl Fn() + 'static) -> gtk::Button {
    install_icons();
    let b = widgets::bar_button(SETTINGS_ICON, "Settings (Ctrl+,)");
    b.connect_clicked(move |_| open());
    b
}

const SETTINGS_ICON: &str = "nyx-settings-symbolic";

/// The suite's own symbolic icons, written once to the app's cache and added to
/// the icon theme. Filled shapes: GTK fills every path of a symbolic icon.
fn install_icons() {
    thread_local! {
        static DONE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    }
    if DONE.with(|d| d.replace(true)) {
        return;
    }
    let dir = nyx_core::paths::cache_dir().join("nyx-icons");
    let path = dir.join(format!("{SETTINGS_ICON}.svg"));
    if std::fs::read_to_string(&path).ok().as_deref() != Some(SETTINGS_SVG) {
        let _ = nyx_core::cmd::atomic_write(&path, SETTINGS_SVG);
    }
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::IconTheme::for_display(&display).add_search_path(dir);
    }
}

const SETTINGS_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24">
<g fill="#000000">
<rect x="3.2" y="6.2" width="10.6" height="1.6" rx="0.8"/>
<rect x="16.2" y="6.2" width="4.6" height="1.6" rx="0.8"/>
<rect x="3.2" y="16.2" width="4.6" height="1.6" rx="0.8"/>
<rect x="10.2" y="16.2" width="10.6" height="1.6" rx="0.8"/>
<rect x="14.2" y="4.2" width="1.6" height="5.6" rx="0.8"/>
<rect x="8.2" y="14.2" width="1.6" height="5.6" rx="0.8"/>
</g>
</svg>
"##;

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

/// The Nyx 2.0 unified header: the app's symbolic icon and name (in its accent)
/// with an optional subtitle at the left, what the app navigates by in the
/// middle, its actions at the right, and the window controls after them while
/// the window floats. Empty parts drag the window; a double-click maximizes.
pub struct AppHeader {
    /// Put this at the top of the window.
    pub root: gtk::WindowHandle,
    pub subtitle: gtk::Label,
    /// After the name and subtitle (a sidebar toggle, back and forward).
    pub start: gtk::Box,
    /// Centred in the header (a path, a search field, a view switch).
    pub center: gtk::Box,
    /// Before the window controls (the app's actions and its `⋯` menu).
    pub end: gtk::Box,
}

pub fn app_header(window: &impl IsA<gtk::Window>, icon: &str, name: &str) -> AppHeader {
    let bar = gtk::CenterBox::new();
    bar.add_css_class("app-header");
    let left = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let image = gtk::Image::from_icon_name(icon);
    image.add_css_class("app-icon");
    left.append(&image);
    left.append(&widgets::label(name, "app-title"));
    let subtitle = widgets::label("", "app-subtitle");
    subtitle.set_ellipsize(gtk::pango::EllipsizeMode::End);
    subtitle.set_visible(false);
    left.append(&subtitle);
    let start = gtk::Box::new(gtk::Orientation::Horizontal, 2);
    start.set_margin_start(10);
    left.append(&start);
    let center = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let right = gtk::Box::new(gtk::Orientation::Horizontal, 2);
    let end = gtk::Box::new(gtk::Orientation::Horizontal, 2);
    right.append(&end);
    let controls = gtk::WindowControls::new(gtk::PackType::End);
    controls.set_decoration_layout(Some(":minimize,maximize,close"));
    controls.add_css_class("window-controls");
    right.append(&controls);
    bar.set_start_widget(Some(&left));
    bar.set_center_widget(Some(&center));
    bar.set_end_widget(Some(&right));
    show_controls_when_floating(window.upcast_ref(), &controls);
    let root = gtk::WindowHandle::new();
    root.set_child(Some(&bar));
    AppHeader { root, subtitle, start, center, end }
}

impl AppHeader {
    /// Show `text` after the app's name (the page, folder or account), or nothing.
    pub fn set_subtitle(&self, text: &str) {
        self.subtitle.set_text(text);
        self.subtitle.set_visible(!text.is_empty());
    }
}

/// Window controls only make sense on a floating window: a tiled, maximized or
/// fullscreen one is placed by the compositor. Hyprland reports tiled windows
/// through the toplevel's tiled states.
fn show_controls_when_floating(window: &gtk::Window, controls: &gtk::WindowControls) {
    let update = {
        let controls = controls.clone();
        move |surface: &gtk::gdk::Toplevel| {
            use gtk::gdk::ToplevelState as S;
            let placed =
                S::TILED | S::TOP_TILED | S::RIGHT_TILED | S::BOTTOM_TILED | S::LEFT_TILED | S::MAXIMIZED | S::FULLSCREEN;
            controls.set_visible(!surface.state().intersects(placed));
        }
    };
    window.connect_realize(move |w| {
        let Some(toplevel) = w.surface().and_downcast::<gtk::gdk::Toplevel>() else { return };
        update(&toplevel);
        let update = update.clone();
        toplevel.connect_state_notify(move |t| update(t));
    });
}
