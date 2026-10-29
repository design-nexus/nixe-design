//! Settings as a card over the window: the groups listed on the left, the
//! chosen group's options on the right, and a search across all of them.
//!
//! The app builds its settings page as usual (a body whose children are
//! `settings-group` boxes, each named after its group) and hands it here. Every
//! app's card then ends with the same three groups, built here from its
//! [`Standard`]: Appearance (Follow Omarchy, Theme, Accent, Reduce motion, Open
//! on), Keyboard (its shortcuts) and About (version, licence, links).

use crate::widgets;
use gtk::prelude::*;
use nyx_core::paths;
use std::cell::RefCell;

/// How the card lists an app's settings.
#[derive(Clone, Copy)]
pub struct Spec {
    /// Each of the app's own groups: name, icon and one-line summary, in the
    /// card's list (the standard groups follow them).
    pub groups: &'static [(&'static str, &'static str, &'static str)],
    /// The words a row is found by, when the app keeps its own; otherwise the
    /// row's visible text is searched.
    pub search_text: Option<fn(&gtk::Widget) -> Option<String>>,
    /// What the standard groups need; `None` for a card that isn't an app's settings.
    pub standard: Option<Standard>,
}

/// What the card's Appearance, Keyboard and About groups are built from.
#[derive(Clone, Copy)]
pub struct Standard {
    /// The app's key for its accent (`"music"`), as in [`crate::widgets::accent_row`].
    pub app: &'static str,
    pub about: About,
    /// Every shortcut: its keys and what it does.
    pub shortcuts: &'static [(&'static [&'static str], &'static str)],
    /// The app's Reduce motion preference.
    pub reduce_motion: fn() -> bool,
    pub set_reduce_motion: fn(bool),
}

/// The About group: who the app is.
#[derive(Clone, Copy)]
pub struct About {
    pub name: &'static str,
    /// The app's icon name (its application id).
    pub icon: &'static str,
    /// `env!("CARGO_PKG_VERSION")` of the app's crate.
    pub version: &'static str,
    /// One line on what it does.
    pub description: &'static str,
    pub licence: &'static str,
    /// The project page and where to report a problem (empty: no row).
    pub website: &'static str,
    pub issues: &'static str,
    /// What it's built on, by name and a few words (`("mpv", "video playback")`).
    pub credits: &'static [(&'static str, &'static str)],
}

/// The standard groups' names, icons and summaries, after the app's own.
const STANDARD_GROUPS: &[(&str, &str, &str)] = &[
    ("Appearance", "applications-graphics-symbolic", "Theme, accent, motion"),
    ("Keyboard", "input-keyboard-symbolic", "Shortcuts"),
    ("About", "help-about-symbolic", "Version, licence, links"),
];

struct Group {
    name: String,
    wrapper: gtk::Widget,
    button: gtk::Button,
}

struct Dialog {
    veil: gtk::Box,
    card: gtk::Box,
    title: gtk::Label,
    search: gtk::SearchEntry,
    empty: gtk::Label,
    groups: Vec<Group>,
    current: String,
    /// The settings page this card was built around.
    root: gtk::Widget,
}

thread_local! {
    static DIALOG: RefCell<Option<Dialog>> = const { RefCell::new(None) };
    static SPEC: RefCell<Option<Spec>> = const { RefCell::new(None) };
}

/// Show the card over `overlay`. The settings page comes from `page` (its
/// scrolled root and its body) the first time; later calls reuse it.
pub fn open(overlay: &gtk::Overlay, window: &gtk::ApplicationWindow, spec: Spec, page: impl FnOnce() -> (gtk::Widget, gtk::Box)) {
    if DIALOG.with(|d| d.borrow().is_none()) {
        let (root, body) = page();
        install(overlay, spec, &root, &body, first_group());
    }
    show(window);
}

/// The group the card opens on: the first, or `NYX_SETTINGS_GROUP` (for checks).
fn first_group() -> String {
    std::env::var("NYX_SETTINGS_GROUP").unwrap_or_default()
}

