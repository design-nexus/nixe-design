//! Building blocks shared by every Nixe app: boxes, labels, buttons, option
//! rows, banners and empty states, all styled by the suite's stylesheet.

use gtk::prelude::*;
use gtk::{gio, glib, pango};
use std::path::PathBuf;
use std::rc::Rc;

/// A small modal card over the main window, closed with Esc.
pub fn dialog(title: &str, width: i32) -> (gtk::Window, gtk::Box) {
    let dialog = gtk::Window::builder().modal(true).title(title).default_width(width).build();
    if let Some(parent) = main_window() {
        dialog.set_transient_for(Some(&parent));
        // The app's window class carries its scoped styles.
        for class in parent.css_classes() {
            if class.ends_with("-window") {
                dialog.add_css_class(&class);
            }
        }
    }
    dialog.set_titlebar(Some(&gtk::Box::new(gtk::Orientation::Horizontal, 0)));
    let card = vbox(12);
    card.add_css_class("dialog-card");
    card.append(&label(title, "section-title"));
    dialog.set_child(Some(&card));
    let keys = gtk::EventControllerKey::new();
    let d = dialog.clone();
    keys.connect_key_pressed(move |_, key, _, _| {
        if key == gtk::gdk::Key::Escape {
            d.close();
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    dialog.add_controller(keys);
    (dialog, card)
}

/// The app's main window: the first of the running application's windows.
pub fn main_window() -> Option<gtk::Window> {
    let app = gio::Application::default()?.downcast::<gtk::Application>().ok()?;
    app.windows().into_iter().find(|w| !w.is_modal()).or_else(|| app.active_window())
}

/// A labelled action: the button's text and what it does.
pub type Action<'a> = Option<(&'a str, Box<dyn Fn()>)>;

/// Ask for a line of text in a small dialog ("Not now" first, then the action).
pub fn ask_text(title: &str, desc: &str, initial: &str, confirm: &str, on_ok: impl Fn(String) + 'static) -> gtk::Box {
    let (dialog, card) = dialog(title, 420);
    if !desc.is_empty() {
        let d = label(desc, "dim");
        d.set_wrap(true);
        card.append(&d);
    }
    let entry = gtk::Entry::new();
    entry.set_text(initial);
    entry.set_activates_default(false);
    card.append(&entry);
    let extra = vbox(6);
    card.append(&extra);
    let buttons = hbox(8);
    buttons.set_halign(gtk::Align::End);
    buttons.set_margin_top(6);
    let cancel = gtk::Button::with_label("Not now");
    let ok = gtk::Button::with_label(confirm);
    ok.add_css_class("suggested-action");
    buttons.append(&cancel);
    buttons.append(&ok);
    card.append(&buttons);
    let d = dialog.clone();
    cancel.connect_clicked(move |_| d.close());
    let on_ok = Rc::new(on_ok);
    let submit = {
        let (d, e) = (dialog.clone(), entry.clone());
        move || {
            let text = e.text().trim().to_string();
            if text.is_empty() {
                e.add_css_class("error");
                return;
            }
            d.close();
            on_ok(text);
        }
    };
    let s2 = submit.clone();
    ok.connect_clicked(move |_| s2());
    entry.connect_activate(move |_| submit());
    dialog.present();
    entry.grab_focus();
    entry.select_region(0, -1);
    extra
}

pub fn banner(text: &str, warning: bool) -> gtk::Box {
    let b = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    b.add_css_class("banner");
    if warning {
        b.add_css_class("warning");
    }
    let icon = gtk::Image::from_icon_name(if warning { "dialog-warning-symbolic" } else { "dialog-information-symbolic" });
    icon.set_valign(gtk::Align::Start);
    let l = gtk::Label::new(None);
    l.set_markup(text);
    l.set_wrap(true);
    l.set_xalign(0.0);
    l.set_hexpand(true);
    b.append(&icon);
    b.append(&l);
    b
}

/// A flat icon button for the top bar and dialog headers.
pub fn bar_button(icon: &str, tooltip: &str) -> gtk::Button {
    let b = gtk::Button::from_icon_name(icon);
    b.add_css_class("bar-button");
    b.set_tooltip_text(Some(tooltip));
    b.set_valign(gtk::Align::Center);
    b
}

pub fn button_row(title: &str, desc: &str, label: &str, on_click: impl Fn(&gtk::Button) + 'static) -> (gtk::Box, gtk::Button) {
    let b = gtk::Button::with_label(label);
    b.connect_clicked(on_click);
    let r = row(title, desc, Some(b.upcast_ref()));
    (r, b)
}

pub fn choice_row(
    title: &str,
    desc: &str,
    options: Vec<(String, String)>,
    current: &str,
    on_change: impl Fn(String) + 'static,
) -> (gtk::Box, gtk::DropDown) {
    let dd = dropdown(&options, current);
    dd.connect_selected_notify(move |d| {
        if let Some((id, _)) = options.get(d.selected() as usize) {
            on_change(id.clone());
        }
    });
    let r = row(title, desc, Some(dd.upcast_ref()));
    (r, dd)
}

/// Opens a page's config file; a menu picks one when there are several.
pub fn config_button(files: &[PathBuf]) -> gtk::Widget {
    if let [path] = files {
        let path = path.clone();
        let b = bar_button("text-editor-symbolic", &format!("Open {}", nixe_core::paths::pretty(&path)));
        b.connect_clicked(move |_| nixe_core::cmd::open_in_editor(&path));
        return b.upcast();
    }
    let menu = gtk::MenuButton::new();
    menu.set_icon_name("text-editor-symbolic");
    menu.add_css_class("bar-button");
    menu.set_valign(gtk::Align::Center);
    menu.set_tooltip_text(Some("Open a config file"));
    let list = gtk::Box::new(gtk::Orientation::Vertical, 2);
    let popover = gtk::Popover::new();
    for path in files {
        let b = gtk::Button::with_label(&nixe_core::paths::pretty(path));
        b.add_css_class("flat");
        if let Some(label) = b.child().and_downcast::<gtk::Label>() {
            label.set_xalign(0.0);
        }
        let p = path.clone();
        let pop = popover.clone();
        b.connect_clicked(move |_| {
            pop.popdown();
            nixe_core::cmd::open_in_editor(&p);
        });
        list.append(&b);
    }
    popover.set_child(Some(&list));
    menu.set_popover(Some(&popover));
    menu.upcast()
}

pub fn dropdown(options: &[(String, String)], current: &str) -> gtk::DropDown {
    let labels: Vec<&str> = options.iter().map(|(_, l)| l.as_str()).collect();
    let dd = gtk::DropDown::from_strings(&labels);
    if let Some(i) = options.iter().position(|(id, _)| id == current) {
        dd.set_selected(i as u32);
    } else {
        dd.set_selected(gtk::INVALID_LIST_POSITION);
    }
    dd
}

pub fn empty_state(icon: &str, title: &str, desc: &str, action: Action<'_>) -> gtk::Box {
    let b = vbox(8);
    b.add_css_class("empty-state");
    b.set_valign(gtk::Align::Center);
    b.set_halign(gtk::Align::Center);
    b.set_vexpand(true);
    let i = gtk::Image::from_icon_name(icon);
    i.set_pixel_size(48);
    i.add_css_class("dim");
    b.append(&i);
    let t = gtk::Label::new(Some(title));
    t.add_css_class("empty-title");
    b.append(&t);
    if !desc.is_empty() {
        let d = gtk::Label::new(None);
        d.set_markup(desc);
        d.add_css_class("dim");
        d.set_wrap(true);
        d.set_justify(gtk::Justification::Center);
        d.set_max_width_chars(52);
        b.append(&d);
    }
    if let Some((label, act)) = action {
        let btn = gtk::Button::with_label(label);
        btn.add_css_class("suggested-action");
        btn.set_halign(gtk::Align::Center);
        btn.set_margin_top(8);
        btn.connect_clicked(move |_| act());
        b.append(&btn);
    }
    b
}

pub fn hbox(spacing: i32) -> gtk::Box {
    gtk::Box::new(gtk::Orientation::Horizontal, spacing)
}

/// A flat round icon button with a tooltip.
pub fn icon_button(icon: &str, tooltip: &str) -> gtk::Button {
    let b = gtk::Button::from_icon_name(icon);
    b.add_css_class("flat");
    b.add_css_class("icon-button");
    b.set_tooltip_text(Some(tooltip));
    b.set_valign(gtk::Align::Center);
    b
}

/// Rows of key caps for the keyboard help.
pub fn key_caps(keys: &[&str]) -> gtk::Box {
    let caps = hbox(4);
    for (i, k) in keys.iter().enumerate() {
        if i > 0 {
            caps.append(&label("+", "dim"));
        }
        caps.append(&label(k, "key-cap"));
    }
    caps
}

/// A small key/value pair: dim caption over a mono figure.
pub fn kv(key: &str) -> (gtk::Box, gtk::Label) {
    let b = vbox(1);
    b.append(&label(key, "kv-key"));
    let v = label("–", "kv-value");
    v.add_css_class("mono");
    v.set_ellipsize(pango::EllipsizeMode::End);
    b.append(&v);
    (b, v)
}

/// Lay key figures out in a wrapping grid that fits any width.
pub fn kv_flow(keys: &[&str]) -> (gtk::FlowBox, Vec<gtk::Label>) {
    let flow = gtk::FlowBox::new();
    flow.set_selection_mode(gtk::SelectionMode::None);
    flow.set_homogeneous(true);
    flow.set_min_children_per_line(2);
    flow.set_max_children_per_line(6);
    flow.set_row_spacing(10);
    flow.set_column_spacing(18);
    let mut values = Vec::new();
    for k in keys {
        let (b, v) = kv(k);
        b.set_size_request(110, -1);
        flow.append(&b);
        values.push(v);
    }
    // Children are focusable by default; these are read-only.
    let mut child = flow.first_child();
    while let Some(c) = child {
        c.set_focusable(false);
        child = c.next_sibling();
    }
    (flow, values)
}

pub fn label(text: &str, class: &str) -> gtk::Label {
    let l = gtk::Label::new(Some(text));
    if !class.is_empty() {
        l.add_css_class(class);
    }
    l.set_xalign(0.0);
    l
}

/// A button with an icon and a label.
pub fn labeled_button(icon: &str, text: &str) -> gtk::Button {
    let b = gtk::Button::new();
    let c = hbox(8);
    c.append(&gtk::Image::from_icon_name(icon));
    c.append(&gtk::Label::new(Some(text)));
    b.set_child(Some(&c));
    b
}

/// Add a `.m3u` / `.m3u8` filter to a file dialog.
pub fn m3u_filter() -> gtk::FileFilter {
    let f = gtk::FileFilter::new();
    f.set_name(Some("Playlists (M3U)"));
    f.add_suffix("m3u");
    f.add_suffix("m3u8");
    f
}

pub fn opts(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
    pairs.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect()
}

/// Set a widget's background to a colour (for swatches and previews).
pub fn paint(widget: &impl IsA<gtk::Widget>, css_colour: &str) {
    let provider = gtk::CssProvider::new();
    provider.load_from_string(&format!("* {{ background: {css_colour}; }}"));
    #[allow(deprecated)]
    widget.style_context().add_provider(&provider, gtk::STYLE_PROVIDER_PRIORITY_USER);
}

/// An option card: title and description on the left, control on the right.
pub fn row(title: &str, desc: &str, control: Option<&gtk::Widget>) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 14);
    row.add_css_class("settings-option");
    let text = gtk::Box::new(gtk::Orientation::Vertical, 2);
    text.set_valign(gtk::Align::Center);
    text.set_hexpand(true);
    let t = gtk::Label::new(Some(title));
    t.add_css_class("settings-option-title");
    t.set_xalign(0.0);
    t.set_wrap(true);
    text.append(&t);
    if !desc.is_empty() {
        let d = gtk::Label::new(None);
        d.set_markup(desc);
        d.add_css_class("settings-option-description");
        d.set_xalign(0.0);
        d.set_wrap(true);
        d.set_wrap_mode(pango::WrapMode::WordChar);
        text.append(&d);
    }
    row.append(&text);
    if let Some(c) = control {
        c.set_valign(gtk::Align::Center);
        row.append(c);
    }
    row
}

