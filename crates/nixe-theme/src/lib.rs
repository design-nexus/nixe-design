//! Theming for Nixe apps. Every stylesheet is written against the semantic
//! colour tokens (`@theme_bg`, `@theme_accent`, …); a theme is just a value
//! for each. An app hands [`install`] its stylesheet and a way to read its
//! theme preferences, and gets live restyling, including Follow Omarchy.

use gtk::prelude::*;
use gtk::{gdk, glib};
use nixe_core::paths;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::time::SystemTime;

/// Where the colours come from.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeMode {
    /// Follow the active Omarchy theme live.
    Omarchy,
    /// Use a bundled or custom theme.
    Theme,
}

/// The theme-related part of an app's preferences.
#[derive(Debug, Clone)]
pub struct ThemePrefs {
    pub mode: ThemeMode,
    /// Theme id, used when `mode` is [`ThemeMode::Theme`].
    pub theme: String,
    pub glow: bool,
    pub reduce_motion: bool,
}

/// What an app supplies to [`install`].
pub struct Style {
    /// The CSS class of the app's main window, which scopes the shared rules.
    pub window_class: &'static str,
    /// Shared rules the app styles its own way, by selector (as written in
    /// `base.css`, with `@APP@`), left out of its sheet.
    pub base_skip: &'static [&'static str],
    /// The app's stylesheet, loaded after the shared one.
    pub css: &'static str,
    /// Extra rules loaded when the Glow preference is off.
    pub no_glow_css: &'static str,
    /// Extra `@define-color` lines derived from the palette, if the app needs any.
    pub extra_colors: Option<fn(&Palette) -> String>,
    /// Reads the app's current theme preferences.
    pub prefs: fn() -> ThemePrefs,
    /// Stores a theme chosen in another Nixe app in this app's preferences.
    pub set_theme: Option<fn(ThemeMode, String)>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Palette {
    pub bg: String,
    pub surface: String,
    pub text: String,
    pub dim_text: String,
    pub accent: String,
    pub border: String,
    pub muted: String,
    pub highlight: String,
    pub danger: String,
    pub glow: String,
    pub shadow: String,
    #[serde(default)]
    pub light: bool,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub id: String,
    pub name: String,
    pub palette: Palette,
}

macro_rules! palette {
    ($bg:expr, $surface:expr, $text:expr, $dim:expr, $accent:expr, $border:expr, $muted:expr,
     $highlight:expr, $danger:expr, $glow:expr, $shadow:expr, $light:expr) => {
        Palette {
            bg: $bg.into(),
            surface: $surface.into(),
            text: $text.into(),
            dim_text: $dim.into(),
            accent: $accent.into(),
            border: $border.into(),
            muted: $muted.into(),
            highlight: $highlight.into(),
            danger: $danger.into(),
            glow: $glow.into(),
            shadow: $shadow.into(),
            light: $light,
        }
    };
}

fn theme(id: &str, name: &str, palette: Palette) -> Theme {
    Theme { id: id.into(), name: name.into(), palette }
}

