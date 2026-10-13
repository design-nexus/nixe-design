//! Ctrl+Shift+P: one box to find anything the window can do. It lists the
//! sidebar's pages and every button with a name or a tooltip (the inspector's
//! too), and choosing one presses that button, so it always matches the app.

use crate::widgets;
use gtk::prelude::*;
use gtk::{gdk, glib, pango};
use std::cell::RefCell;
use std::rc::Rc;

/// Most results shown at once.
const LIMIT: usize = 60;

#[derive(Clone)]
struct Item {
    title: String,
    /// The keys, from the tooltip's "(Ctrl+X)".
    keys: String,
    kind: &'static str,
    target: gtk::Widget,
}

/// "Find in the file (Ctrl+F)" → ("Find in the file", "Ctrl+F").
pub(crate) fn split_keys(text: &str) -> (String, String) {
    let text = text.trim();
    if let Some(open) = text.rfind(" (")
        && text.ends_with(')')
    {
        let inside = &text[open + 2..text.len() - 1];
        if inside.contains("Ctrl") || inside.contains("Shift") || inside.contains("Alt") || inside.starts_with('F') {
            return (text[..open].to_string(), inside.to_string());
        }
    }
    (text.to_string(), String::new())
}

/// The text a button shows (with `hidden`, also labels hidden for now, as in
/// the icon-only sidebar).
fn label_of(w: &gtk::Widget, hidden: bool) -> String {
    let mut out = String::new();
    let mut child = w.first_child();
    while let Some(c) = child {
        if let Some(l) = c.downcast_ref::<gtk::Label>()
            && (hidden || l.is_visible())
            && !l.text().is_empty()
        {
            if !out.is_empty() {
                out.push(' ');
            }
            out.push_str(&l.text());
        } else {
            let inner = label_of(&c, hidden);
            if !inner.is_empty() {
                if !out.is_empty() {
                    out.push(' ');
                }
                out.push_str(&inner);
            }
        }
        child = c.next_sibling();
    }
    out
}

/// Visible as far as the window goes (a hidden tab of a stack still counts).
fn reachable(w: &gtk::Widget, root: &gtk::Widget) -> bool {
    let mut cur = Some(w.clone());
    while let Some(c) = cur {
        if &c == root {
            return true;
        }
        let in_stack_page = c.parent().is_some_and(|p| p.is::<gtk::Stack>());
        let hidden_page = in_stack_page && c.has_css_class("contextual-page");
        if !c.is_visible() && (!in_stack_page || hidden_page) && !c.has_css_class("commandbar-menu") {
            return false;
        }
        if !c.is_sensitive() {
            return false;
        }
        cur = c.parent();
    }
    false
}

fn collect(w: &gtk::Widget, root: &gtk::Widget, out: &mut Vec<Item>) {
    // A dropdown's insides aren't commands of their own.
    // (The command bar's ⋯ menu is the exception: its tools count.)
    if w.is::<gtk::DropDown>() || w.is::<gtk::SpinButton>() || (w.is::<gtk::Popover>() && !w.has_css_class("commandbar-menu")) {
        return;
    }
    let is_button = w.is::<gtk::Button>() || w.is::<gtk::MenuButton>();
    if is_button && reachable(w, root) {
        let page = w.has_css_class("nav-item");
        let label = label_of(w, page);
        let tip = w.tooltip_text().map(|t| t.to_string()).unwrap_or_default();
        let (tip_title, keys) = split_keys(&tip);
        let title = if page && !label.is_empty() {
            label
        } else if !tip_title.is_empty() {
            tip_title
        } else {
            label
        };
        if !title.is_empty() && !out.iter().any(|i| i.title == title) {
            let kind = if page { "Page" } else if w.is::<gtk::MenuButton>() { "Menu" } else { "Action" };
            out.push(Item { title, keys, kind, target: w.clone() });
        }
        // A menu button's own inner button is the same thing.
        if w.is::<gtk::MenuButton>() {
            return;
        }
    }
    let mut child = w.first_child();
    while let Some(c) = child {
        collect(&c, root, out);
        child = c.next_sibling();
    }
}

fn rank(items: &[Item], query: &str) -> Vec<Item> {
    let q = query.trim().to_lowercase();
    let mut hits: Vec<(u8, &Item)> = items
        .iter()
        .filter_map(|i| {
            let t = i.title.to_lowercase();
            let score = if q.is_empty() {
                if i.kind == "Page" { 0 } else { 1 }
            } else if t.starts_with(&q) {
                0
            } else if t.split_whitespace().any(|w| w.starts_with(&q)) {
                1
            } else if q.split_whitespace().all(|w| t.contains(w)) {
                2
            } else {
                return None;
            };
            Some((score, i))
        })
        .collect();
    hits.sort_by(|a, b| a.0.cmp(&b.0).then((a.1.kind != "Page").cmp(&(b.1.kind != "Page"))).then(a.1.title.cmp(&b.1.title)));
    hits.into_iter().take(LIMIT).map(|(_, i)| i.clone()).collect()
}