/// Buttons joined into one control; exactly one is selected.
pub fn segmented(options: &[(String, String)], current: &str, on_change: impl Fn(String) + 'static) -> gtk::Box {
    let bx = hbox(0);
    bx.add_css_class("segmented");
    bx.set_valign(gtk::Align::Center);
    let mut first: Option<gtk::ToggleButton> = None;
    let on_change = Rc::new(on_change);
    for (id, label) in options {
        let b = gtk::ToggleButton::with_label(label);
        b.add_css_class("segment");
        if let Some(f) = &first {
            b.set_group(Some(f));
        } else {
            first = Some(b.clone());
        }
        b.set_active(id == current);
        let id = id.clone();
        let cb = on_change.clone();
        b.connect_toggled(move |b| {
            if b.is_active() {
                cb(id.clone());
            }
        });
        bx.append(&b);
    }
    bx
}

pub fn segmented_row(
    title: &str,
    desc: &str,
    options: Vec<(String, String)>,
    current: &str,
    on_change: impl Fn(String) + 'static,
) -> gtk::Box {
    let seg = segmented(&options, current, on_change);
    row(title, desc, Some(seg.upcast_ref()))
}

pub fn switch_row(title: &str, desc: &str, active: bool, on_change: impl Fn(bool) + 'static) -> (gtk::Box, gtk::Switch) {
    let sw = gtk::Switch::new();
    sw.set_active(active);
    sw.connect_active_notify(move |s| on_change(s.is_active()));
    let r = row(title, desc, Some(sw.upcast_ref()));
    (r, sw)
}