pub fn bundled() -> Vec<Theme> {
    vec![
        theme(
            "dracula",
            "Dracula",
            palette!(
                "#282a36", "#21222c", "#f8f8f2", "#6272a4", "#bd93f9", "#44475a", "#44475a", "#343746", "#ff5555", "#bd93f9",
                "#191a21", false
            ),
        ),
        theme(
            "catppuccin-mocha",
            "Catppuccin Mocha",
            palette!(
                "#1e1e2e", "#181825", "#cdd6f4", "#7f849c", "#89b4fa", "#313244", "#45475a", "#313244", "#f38ba8", "#89b4fa",
                "#11111b", false
            ),
        ),
        theme(
            "catppuccin-macchiato",
            "Catppuccin Macchiato",
            palette!(
                "#24273a", "#1e2030", "#cad3f5", "#8087a2", "#8aadf4", "#363a4f", "#494d64", "#363a4f", "#ed8796", "#8aadf4",
                "#181926", false
            ),
        ),
        theme(
            "catppuccin-frappe",
            "Catppuccin Frappé",
            palette!(
                "#303446", "#292c3c", "#c6d0f5", "#838ba7", "#8caaee", "#414559", "#51576d", "#414559", "#e78284", "#8caaee",
                "#232634", false
            ),
        ),
        theme(
            "catppuccin-latte",
            "Catppuccin Latte",
            palette!(
                "#eff1f5", "#e6e9ef", "#4c4f69", "#8c8fa1", "#1e66f5", "#ccd0da", "#bcc0cc", "#dce0e8", "#d20f39", "#1e66f5",
                "#9ca0b0", true
            ),
        ),
        theme(
            "tokyo-night",
            "Tokyo Night",
            palette!(
                "#1a1b26", "#16161e", "#c0caf5", "#565f89", "#7aa2f7", "#292e42", "#3b4261", "#283457", "#f7768e", "#7aa2f7",
                "#0f0f14", false
            ),
        ),
        theme(
            "tokyo-night-storm",
            "Tokyo Night Storm",
            palette!(
                "#24283b", "#1f2335", "#c0caf5", "#565f89", "#7aa2f7", "#292e42", "#3b4261", "#2e3c64", "#f7768e", "#7aa2f7",
                "#1b1e2d", false
            ),
        ),
        theme(
            "tokyo-night-moon",
            "Tokyo Night Moon",
            palette!(
                "#222436", "#1e2030", "#c8d3f5", "#636da6", "#82aaff", "#2f334d", "#3b4261", "#2d3f76", "#ff757f", "#82aaff",
                "#191a2a", false
            ),
        ),
        theme(
            "one-dark-pro",
            "One Dark Pro",
            palette!(
                "#282c34", "#21252b", "#abb2bf", "#5c6370", "#61afef", "#3e4451", "#4b5263", "#2c313a", "#e06c75", "#61afef",
                "#181a1f", false
            ),
        ),
        theme(
            "nord",
            "Nord",
            palette!(
                "#2e3440", "#292e39", "#eceff4", "#7b88a1", "#88c0d0", "#3b4252", "#434c5e", "#3b4252", "#bf616a", "#88c0d0",
                "#242933", false
            ),
        ),
        theme(
            "gruvbox-dark",
            "Gruvbox Dark",
            palette!(
                "#282828", "#1d2021", "#ebdbb2", "#928374", "#fabd2f", "#3c3836", "#504945", "#3c3836", "#fb4934", "#fabd2f",
                "#141414", false
            ),
        ),
        theme(
            "rose-pine",
            "Rosé Pine",
            palette!(
                "#191724", "#1f1d2e", "#e0def4", "#6e6a86", "#c4a7e7", "#26233a", "#403d52", "#26233a", "#eb6f92", "#c4a7e7",
                "#111019", false
            ),
        ),
        theme(
            "everforest",
            "Everforest",
            palette!(
                "#2d353b", "#272e33", "#d3c6aa", "#859289", "#a7c080", "#3d484d", "#475258", "#3d484d", "#e67e80", "#a7c080",
                "#1e2326", false
            ),
        ),
        theme(
            "solarized-dark",
            "Solarized Dark",
            palette!(
                "#002b36", "#00232c", "#93a1a1", "#586e75", "#268bd2", "#073642", "#0f4a56", "#073642", "#dc322f", "#268bd2",
                "#001e26", false
            ),
        ),
        theme(
            "kanagawa",
            "Kanagawa",
            palette!(
                "#1f1f28", "#16161d", "#dcd7ba", "#727169", "#7e9cd8", "#2a2a37", "#363646", "#2d4f67", "#e82424", "#7e9cd8",
                "#0d0c0c", false
            ),
        ),
    ]
}

#[derive(Deserialize)]
struct CustomFile {
    name: String,
    #[serde(flatten)]
    palette: Palette,
}

