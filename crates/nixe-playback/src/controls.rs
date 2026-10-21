//! Media Player's playback controls, shared by both video applications.
//! Consumers supply player state and menu actions; construction and CSS live here.
use gtk::{glib, prelude::*};
use nixe_ui::widgets;

pub const CSS: &str = include_str!("controls.css");

/// Installed once with each app's theme provider, after the shared base rules.
pub fn stylesheet(app_css: &str) -> &'static str {
    // Theme installation runs once per process and requires a static stylesheet.
    Box::leak(format!("{CSS}\n{}\n{app_css}", nixe_ui::media::CSS).into_boxed_str())
}

pub fn volume_icon(v: f64, muted: bool) -> &'static str {
    if muted || v <= 0.001 {
        "audio-volume-muted-symbolic"
    } else if v < 0.34 {
        "audio-volume-low-symbolic"
    } else if v < 0.67 {
        "audio-volume-medium-symbolic"
    } else {
        "audio-volume-high-symbolic"
    }
}

pub fn time(secs: f64) -> String {
    let s = secs.max(0.0) as u64;
    let (h, m, sec) = (s / 3600, (s % 3600) / 60, s % 60);
    if h > 0 { format!("{h}:{m:02}:{sec:02}") } else { format!("{m}:{sec:02}") }
}

pub fn choice(label: &str, detail: &str, selected: bool, on: impl Fn() + 'static) -> gtk::Button {
    let b = gtk::Button::new();
    b.add_css_class("flat");
    b.add_css_class("menu-item");
    let row = widgets::hbox(10);
    let check = gtk::Image::from_icon_name("object-select-symbolic");
    check.set_opacity(if selected { 1.0 } else { 0.0 });
    check.add_css_class("accent-text");
    row.append(&check);
    let l = widgets::label(label, "");
    l.set_hexpand(true);
    l.set_ellipsize(gtk::pango::EllipsizeMode::End);
    l.set_max_width_chars(32);
    row.append(&l);
    if !detail.is_empty() {
        let d = widgets::label(detail, "dim");
        d.add_css_class("mono");
        row.append(&d);
    }
    b.set_child(Some(&row));
    b.connect_clicked(move |_| on());
    b
}
fn menu_button(icon: &str, tooltip: &str) -> gtk::MenuButton {
    let mb = gtk::MenuButton::new();
    mb.set_icon_name(icon);
    mb.set_tooltip_text(Some(tooltip));
    mb.add_css_class("flat");
    mb.add_css_class("osd-button");
    mb.set_focus_on_click(false);
    let pop = gtk::Popover::new();
    pop.add_css_class("menu-popover");
    let content = widgets::vbox(1);
    pop.set_child(Some(&content));
    mb.set_popover(Some(&pop));
    mb
}

/// Fill a menu fresh every time it opens, as Media Player does.
pub fn fill_menu(button: &gtk::MenuButton, fill: impl Fn(&gtk::Box, &gtk::Popover) + 'static) {
    let pop = button.popover().expect("playback menus have popovers");
    let content = pop.child().and_downcast::<gtk::Box>().expect("playback menu content");
    pop.connect_show(move |pop| {
        while let Some(child) = content.first_child() {
            content.remove(&child);
        }
        fill(&content, pop);
    });
}

pub struct SeekWidgets {
    pub root: gtk::Box,
    pub pos: gtk::Label,
    pub seek: gtk::Scale,
    pub dur: gtk::Label,
}
impl Default for SeekWidgets {
    fn default() -> Self {
        Self::new()
    }
}
impl SeekWidgets {
    pub fn new() -> Self {
        let root = widgets::hbox(10);
        root.add_css_class("seek-row");
        let pos = widgets::label("0:00", "time-readout");
        pos.add_css_class("mono");
        pos.set_xalign(1.0);
        pos.set_width_chars(7);
        let seek = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 1.0, 1.0);
        seek.set_draw_value(false);
        seek.set_hexpand(true);
        seek.add_css_class("seek");
        seek.set_focus_on_click(false);
        let dur = widgets::label("0:00", "time-readout");
        dur.add_css_class("mono");
        dur.add_css_class("clickable");
        dur.set_width_chars(8);
        dur.set_xalign(0.0);
        dur.set_tooltip_text(Some("Show the time left or the length"));
        dur.set_cursor_from_name(Some("pointer"));
        root.append(&pos);
        root.append(&seek);
        root.append(&dur);