/// Show the card around a page the caller has just built (rebuilt each time it
/// opens), keeping the group that was showing last time.
pub fn open_page(overlay: &gtk::Overlay, window: &gtk::ApplicationWindow, spec: Spec, root: &gtk::Widget, body: &gtk::Box) {
    let current = DIALOG.with(|d| {
        let old = d.borrow_mut().take()?;
        if let Some(parent) = old.veil.parent().and_downcast::<gtk::Overlay>() {
            parent.remove_overlay(&old.veil);
        }
        Some(old.current)
    });
    install(overlay, spec, root, body, current.unwrap_or_else(first_group));
    show(window);
}

/// The settings page the card is showing, if it has been built.
pub fn page() -> Option<gtk::Widget> {
    DIALOG.with(|d| d.borrow().as_ref().map(|d| d.root.clone()))
}

fn install(overlay: &gtk::Overlay, spec: Spec, root: &gtk::Widget, body: &gtk::Box, current: String) {
    SPEC.with(|s| *s.borrow_mut() = Some(spec));
    if let Some(standard) = spec.standard {
        append_standard(body, &standard);
    }
    let mut dialog = build(spec, root, body);
    dialog.current = current;
    overlay.add_overlay(&dialog.veil);
    overlay.set_measure_overlay(&dialog.veil, false);
    DIALOG.with(|d| *d.borrow_mut() = Some(dialog));
}

fn show(window: &gtk::ApplicationWindow) {
    let first = DIALOG.with(|d| {
        let d = d.borrow();
        let d = d.as_ref()?;
        d.veil.set_visible(true);
        let known = d.groups.iter().any(|g| g.name == d.current);
        Some(if known { d.current.clone() } else { d.groups.first().map(|g| g.name.clone()).unwrap_or_default() })
    });
    fit(window);
    if let Some(name) = first {
        select(&name);
    }
}

pub fn is_open() -> bool {
    DIALOG.with(|d| d.borrow().as_ref().is_some_and(|d| d.veil.is_visible()))
}

pub fn close() {
    DIALOG.with(|d| {
        if let Some(d) = d.borrow().as_ref() {
            d.card.set_size_request(-1, -1);
            d.veil.set_visible(false);
        }
    });
}

/// Esc clears the search first, then closes.
pub fn escape() {
    let search = DIALOG.with(|d| d.borrow().as_ref().map(|d| d.search.clone()));
    match search {
        Some(s) if !s.text().is_empty() => s.set_text(""),
        _ => close(),
    }
}

pub fn focus_search() {
    if let Some(s) = DIALOG.with(|d| d.borrow().as_ref().map(|d| d.search.clone())) {
        s.grab_focus();
    }
}

/// Size the card to the window: as large as the window allows, up to a limit.
pub fn fit(window: &gtk::ApplicationWindow) {
    let (w, h) = (window.width(), window.height());
    if w <= 0 || h <= 0 {
        return;
    }
    let margin = if w < 580 { 20 } else { 48 };
    let size = ((w - margin).clamp(0, 1040), (h - margin).clamp(0, 720));
    DIALOG.with(|d| {
        if let Some(d) = d.borrow().as_ref()
            && d.card.size_request() != size
        {
            d.card.set_size_request(size.0, size.1);
            // A tiled half-screen window gets a slimmer list and less padding.
            if size.0 < 900 {
                d.card.add_css_class("narrow");
            } else {
                d.card.remove_css_class("narrow");
            }
            if size.0 < 540 {
                d.card.add_css_class("compact");
            } else {
                d.card.remove_css_class("compact");
            }
        }
    });
}