/// User themes in `~/.config/nixe/themes/*.toml` (for every app) and
/// `~/.config/<app>/themes/*.toml` (this one; same name wins).
pub fn custom() -> Vec<Theme> {
    let mut themes: Vec<Theme> = [paths::suite_dir().join("themes"), paths::custom_themes_dir()]
        .iter()
        .filter_map(|dir| std::fs::read_dir(dir).ok())
        .flat_map(|entries| entries.flatten())
        .filter(|e| e.path().extension().is_some_and(|x| x == "toml"))
        .filter_map(|e| {
            let text = std::fs::read_to_string(e.path()).ok()?;
            let file: CustomFile = toml::from_str(&text).ok()?;
            let id = format!("custom:{}", e.path().file_stem()?.to_string_lossy());
            Some(Theme { id, name: file.name, palette: file.palette })
        })
        .collect();
    // The app's own copy of a theme comes later in the list and wins.
    let mut seen = std::collections::HashSet::new();
    themes.reverse();
    themes.retain(|t| seen.insert(t.id.clone()));
    themes.sort_by(|a, b| a.name.cmp(&b.name));
    themes
}

/// Settings every Nixe app shares (`~/.config/nixe/theme.toml`): the theme,
/// and how dense and large the interface is.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Suite {
    pub mode: ThemeMode,
    pub theme: String,
    /// `comfortable` or `compact`.
    pub density: String,
    /// Text size as a multiple of the normal size, 0.9 to 1.25.
    pub text_scale: f64,
}

impl Default for Suite {
    fn default() -> Self {
        Self { mode: ThemeMode::Omarchy, theme: "tokyo-night".into(), density: "comfortable".into(), text_scale: 1.0 }
    }
}

/// The suite's density and text size as CSS for one app, loaded after its own
/// stylesheet so they apply on top.
pub fn suite_css(window_class: &str, suite: &Suite) -> String {
    let mut css = String::new();
    let scale = suite.text_scale.clamp(0.9, 1.25);
    if (scale - 1.0).abs() > 0.001 {
        css.push_str(&format!("window.{window_class} {{ font-size: {scale:.3}em; }}\n"));
    }
    if suite.density == "compact" {
        css.push_str(&COMPACT.replace("@APP@", window_class));
    }
    css
}

/// Compact density: the same layout, tighter.
const COMPACT: &str = "
window.@APP@ .settings-navigation button.nav-item { min-height: 34px; }
window.@APP@ .status-bar { min-height: 28px; }
window.@APP@ .toolbar, window.@APP@ .list-toolbar { min-height: 38px; }
window.@APP@ .settings-option { min-height: 40px; padding-top: 6px; padding-bottom: 6px; }
window.@APP@ columnview > listview > row { min-height: 28px; }
window.@APP@ .commandbar { min-height: 40px; }
window.@APP@ .inspector-section { padding-top: 10px; padding-bottom: 12px; }
";

pub fn suite_file() -> std::path::PathBuf {
    paths::suite_dir().join("theme.toml")
}

/// The shared settings, if any app has written them yet.
pub fn read_suite() -> Option<Suite> {
    toml::from_str(&std::fs::read_to_string(suite_file()).ok()?).ok()
}

/// Change the shared settings; every open Nixe app follows within a second.
pub fn update_suite(change: impl FnOnce(&mut Suite)) {
    let mut suite = read_suite().unwrap_or_default();
    change(&mut suite);
    if let Ok(text) = toml::to_string_pretty(&suite) {
        let _ = nixe_core::cmd::atomic_write(&suite_file(), text);
    }
}

pub fn all() -> Vec<Theme> {
    let mut themes = bundled();
    themes.extend(custom());
    themes
}

/// Map Omarchy Quattro's `colors.toml` onto the semantic tokens.
pub fn from_omarchy_colors(text: &str) -> Option<Palette> {
    let table: toml::Table = toml::from_str(text).ok()?;
    let get = |key: &str| table.get(key).and_then(|v| v.as_str()).map(str::to_string);
    let background = get("background")?;
    let foreground = get("foreground")?;
    let accent = get("accent").or_else(|| get("blue"))?;
    let light = table.get("mode").and_then(|v| v.as_str()) == Some("light");
    Some(Palette {
        surface: get("dark_background").unwrap_or_else(|| background.clone()),
        dim_text: get("dark_foreground").or_else(|| get("muted")).unwrap_or_else(|| foreground.clone()),
        border: get("lighter_background").or_else(|| get("selection")).unwrap_or_else(|| foreground.clone()),
        muted: get("muted").or_else(|| get("lighter_background")).unwrap_or_else(|| background.clone()),
        highlight: get("selection").or_else(|| get("lighter_background")).unwrap_or_else(|| background.clone()),
        danger: get("red").unwrap_or_else(|| "#e06c75".into()),
        shadow: get("darker_background").unwrap_or_else(|| "#000000".into()),
        glow: accent.clone(),
        bg: background,
        text: foreground,
        accent,
        light,
    })
}