        Self { root, pos, seek, dur }
    }
}

pub struct OsdControls {
    pub root: gtk::Box,
    pub prev: gtk::Button,
    pub back: gtk::Button,
    pub play: gtk::Button,
    pub fwd: gtk::Button,
    pub next: gtk::Button,
    pub mute: gtk::Button,
    pub volume: gtk::Scale,
    pub speed_label: gtk::Label,
    pub chapters: gtk::MenuButton,
    pub shape: gtk::MenuButton,
    pub speed: gtk::MenuButton,
    pub audio: gtk::MenuButton,
    pub subs: gtk::MenuButton,
    pub full: gtk::Button,
    pub close: gtk::Button,
}
impl OsdControls {
    pub fn new(seek: &impl IsA<gtk::Widget>, initial_volume: f64, max_volume: f64, muted: bool) -> Self {
        let root = widgets::vbox(6);
        root.add_css_class("osd-controls");
        root.set_valign(gtk::Align::End);
        root.append(seek);
        let row = widgets::hbox(4);
        let prev = widgets::icon_button("media-skip-backward-symbolic", "Previous (Ctrl+←)");
        let back = widgets::icon_button("media-seek-backward-symbolic", "Back (←)");
        let play = gtk::Button::from_icon_name("media-playback-start-symbolic");
        play.add_css_class("play-button");
        let fwd = widgets::icon_button("media-seek-forward-symbolic", "Forward (→)");
        let next = widgets::icon_button("media-skip-forward-symbolic", "Next (Ctrl+→)");
        for b in [&prev, &back, &play, &fwd, &next] {
            b.set_focus_on_click(false);
            b.add_css_class("osd-button");
            row.append(b);
        }
        let mute = widgets::icon_button(volume_icon(initial_volume, muted), "Mute (M)");
        mute.add_css_class("osd-button");
        mute.set_focus_on_click(false);
        mute.set_margin_start(10);
        row.append(&mute);
        let volume = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, max_volume, 0.01);
        volume.set_draw_value(false);
        volume.set_value(initial_volume);
        volume.set_size_request(96, -1);
        volume.add_css_class("volume");
        volume.set_focus_on_click(false);
        row.append(&volume);
        let spacer = widgets::hbox(0);
        spacer.set_hexpand(true);
        row.append(&spacer);
        let speed_label = widgets::label("", "osd-speed");
        speed_label.add_css_class("mono");
        row.append(&speed_label);
        let chapters = menu_button("view-list-ordered-symbolic", "Chapters (Page Up and Page Down)");
        chapters.set_visible(false);
        let shape = menu_button("video-display-symbolic", "Picture: shape, fill, turn");
        let speed = menu_button("nmp-speed-symbolic", "Speed ([ and ])");
        let audio = menu_button("nmp-audio-track-symbolic", "Sound: track, sync and night mode (A)");
        let subs = menu_button("nmp-subtitles-symbolic", "Subtitles (S)");
        let full = widgets::icon_button("view-fullscreen-symbolic", "Fullscreen (F)");
        full.add_css_class("osd-button");
        full.set_focus_on_click(false);
        let close = widgets::icon_button("window-close-symbolic", "Close video");
        close.add_css_class("osd-button");
        close.set_focus_on_click(false);
        row.append(&chapters);
        row.append(&shape);
        row.append(&speed);
        row.append(&audio);
        row.append(&subs);
        row.append(&full);
        row.append(&close);
        root.append(&row);
        Self { root, prev, back, play, fwd, next, mute, volume, speed_label, chapters, shape, speed, audio, subs, full, close }
    }
}