fn build(spec: Spec, root: &gtk::Widget, body: &gtk::Box) -> Dialog {
    let veil = gtk::Box::new(gtk::Orientation::Vertical, 0);
    veil.add_css_class("settings-veil");
    veil.set_hexpand(true);
    veil.set_vexpand(true);
    veil.set_visible(false);
    let card = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    card.add_css_class("settings-dialog");
    card.set_halign(gtk::Align::Center);
    card.set_valign(gtk::Align::Center);
    card.set_vexpand(true);
    card.set_overflow(gtk::Overflow::Hidden);
    veil.append(&card);

    // A click on the dimmed window around the card closes it.
    let click = gtk::GestureClick::new();
    let c = card.clone();
    click.connect_pressed(move |g, _, x, y| {
        let Some(veil) = g.widget() else { return };
        let inside = c.compute_bounds(&veil).is_some_and(|b| b.contains_point(&gtk::graphene::Point::new(x as f32, y as f32)));
        if !inside {
            close();
        }
    });
    veil.add_controller(click);

    // ----- The list of groups -----
    let side = gtk::Box::new(gtk::Orientation::Vertical, 0);
    side.add_css_class("settings-dialog-side");
    side.append(&widgets::label("SETTINGS", "dialog-heading"));
    let search = gtk::SearchEntry::new();
    search.set_placeholder_text(Some("Search settings"));
    search.add_css_class("settings-search");
    side.append(&search);
    let list = gtk::Box::new(gtk::Orientation::Vertical, 4);

    let mut groups = Vec::new();
    let mut child = body.first_child();
    while let Some(w) = child {
        child = w.next_sibling();
        if !w.has_css_class("settings-group") {
            continue;
        }
        let name = w.widget_name().to_string();
        let (icon, summary) = spec
            .groups
            .iter()
            .chain(STANDARD_GROUPS.iter().filter(|_| spec.standard.is_some()))
            .find(|g| g.0 == name)
            .map(|g| (g.1, g.2))
            .unwrap_or(("emblem-system-symbolic", ""));
        let button = gtk::Button::new();
        button.add_css_class("dialog-nav");
        let content = gtk::Box::new(gtk::Orientation::Horizontal, 14);
        let image = gtk::Image::from_icon_name(icon);
        image.set_valign(gtk::Align::Start);
        content.append(&image);
        let text = gtk::Box::new(gtk::Orientation::Vertical, 1);
        text.append(&widgets::label(&name, "dialog-nav-title"));
        if !summary.is_empty() {
            let s = widgets::label(summary, "dialog-nav-summary");
            s.set_ellipsize(gtk::pango::EllipsizeMode::End);
            text.append(&s);
        }
        content.append(&text);
        button.set_child(Some(&content));
        let n = name.clone();
        button.connect_clicked(move |_| {
            let search = DIALOG.with(|d| d.borrow().as_ref().map(|d| d.search.clone()));
            if let Some(s) = search {
                s.set_text("");
            }
            select(&n);
        });
        list.append(&button);
        groups.push(Group { name, wrapper: w, button });
    }
    let list_scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::External)
        .vexpand(true)
        .child(&list)
        .build();
    side.append(&list_scroll);
    card.append(&side);

    // ----- The chosen group -----
    let main = gtk::Box::new(gtk::Orientation::Vertical, 0);
    main.add_css_class("settings-dialog-main");
    main.set_hexpand(true);
    let head = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    head.add_css_class("dialog-head");
    let title = widgets::label("", "dialog-title");
    title.set_hexpand(true);
    head.append(&title);
    head.append(&widgets::config_button(&[paths::prefs_file()]));
    let x = widgets::bar_button("window-close-symbolic", "Close (Esc)");
    x.connect_clicked(|_| close());
    head.append(&x);
    main.append(&head);
    let empty = widgets::label("", "dialog-empty");
    empty.set_xalign(0.5);
    empty.set_vexpand(true);
    empty.set_visible(false);
    main.append(&empty);
    if let Some(parent) = root.parent().and_downcast::<gtk::Box>() {
        parent.remove(root);
    }
    main.append(root);
    card.append(&main);

    search.connect_search_changed(|e| on_search(&e.text()));
    Dialog { veil, card, title, search, empty, groups, current: String::new(), root: root.clone() }
}

/// Show one group on its own.
pub fn select(name: &str) {
    DIALOG.with(|d| {
        let mut d = d.borrow_mut();
        let Some(d) = d.as_mut() else { return };
        d.current = name.to_string();
        d.title.set_text(name);
        for g in &d.groups {
            let on = g.name == name;
            g.wrapper.set_visible(on);
            set_heading(&g.wrapper, false);
            if on {
                g.button.add_css_class("active");
            } else {
                g.button.remove_css_class("active");
            }
        }
    });
}

/// A group's own heading shows only in search results, where several groups
/// share the page.
fn set_heading(wrapper: &gtk::Widget, visible: bool) {
    if let Some(first) = wrapper.first_child().filter(|w| w.has_css_class("group-title")) {
        first.set_visible(visible);
    }
}