pub fn omarchy_palette() -> Option<Palette> {
    from_omarchy_colors(&std::fs::read_to_string(paths::omarchy_colors()).ok()?)
}

pub fn omarchy_available() -> bool {
    omarchy_palette().is_some()
}

/// The palette that should be showing right now according to the prefs.
pub fn current_palette() -> Palette {
    let prefs = config_prefs();
    if prefs.mode == ThemeMode::Omarchy
        && let Some(p) = omarchy_palette()
    {
        return p;
    }
    let themes = all();
    themes
        .iter()
        .find(|t| t.id == prefs.theme)
        .or_else(|| themes.iter().find(|t| t.id == "tokyo-night"))
        .map(|t| t.palette.clone())
        .expect("bundled themes are never empty")
}

pub fn palette_css(p: &Palette) -> String {
    format!(
        "@define-color theme_bg {};\n@define-color theme_surface {};\n@define-color theme_text {};\n\
         @define-color theme_dim_text {};\n@define-color theme_accent {};\n@define-color theme_border {};\n\
         @define-color theme_muted {};\n@define-color theme_highlight {};\n@define-color theme_danger {};\n\
         @define-color theme_glow {};\n@define-color theme_shadow {};\n\
         @define-color theme_on_accent {};\n",
        p.bg,
        p.surface,
        p.text,
        p.dim_text,
        p.accent,
        p.border,
        p.muted,
        p.highlight,
        p.danger,
        p.glow,
        p.shadow,
        if p.light { "#ffffff" } else { &p.bg },
    )
}

/// The stylesheet every app shares.
pub const BASE_CSS: &str = include_str!("base.css");

/// The shared stylesheet for one app: scoped to its window class, without the
/// rules it styles its own way.
pub fn base_css(window_class: &str, skip: &[&str]) -> String {
    let mut out = String::with_capacity(BASE_CSS.len());
    for block in blocks(BASE_CSS) {
        let selector = block.split('{').next().unwrap_or_default();
        let selector = strip_comments(selector).split_whitespace().collect::<Vec<_>>().join(" ");
        if block.contains('{') && skip.contains(&selector.as_str()) {
            continue;
        }
        out.push_str(block);
    }
    out.replace("@APP@", window_class)
}

/// Top-level statements and rule blocks, each with the comments before it.
fn blocks(css: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let (mut depth, mut start, mut i) = (0usize, 0usize, 0usize);
    let bytes = css.as_bytes();
    while i < bytes.len() {
        if css[i..].starts_with("/*") {
            i = css[i..].find("*/").map_or(bytes.len(), |j| i + j + 2);
            continue;
        }
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    out.push(&css[start..=i]);
                    start = i + 1;
                }
            }
            b';' if depth == 0 => {
                out.push(&css[start..=i]);
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    out.push(&css[start..]);
    out
}

fn strip_comments(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(i) = rest.find("/*") {
        out.push_str(&rest[..i]);
        rest = rest[i..].find("*/").map_or("", |j| &rest[i + j + 2..]);
    }
    out.push_str(rest);
    out
}

type Listener = Box<dyn Fn()>;

thread_local! {
    static STYLE: RefCell<Option<Style>> = const { RefCell::new(None) };
    static PROVIDER: RefCell<Option<gtk::CssProvider>> = const { RefCell::new(None) };
    static ACTIVE: RefCell<Option<Palette>> = const { RefCell::new(None) };
    static LISTENERS: RefCell<Vec<(glib::WeakRef<gtk::Widget>, Listener)>> = const { RefCell::new(Vec::new()) };
    static GLOBAL: RefCell<Vec<Listener>> = const { RefCell::new(Vec::new()) };
    /// The theme this app and the suite file last agreed on.
    static SYNCED: RefCell<Option<(ThemeMode, String)>> = const { RefCell::new(None) };
}

