//! Parses the shared stylesheets as an app would load them, v1 and 2.0, for
//! every shipped theme, and prints anything GTK rejects. Needs a display but
//! opens no window: `cargo run -p nyx-theme --example check_css`.

use std::cell::Cell;
use std::rc::Rc;

fn main() {
    gtk::init().expect("no display");
    let errors = Rc::new(Cell::new(0));
    // The checker must catch a bad rule, or a clean run means nothing.
    let probe = gtk::CssProvider::new();
    let caught = Rc::new(Cell::new(false));
    let flag = caught.clone();
    probe.connect_parsing_error(move |_, _, _| flag.set(true));
    probe.load_from_string("label { colr: red; }");
    assert!(caught.get(), "GTK reported no error for a bad rule");
    for theme in nyx_theme::bundled() {
        for v2 in [false, true] {
            let css = nyx_theme::sheet_for_check("check-window", &theme, v2);
            let provider = gtk::CssProvider::new();
            let (id, count) = (theme.id.clone(), errors.clone());
            provider.connect_parsing_error(move |_, section, error| {
                count.set(count.get() + 1);
                let at = section.start_location();
                eprintln!("{id} ({}): line {}: {error}", if v2 { "2.0" } else { "v1" }, at.lines() + 1);
            });
            provider.load_from_string(&css);
        }
    }
    println!("{} parse errors", errors.get());
    std::process::exit(if errors.get() == 0 { 0 } else { 1 });
}