fn row(item: &Item) -> gtk::ListBoxRow {
    let b = widgets::hbox(10);
    b.add_css_class("palette-row");
    let t = widgets::label(&item.title, "");
    t.set_hexpand(true);
    t.set_ellipsize(pango::EllipsizeMode::End);
    b.append(&t);
    if !item.keys.is_empty() {
        b.append(&widgets::label(&item.keys, "dim"));
    }
    b.append(&widgets::tag(item.kind, ""));
    let r = gtk::ListBoxRow::new();
    r.set_child(Some(&b));
    r
}

fn press(w: &gtk::Widget) {
    crate::inspector::reveal(w);
    // A menu button folded into the command bar's ⋯ menu opens from there.
    if w.is::<gtk::MenuButton>()
        && let Some(menu) = w.ancestor(gtk::Popover::static_type()).and_downcast::<gtk::Popover>()
    {
        menu.popup();
    }
    if let Some(m) = w.downcast_ref::<gtk::MenuButton>() {
        m.popup();
    } else if let Some(b) = w.downcast_ref::<gtk::Button>() {
        b.emit_clicked();
    }
}

/// Open the palette for `window`.
pub fn open(window: &impl IsA<gtk::Window>) {
    let window = window.upcast_ref::<gtk::Window>().clone();
    let Some(root) = window.child() else { return };
    let mut items = Vec::new();
    collect(&root, &root, &mut items);

    let (dialog, card) = widgets::dialog("Go to", 560);
    dialog.set_default_height(480);
    card.add_css_class("palette");
    let entry = gtk::SearchEntry::new();
    entry.set_placeholder_text(Some("Pages and actions"));
    card.append(&entry);
    let list = gtk::ListBox::new();
    list.set_selection_mode(gtk::SelectionMode::Single);
    list.add_css_class("palette-list");
    let scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::External)
        .vexpand(true)
        .child(&list)
        .build();
    card.append(&scroll);
    card.append(&widgets::label("Enter goes · ↑↓ chooses · Esc closes", "dim"));

    let items = Rc::new(items);
    let shown: Rc<RefCell<Vec<Item>>> = Rc::default();
    let refresh: Rc<dyn Fn()> = {
        let (items, shown, list, entry) = (items.clone(), shown.clone(), list.clone(), entry.clone());
        Rc::new(move || {
            while let Some(c) = list.first_child() {
                list.remove(&c);
            }
            let ranked = rank(&items, &entry.text());
            for i in &ranked {
                list.append(&row(i));
            }
            if ranked.is_empty() {
                let none = gtk::ListBoxRow::new();
                none.set_child(Some(&widgets::label("Nothing found.", "empty-state")));
                none.set_selectable(false);
                none.set_activatable(false);
                list.append(&none);
            }
            *shown.borrow_mut() = ranked;
            if let Some(first) = list.row_at_index(0) {
                list.select_row(Some(&first));
            }
        })
    };
    refresh();

    let go: Rc<dyn Fn(usize)> = {
        let (shown, dialog) = (shown.clone(), dialog.clone());
        Rc::new(move |index| {
            let Some(item) = shown.borrow().get(index).cloned() else { return };
            dialog.close();
            // After the card is gone, so focus lands back in the window.
            glib::idle_add_local_once(move || press(&item.target));
        })
    };
    let g = go.clone();
    list.connect_row_activated(move |_, r| g(r.index() as usize));
    let r = refresh.clone();
    entry.connect_search_changed(move |_| r());
    let (g, l) = (go.clone(), list.clone());
    entry.connect_activate(move |_| g(l.selected_row().map_or(0, |r| r.index() as usize)));

    let keys = gtk::EventControllerKey::new();
    keys.set_propagation_phase(gtk::PropagationPhase::Capture);
    let (l, sc) = (list.clone(), scroll.clone());
    keys.connect_key_pressed(move |_, key, _, _| {
        if !matches!(key, gdk::Key::Down | gdk::Key::Up) {
            return glib::Propagation::Proceed;
        }
        let index = l.selected_row().map_or(0, |r| r.index());
        let next = if key == gdk::Key::Down { index + 1 } else { (index - 1).max(0) };
        // Focus stays in the search box so typing carries on; scroll by hand.
        if let Some(r) = l.row_at_index(next) {
            l.select_row(Some(&r));
            if let Some(p) = r.compute_point(&l, &gtk::graphene::Point::new(0.0, 0.0)) {
                let adj = sc.vadjustment();
                let (top, h) = (p.y() as f64, r.height() as f64);
                if top < adj.value() {
                    adj.set_value(top);
                } else if top + h > adj.value() + adj.page_size() {
                    adj.set_value(top + h - adj.page_size());
                }
            }
        }
        glib::Propagation::Stop
    });
    dialog.add_controller(keys);
    dialog.present();
    entry.grab_focus();
}

#[cfg(test)]
mod tests {
    use super::split_keys;

    #[test]
    fn splits_keys_from_tooltips() {
        assert_eq!(split_keys("Find (Ctrl+F)"), ("Find".into(), "Ctrl+F".into()));
        assert_eq!(split_keys("Full screen (F11)"), ("Full screen".into(), "F11".into()));
        assert_eq!(split_keys("Shapes (rectangle, arrow)"), ("Shapes (rectangle, arrow)".into(), String::new()));
        assert_eq!(split_keys("Settings"), ("Settings".into(), String::new()));
    }
}