pub fn install_icons() {
    let dir = nixe_core::paths::cache_dir().join("playback-icons");
    for (name, svg) in [
        ("nmp-speed-symbolic.svg", include_str!("../data/nmp-speed-symbolic.svg")),
        ("nmp-audio-track-symbolic.svg", include_str!("../data/nmp-audio-track-symbolic.svg")),
        ("nmp-subtitles-symbolic.svg", include_str!("../data/nmp-subtitles-symbolic.svg")),
    ] {
        let path = dir.join(name);
        if std::fs::read_to_string(&path).ok().as_deref() != Some(svg) {
            let _ = nixe_core::cmd::atomic_write(&path, svg);
        }
    }
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::IconTheme::for_display(&display).add_search_path(dir);
    }
}

/// Media Player's shared pointer/idle behavior. Keep controls available while paused,
/// hovered or a menu is open. The callback lets each app lift its subtitles.
pub fn install_autohide(
    overlay: &gtk::Overlay,
    top: &gtk::Box,
    controls: &gtk::Box,
    menus: Vec<gtk::MenuButton>,
    is_playing: std::rc::Rc<dyn Fn() -> bool>,
    place_subs: std::rc::Rc<dyn Fn(bool)>,
) -> std::rc::Rc<dyn Fn()> {
    use std::{cell::Cell, rc::Rc};
    // Controls fade out while playing and the pointer rests.
    let idle_gen: Rc<Cell<u32>> = Rc::default();
    let over_controls = Rc::new(Cell::new(false));
    let wake: Rc<dyn Fn()> = {
        let (overlay, top, controls, idle_gen, over_controls, place_subs) =
            (overlay.clone(), top.clone(), controls.clone(), idle_gen.clone(), over_controls.clone(), place_subs.clone());
        Rc::new(move || {
            for w in [top.upcast_ref::<gtk::Widget>(), controls.upcast_ref()] {
                w.remove_css_class("idle");
                w.set_can_target(true);
            }
            place_subs(true);
            overlay.set_cursor(None);
            let g = idle_gen.get().wrapping_add(1);
            idle_gen.set(g);
            let is_playing = is_playing.clone();
            let (overlay, top, controls, idle_gen, over, menus, place_subs) = (
                overlay.clone(),
                top.clone(),
                controls.clone(),
                idle_gen.clone(),
                over_controls.clone(),
                menus.clone(),
                place_subs.clone(),
            );
            glib::timeout_add_local_once(std::time::Duration::from_millis(2500), move || {
                // Stay while a menu is open.
                let busy_menu = menus.iter().any(|m| m.is_active());
                if idle_gen.get() != g || over.get() || busy_menu || !is_playing() {
                    return;
                }
                for w in [top.upcast_ref::<gtk::Widget>(), controls.upcast_ref()] {
                    w.add_css_class("idle");
                    w.set_can_target(false);
                }
                place_subs(false);
                overlay.set_cursor_from_name(Some("none"));
            });
        })
    };
    let motion = gtk::EventControllerMotion::new();
    let last_xy = Rc::new(Cell::new((0.0, 0.0)));
    let w = wake.clone();
    motion.connect_motion(move |_, x, y| {
        // Some compositors send motion with no movement; ignore those.
        let (lx, ly) = last_xy.get();
        if (x - lx).abs() + (y - ly).abs() > 1.0 {
            last_xy.set((x, y));
            w();
        }
    });
    overlay.add_controller(motion);
    let over = gtk::EventControllerMotion::new();
    let o = over_controls.clone();
    over.connect_enter(move |_, _, _| o.set(true));
    let o = over_controls.clone();
    over.connect_leave(move |_| o.set(false));
    controls.add_controller(over);

    wake
}