fn on_search(text: &str) {
    let q = text.trim().to_lowercase();
    if q.is_empty() {
        let current = DIALOG.with(|d| {
            let d = d.borrow();
            let d = d.as_ref()?;
            for g in &d.groups {
                for_each_row(&g.wrapper, &mut |row| {
                    if row.has_css_class("search-hidden") {
                        row.remove_css_class("search-hidden");
                        row.set_visible(true);
                    }
                });
                g.button.set_visible(true);
            }
            d.empty.set_visible(false);
            Some(d.current.clone())
        });
        if let Some(c) = current {
            select(&c);
        }
        return;
    }
    DIALOG.with(|d| {
        let d = d.borrow();
        let Some(d) = d.as_ref() else { return };
        let mut total = 0;
        for g in &d.groups {
            let whole = g.name.to_lowercase().contains(&q);
            let mut hits = 0;
            for_each_row(&g.wrapper, &mut |row| {
                let text = SPEC
                    .with(|s| s.borrow().as_ref().and_then(|s| s.search_text).and_then(|f| f(row)))
                    .unwrap_or_else(|| text_of(row).to_lowercase());
                let hit = whole || q.split_whitespace().all(|t| text.contains(t));
                if hit && row.has_css_class("search-hidden") {
                    row.remove_css_class("search-hidden");
                    row.set_visible(true);
                } else if !hit && row.is_visible() {
                    row.add_css_class("search-hidden");
                    row.set_visible(false);
                }
                if hit && row.is_visible() {
                    hits += 1;
                }
            });
            g.wrapper.set_visible(hits > 0);
            set_heading(&g.wrapper, true);
            g.button.set_visible(hits > 0);
            g.button.remove_css_class("active");
            total += hits;
        }
        d.title.set_text("Search");
        d.empty.set_text(&format!("No settings match “{}”", text.trim()));
        d.empty.set_visible(total == 0);
    });
}

fn for_each_row(w: &gtk::Widget, f: &mut dyn FnMut(&gtk::Widget)) {
    let mut child = w.first_child();
    while let Some(c) = child {
        if c.has_css_class("settings-option") {
            f(&c);
        } else {
            for_each_row(&c, f);
        }
        child = c.next_sibling();
    }
}

/// Every label's text in a row, for matching.
fn text_of(w: &gtk::Widget) -> String {
    let mut out = String::new();
    if let Some(l) = w.downcast_ref::<gtk::Label>() {
        out.push_str(&l.text());
        out.push(' ');
    }
    let mut child = w.first_child();
    while let Some(c) = child {
        out.push_str(&text_of(&c));
        child = c.next_sibling();
    }
    out
}

// ----- The standard groups -----

/// A group as the apps build theirs: a box named after it, its heading (shown in
/// search results) and the list its rows join.
fn group(body: &gtk::Box, title: &str) -> gtk::Box {
    let wrapper = gtk::Box::new(gtk::Orientation::Vertical, 0);
    wrapper.add_css_class("settings-group");
    wrapper.set_widget_name(title);
    let heading = gtk::Label::new(Some(&title.to_uppercase()));
    heading.add_css_class("group-title");
    heading.set_xalign(0.0);
    wrapper.append(&heading);
    let list = gtk::Box::new(gtk::Orientation::Vertical, 0);
    list.add_css_class("group-list");
    wrapper.append(&list);
    body.append(&wrapper);
    list
}

/// Appearance, Keyboard and About, after the app's own groups.
fn append_standard(body: &gtk::Box, standard: &Standard) {
    for (name, _, _) in STANDARD_GROUPS {
        standard_group(body, name, standard);
    }
}

/// One of the standard groups (`"Appearance"`, `"Keyboard"` or `"About"`), for an
/// app whose settings aren't this card but should hold the same groups.
pub fn standard_group(body: &gtk::Box, name: &str, standard: &Standard) {
    match name {
        "Appearance" => appearance(&group(body, name), standard),
        "Keyboard" => keyboard(&group(body, name), standard.shortcuts),
        "About" => about(&group(body, name), &standard.about),
        _ => {}
    }
}