/// A small pill label ("Stopped", "Omarchy", "Failed").
pub fn tag(text: &str, kind: &str) -> gtk::Label {
    let l = gtk::Label::new(Some(text));
    l.add_css_class("tag");
    if !kind.is_empty() {
        l.add_css_class(kind);
    }
    l.set_valign(gtk::Align::Center);
    l
}

/// A button that needs two clicks: the first arms it ("Click again to …"),
/// the second acts. It disarms itself after a few seconds.
pub fn two_click(label_text: &str, armed_text: &str, act: impl Fn() + 'static) -> gtk::Button {
    let b = gtk::Button::with_label(label_text);
    b.add_css_class("destructive-action");
    let armed = Rc::new(std::cell::Cell::new(false));
    let (idle, armed_label) = (label_text.to_string(), armed_text.to_string());
    b.connect_clicked(move |b| {
        if armed.get() {
            armed.set(false);
            b.set_label(&idle);
            act();
            return;
        }
        armed.set(true);
        b.set_label(&armed_label);
        let (b2, armed2, idle2) = (b.clone(), armed.clone(), idle.clone());
        gtk::glib::timeout_add_local_once(std::time::Duration::from_secs(4), move || {
            if armed2.get() {
                armed2.set(false);
                b2.set_label(&idle2);
            }
        });
    });
    b
}

/// A thin usage bar that turns `danger` above 90%.
pub fn usage_bar() -> gtk::LevelBar {
    let bar = gtk::LevelBar::for_interval(0.0, 1.0);
    for name in ["low", "high", "full"] {
        bar.remove_offset_value(Some(name));
    }
    bar.add_offset_value("ok", 0.9);
    bar.add_offset_value("hot", 1.0);
    bar.set_hexpand(true);
    bar.set_valign(gtk::Align::Center);
    bar
}

pub fn vbox(spacing: i32) -> gtk::Box {
    gtk::Box::new(gtk::Orientation::Vertical, spacing)
}