fn config_prefs() -> ThemePrefs {
    STYLE.with(|s| s.borrow().as_ref().map(|s| (s.prefs)())).unwrap_or(ThemePrefs {
        mode: ThemeMode::Omarchy,
        theme: "tokyo-night".into(),
        glow: true,
        reduce_motion: false,
    })
}

/// The palette currently on screen (cached; graphs read it on every frame).
pub fn palette() -> Palette {
    ACTIVE.with(|a| a.borrow_mut().get_or_insert_with(current_palette).clone())
}

/// Call `f` whenever the palette changes, for as long as `owner` lives.
pub fn subscribe(owner: &impl IsA<gtk::Widget>, f: impl Fn() + 'static) {
    LISTENERS.with(|l| l.borrow_mut().push((owner.upcast_ref::<gtk::Widget>().downgrade(), Box::new(f))));
}

/// Call `f` after every restyle for the rest of the run.
pub fn on_change(f: impl Fn() + 'static) {
    GLOBAL.with(|g| g.borrow_mut().push(Box::new(f)));
}

/// Install the app's stylesheet and the current palette on the default display.
pub fn install(style: Style) {
    // Start with the suite's theme if another app has chosen one.
    let mine = {
        let p = (style.prefs)();
        (p.mode, p.theme)
    };
    let synced = match read_suite() {
        Some(suite) if (suite.mode.clone(), suite.theme.clone()) != mine => {
            if let Some(set) = style.set_theme {
                set(suite.mode.clone(), suite.theme.clone());
            }
            (suite.mode, suite.theme)
        }
        Some(_) => mine,
        None => {
            update_suite(|s| {
                s.mode = mine.0.clone();
                s.theme = mine.1.clone();
            });
            mine
        }
    };
    SYNCED.with(|s| *s.borrow_mut() = Some(synced));
    STYLE.with(|s| *s.borrow_mut() = Some(style));
    let display = gdk::Display::default().expect("no display");
    let provider = gtk::CssProvider::new();
    gtk::style_context_add_provider_for_display(&display, &provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
    PROVIDER.with(|p| *p.borrow_mut() = Some(provider));
    apply();
    watch_omarchy();
}

/// Re-read prefs and restyle live. The palette and the stylesheet are loaded as
/// one sheet so every `@theme_*` reference resolves against the new colours.
pub fn apply() {
    let prefs = config_prefs();
    // A theme chosen in this app goes to every app.
    let mine = (prefs.mode.clone(), prefs.theme.clone());
    if SYNCED.with(|s| s.borrow().as_ref() != Some(&mine)) {
        update_suite(|s| {
            s.mode = mine.0.clone();
            s.theme = mine.1.clone();
        });
        SYNCED.with(|s| *s.borrow_mut() = Some(mine));
    }
    let palette = current_palette();
    ACTIVE.with(|a| *a.borrow_mut() = Some(palette.clone()));
    let mut css = palette_css(&palette);
    STYLE.with(|s| {
        if let Some(style) = s.borrow().as_ref() {
            if let Some(extra) = style.extra_colors {
                css.push_str(&extra(&palette));
            }
            css.push_str(&base_css(style.window_class, style.base_skip));
            css.push_str(style.css);
            css.push_str(&suite_css(style.window_class, &read_suite().unwrap_or_default()));
            if !prefs.glow {
                css.push('\n');
                css.push_str(style.no_glow_css);
            }
        }
    });
    PROVIDER.with(|p| {
        if let Some(provider) = p.borrow().as_ref() {
            provider.load_from_string(&css);
        }
    });
    if let Some(settings) = gtk::Settings::default() {
        settings.set_gtk_application_prefer_dark_theme(!palette.light);
        settings.set_gtk_enable_animations(!prefs.reduce_motion);
    }
    LISTENERS.with(|l| {
        l.borrow_mut().retain(|(w, _)| w.upgrade().is_some());
        for (_, f) in l.borrow().iter() {
            f();
        }
    });
    GLOBAL.with(|g| {
        for f in g.borrow().iter() {
            f();
        }
    });
}

fn stamp() -> Option<(SystemTime, String)> {
    let colors = paths::omarchy_colors();
    let modified = std::fs::metadata(&colors).and_then(|m| m.modified()).ok()?;
    let name = std::fs::read_to_string(paths::state_home().join("omarchy/current/theme.name")).unwrap_or_default();
    Some((modified, name))
}

/// Omarchy swaps the theme by rewriting its state directory; a light poll is the
/// most robust way to notice every variant of that.
fn watch_omarchy() {
    let last = RefCell::new(stamp());
    let suite_stamp = || std::fs::metadata(suite_file()).and_then(|m| m.modified()).ok();
    let last_suite = RefCell::new(suite_stamp());
    glib::timeout_add_seconds_local(1, move || {
        let now = stamp();
        if now != *last.borrow() {
            *last.borrow_mut() = now;
            if config_prefs().mode == ThemeMode::Omarchy {
                apply();
            }
        }
        // Another Nixe app changed the shared theme.
        let now = suite_stamp();
        if now != *last_suite.borrow() {
            *last_suite.borrow_mut() = now;
            if let Some(suite) = read_suite() {
                let theirs = (suite.mode.clone(), suite.theme.clone());
                if SYNCED.with(|s| s.borrow().as_ref() != Some(&theirs)) {
                    if let Some(set) = STYLE.with(|s| s.borrow().as_ref().and_then(|s| s.set_theme)) {
                        set(suite.mode, suite.theme);
                    }
                    SYNCED.with(|s| *s.borrow_mut() = Some(theirs));
                }
                apply();
            }
        }
        glib::ControlFlow::Continue
    });
}

/// An RGB colour in 0..1, for drawing with cairo.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgb(pub f64, pub f64, pub f64);

impl Rgb {
    pub fn hex(s: &str) -> Rgb {
        let h = s.trim().trim_start_matches('#');
        let v = |i: usize| h.get(i..i + 2).and_then(|x| u8::from_str_radix(x, 16).ok()).unwrap_or(0) as f64 / 255.0;
        if h.len() == 3 {
            let c = |i: usize| h.get(i..i + 1).and_then(|x| u8::from_str_radix(x, 16).ok()).unwrap_or(0) as f64 / 15.0;
            return Rgb(c(0), c(1), c(2));
        }
        Rgb(v(0), v(2), v(4))
    }

    pub fn to_hex(self) -> String {
        let c = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        format!("#{:02x}{:02x}{:02x}", c(self.0), c(self.1), c(self.2))
    }

    /// `mix(self, other, t)`, as in GTK CSS.
    pub fn mix(self, other: Rgb, t: f64) -> Rgb {
        Rgb(self.0 + (other.0 - self.0) * t, self.1 + (other.1 - self.1) * t, self.2 + (other.2 - self.2) * t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_omarchy_colors() {
        let text = r##"
            mode = "dark"
            accent = "#89b4fa"
            selection = "#45475a"
            muted = "#585b70"
            background = "#1e1e2e"
            dark_background = "#161622"
            darker_background = "#101019"
            lighter_background = "#313244"
            foreground = "#cdd6f4"
            dark_foreground = "#6c7086"
            red = "#f38ba8"
        "##;
        let p = from_omarchy_colors(text).unwrap();
        assert_eq!(p.bg, "#1e1e2e");
        assert_eq!(p.surface, "#161622");
        assert_eq!(p.text, "#cdd6f4");
        assert_eq!(p.dim_text, "#6c7086");
        assert_eq!(p.accent, "#89b4fa");
        assert_eq!(p.border, "#313244");
        assert_eq!(p.highlight, "#45475a");
        assert_eq!(p.danger, "#f38ba8");
        assert_eq!(p.glow, "#89b4fa");
        assert!(!p.light);
    }

    #[test]
    fn rejects_incomplete_colors() {
        assert!(from_omarchy_colors("accent = \"#fff\"").is_none());
    }

    #[test]
    fn base_css_scopes_and_skips() {
        let all = base_css("music-window", &[]);
        assert!(all.contains("window.music-window {"));
        assert!(!all.contains("@APP@"));
        let without = base_css("music-window", &[".toast"]);
        assert!(all.contains(".toast {") && !without.contains("\n.toast {"));
        assert!(without.contains(".toast-"), "only the exact selector is dropped");
    }

    #[test]
    fn bundled_ids_are_unique() {
        let themes = bundled();
        let mut ids: Vec<_> = themes.iter().map(|t| t.id.clone()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), themes.len());
    }
}