/// The standard groups' names, icons and summaries, in order.
pub fn standard_groups() -> &'static [(&'static str, &'static str, &'static str)] {
    STANDARD_GROUPS
}

/// Follow Omarchy, Theme, Accent, Reduce motion and Open on.
fn appearance(list: &gtk::Box, standard: &Standard) {
    let prefs = nyx_theme::app_prefs();
    let chooses = nyx_theme::can_choose_theme();
    let themes: Vec<(String, String)> = nyx_theme::all().iter().map(|t| (t.id.clone(), t.name.clone())).collect();
    let (theme_row, theme_dd) = widgets::choice_row(
        "Theme",
        "A bundled theme, or one of your own.",
        themes,
        &prefs.theme,
        |id| nyx_theme::choose_theme(nyx_theme::ThemeMode::Theme, &id),
    );
    let following = prefs.mode == nyx_theme::ThemeMode::Omarchy;
    theme_dd.set_sensitive(chooses && (!following || !nyx_theme::omarchy_available()));
    if nyx_theme::omarchy_available() {
        let dd = theme_dd.clone();
        let (row, switch) = widgets::switch_row(
            "Follow Omarchy theme",
            "Match the desktop's colors and update when the Omarchy theme changes.",
            following,
            move |on| {
                let current = nyx_theme::app_prefs().theme;
                let mode = if on { nyx_theme::ThemeMode::Omarchy } else { nyx_theme::ThemeMode::Theme };
                nyx_theme::choose_theme(mode, &current);
                dd.set_sensitive(!on);
            },
        );
        switch.set_sensitive(chooses);
        list.append(&row);
    }
    list.append(&theme_row);
    list.append(&widgets::accent_row(standard.app));
    let set = standard.set_reduce_motion;
    let (row, _) =
        widgets::switch_row("Reduce motion", "Turn off transitions in this window.", (standard.reduce_motion)(), move |on| {
            set(on);
            nyx_theme::apply();
        });
    list.append(&row);
    list.append(&widgets::start_row());
}

/// Every shortcut, with its keys.
fn keyboard(list: &gtk::Box, shortcuts: &[(&[&str], &str)]) {
    for (keys, what) in shortcuts {
        list.append(&widgets::row(what, "", Some(widgets::key_caps(keys).upcast_ref())));
    }
}

/// The app's icon, name, version and what it does; its licence, links and credits.
fn about(list: &gtk::Box, about: &About) {
    let head = gtk::Box::new(gtk::Orientation::Horizontal, 16);
    head.add_css_class("settings-option");
    head.add_css_class("about-head");
    let icon = gtk::Image::from_icon_name(about.icon);
    icon.set_pixel_size(64);
    head.append(&icon);
    let text = gtk::Box::new(gtk::Orientation::Vertical, 3);
    text.set_valign(gtk::Align::Center);
    let name = widgets::label(about.name, "about-name");
    name.set_xalign(0.0);
    text.append(&name);
    let version = widgets::label(&format!("Version {}", about.version), "about-version");
    version.set_xalign(0.0);
    version.set_selectable(true);
    text.append(&version);
    let desc = widgets::label(about.description, "settings-option-description");
    desc.set_xalign(0.0);
    desc.set_wrap(true);
    text.append(&desc);
    head.append(&text);
    list.append(&head);
    list.append(&widgets::row("Licence", about.licence, None));
    for (title, desc, url, icon) in [
        ("Project page", "The source, releases and documentation.", about.website, "web-browser-symbolic"),
        ("Report a problem", "Open an issue on the project page.", about.issues, "dialog-warning-symbolic"),
    ] {
        if url.is_empty() {
            continue;
        }
        let open = widgets::bar_button(icon, &format!("Open {url}"));
        let url = url.to_string();
        open.connect_clicked(move |b| {
            let window = b.root().and_downcast::<gtk::Window>();
            gtk::UriLauncher::new(&url).launch(window.as_ref(), gtk::gio::Cancellable::NONE, |_| {});
        });
        list.append(&widgets::row(title, desc, Some(open.upcast_ref())));
    }
    if !about.credits.is_empty() {
        let built: Vec<String> = about.credits.iter().map(|(name, what)| format!("{name} ({what})")).collect();
        list.append(&widgets::row("Built with", &built.join(", "), None));
    }
}
