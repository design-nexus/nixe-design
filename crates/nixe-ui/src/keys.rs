//! Keys every Nixe app answers the same way. Apps keep their own handlers and
//! add from here only the ones they don't have yet.

use gtk::prelude::*;
use gtk::{gdk, glib};

/// Which of the common keys to add.
#[derive(Default)]
pub struct Common {
    /// Ctrl+, opens settings.
    pub settings: Option<fn()>,
    /// ? lists the shortcuts (F1 already does in every app).
    pub shortcuts: Option<fn()>,
    /// Ctrl+Q, and Ctrl+W, close the window.
    pub close: bool,
}

/// True while the keyboard is typing into a text field.
fn typing(window: &gtk::Window) -> bool {
    let Some(focus) = gtk::prelude::GtkWindowExt::focus(window) else { return false };
    focus.is::<gtk::Text>() || focus.is::<gtk::TextView>() || focus.ancestor(gtk::Entry::static_type()).is_some()
}

/// Add the common keys to `window`, before its own handlers see them.
pub fn install(window: &impl IsA<gtk::Window>, common: Common) {
    let window = window.upcast_ref::<gtk::Window>().clone();
    let keys = gtk::EventControllerKey::new();
    keys.set_propagation_phase(gtk::PropagationPhase::Capture);
    let w = window.clone();
    keys.connect_key_pressed(move |_, key, _, mods| {
        let ctrl = mods.contains(gdk::ModifierType::CONTROL_MASK);
        match key {
            gdk::Key::comma if ctrl => {
                if let Some(f) = common.settings {
                    f();
                    return glib::Propagation::Stop;
                }
            }
            gdk::Key::question if !ctrl && !typing(&w) => {
                if let Some(f) = common.shortcuts {
                    f();
                    return glib::Propagation::Stop;
                }
            }
            gdk::Key::q | gdk::Key::w if ctrl && common.close => {
                w.close();
                return glib::Propagation::Stop;
            }
            _ => {}
        }
        glib::Propagation::Proceed
    });
    window.add_controller(keys);
}
