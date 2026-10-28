//! Theming for Nyx apps. Every stylesheet is written against the semantic
//! colour tokens (`@theme_bg`, `@theme_accent`, …); a theme is just a value
//! for each. An app hands [`install`] its stylesheet and a way to read its
//! theme preferences, and gets live restyling, including Follow Omarchy.

use gtk::prelude::*;
use gtk::{gdk, glib};
use nyx_core::paths;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::time::SystemTime;

mod themes;
use themes::themes_stamp;
pub use themes::{
    FileIconColors, Font, Look, Material, SWATCH_NAMES, Shape, Surface, Syntax, all, bundled, current_id, custom, edit_copy,
    look_css, theme_dirs,
};

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
    /// Stores a theme chosen in another Nyx app in this app's preferences.
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
    /// Everything beyond the core colours: accents, fonts, radii, material.
    pub look: Look,
}

/// Settings every Nyx app shares (`~/.config/nyx/theme.toml`): the theme,
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
    /// Where apps open: `home` (their start page) or `last` (the page shown last).
    pub start: String,
    /// The accent each app was given in its settings, by swatch name (`files = "violet"`).
    pub app_accents: std::collections::BTreeMap<String, String>,
}

impl Default for Suite {
    fn default() -> Self {
        Self {
            mode: ThemeMode::Omarchy,
            theme: "tokyo-night".into(),
            density: "comfortable".into(),
            text_scale: 1.0,
            start: "home".into(),
            app_accents: Default::default(),
        }
    }
}

/// The page an app opens on: `home` (its start page), or `last` (the one it
/// remembers) when the suite is set to reopen the last page.
pub fn start_page(home: &str, last: &str) -> String {
    let reopen = read_suite().is_some_and(|s| s.start == "last");
    if reopen && !last.is_empty() { last.to_string() } else { home.to_string() }
}

/// Whether apps reopen the last page instead of their start page.
pub fn reopens_last_page() -> bool {
    read_suite().is_some_and(|s| s.start == "last")
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

/// The Nyx 2.0 stylesheet, loaded on top of [`BASE_CSS`] by apps that have
/// moved to 2.0 ([`use_design_2`]).
pub const BASE2_CSS: &str = include_str!("base2.css");

/// Lines between areas, for themes that turn `blended` off.
const UNBLENDED: &str = "
.top-bar, .app-header { border-bottom: 1px solid @hairline; }
.settings-navigation { border-right: 1px solid @hairline; }
.status-bar { border-top: 1px solid @hairline; }
.group-list, .card, .settings-option { border: 1px solid @hairline; }
.group-list .settings-option { border: none; border-top: 1px solid @theme_divider; }
.group-list > .settings-option:first-child, .group-list > box:first-child > .settings-option:first-child { border-top: none; }
";

/// 2.0 sizes for the suite's density: `(header, nav item, row, option row, control, status bar)`.
fn density_sizes(density: &str) -> (u32, u32, u32, u32, u32, u32) {
    match density {
        "compact" => (42, 30, 30, 44, 30, 26),
        "relaxed" => (50, 38, 42, 60, 34, 30),
        _ => (46, 34, 36, 52, 32, 28),
    }
}

/// The suite's density and text size as 2.0 CSS for one app.
fn suite2_css(window_class: &str, suite: &Suite) -> String {
    let (header, nav, row, option, control, status) = density_sizes(&suite.density);
    let segment = control - 6;
    let mut css = format!(
        "window.{w} .top-bar, window.{w} .app-header {{ min-height: {header}px; }}\n\
         window.{w} .settings-navigation button.nav-item {{ min-height: {nav}px; }}\n\
         window.{w} listview > row, window.{w} list > row, window.{w} columnview > listview > row {{ min-height: {row}px; }}\n\
         window.{w} .settings-option {{ min-height: {option}px; }}\n\
         window.{w} button, window.{w} entry, window.{w} spinbutton, window.{w} dropdown > button {{ min-height: {control}px; }}\n\
         window.{w} button.bar-button, window.{w} button.icon-button, window.{w} menubutton.bar-button > button {{ min-width: {control}px; min-height: {control}px; padding: 0; }}\n\
         window.{w} .segmented button.segment {{ min-height: {segment}px; }}\n\
         window.{w} .status-bar {{ min-height: {status}px; }}\n",
        w = window_class
    );
    let scale = suite.text_scale.clamp(0.9, 1.25);
    if (scale - 1.0).abs() > 0.001 {
        css.push_str(&format!("window.{window_class} {{ font-size: {scale:.3}em; }}\n"));
    }
    css
}

/// The mixes the 2.0 sheet uses, from the theme's tint strengths.
fn defines2(look: &Look) -> String {
    format!(
        "@define-color hairline {hairline};\n\
         @define-color divider {divider};\n\
         @define-color nav_active mix(@theme_bg, @theme_accent, {sel});\n\
         @define-color row_hover mix(@theme_bg, @theme_accent, {hover});\n\
         @define-color lift @theme_card;\n\
         @define-color lift_active mix(@theme_card, @theme_accent, 0.1);\n\
         @define-color veil alpha(mix(@theme_bg, @theme_shadow, 0.45), 0.82);\n\
         @define-color field alpha(@theme_text, 0.035);\n\
         @define-color track alpha(@theme_text, 0.1);\n",
        sel = look.selection,
        hover = look.hover,
        // Blended: the lines an app's own sheet still draws between areas and rows vanish.
        hairline = if look.material.blended { "alpha(@theme_bg, 0)" } else { "mix(@theme_bg, @theme_border, 0.82)" },
        divider = if look.material.blended { "alpha(@theme_bg, 0)" } else { "@theme_divider" },
    )
}

/// The full sheet an app with no stylesheet of its own would load for `theme`,
/// for checking that GTK accepts it (see `examples/check_css.rs`).
pub fn sheet_for_check(window_class: &str, theme: &Theme, v2: bool) -> String {
    let suite = Suite::default();
    let mut css = palette_css(&theme.palette);
    css.push_str(&look_css(window_class, &theme.palette.accent, &theme.look));
    if v2 {
        css.push_str(&defines2(&theme.look));
        css.push_str(&without_defines(&base_css(window_class, &[])));
        css.push('\n');
        css.push_str(&BASE2_CSS.replace("@APP@", window_class));
        css.push_str(UNBLENDED);
        css.push_str(&suite2_css(window_class, &suite));
    } else {
        css.push_str(&base_css(window_class, &[]));
        css.push_str(&suite_css(window_class, &suite));
    }
    css
}

/// The shared v1 sheet without its colour definitions, which 2.0 replaces.
fn without_defines(css: &str) -> String {
    css.lines().filter(|l| !l.trim_start().starts_with("@define-color")).collect::<Vec<_>>().join("\n")
}

pub fn suite_file() -> std::path::PathBuf {
    paths::suite_dir().join("theme.toml")
}

/// The shared settings, if any app has written them yet.
pub fn read_suite() -> Option<Suite> {
    let mut suite: Suite = toml::from_str(&std::fs::read_to_string(suite_file()).ok()?).ok()?;
    suite.theme = themes::current_id(&suite.theme).to_owned();
    Some(suite)
}

/// Change the shared settings; every open Nyx app follows within a second.
pub fn update_suite(change: impl FnOnce(&mut Suite)) {
    let mut suite = read_suite().unwrap_or_default();
    change(&mut suite);
    if let Ok(text) = toml::to_string_pretty(&suite) {
        let _ = nyx_core::cmd::atomic_write(&suite_file(), text);
    }
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

/// The theme that should be showing right now according to the prefs. Follow
/// Omarchy takes Omarchy's colours and gives each app an accent from them.
pub fn current_theme() -> Theme {
    let prefs = config_prefs();
    theme_for(&prefs.mode, &prefs.theme)
}

/// The theme `mode` and `id` name: Omarchy's colours when following it (and it
/// has a theme), else the theme with that id, else Tokyo Night.
fn theme_for(mode: &ThemeMode, id: &str) -> Theme {
    if *mode == ThemeMode::Omarchy
        && let Ok(text) = std::fs::read_to_string(paths::omarchy_colors())
        && let Some(palette) = from_omarchy_colors(&text)
    {
        let table = toml::from_str::<toml::Table>(&text).unwrap_or_default();
        let look = themes::omarchy_look(&palette, &table);
        return Theme { id: "omarchy".into(), name: "Omarchy".into(), palette, look };
    }
    let mut themes = all();
    let id = themes::current_id(id);
    let pick = themes.iter().position(|t| t.id == id).or_else(|| themes.iter().position(|t| t.id == "tokyo-night"));
    themes.swap_remove(pick.expect("bundled themes are never empty"))
}

/// The suite's theme as app `app` shows it in 2.0: the shared choice from
/// `~/.config/nyx/theme.toml`, with the app's accent (its picked swatch, else
/// the theme's default for it) as the palette's accent. For apps that run their
/// own stylesheet machinery instead of [`install`]; poll it to follow changes.
pub fn suite_theme(app: &str) -> Theme {
    let suite = read_suite().unwrap_or_default();
    let mut theme = theme_for(&suite.mode, &suite.theme);
    let accent = resolve_app_accent(app, &theme.palette, &theme.look, &suite);
    theme.palette.glow = accent.clone();
    theme.palette.accent = accent;
    theme
}

/// The palette that should be showing right now according to the prefs.
pub fn current_palette() -> Palette {
    current_theme().palette
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
    static ACTIVE_LOOK: RefCell<Option<Look>> = const { RefCell::new(None) };
    /// Whether this app has moved to Nyx 2.0.
    static DESIGN_2: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// The section of the app on screen, for section accents (2.0).
    static SECTION: RefCell<Option<String>> = const { RefCell::new(None) };
    static LISTENERS: RefCell<Vec<(glib::WeakRef<gtk::Widget>, Listener)>> = const { RefCell::new(Vec::new()) };
    static GLOBAL: RefCell<Vec<Listener>> = const { RefCell::new(Vec::new()) };
    /// The theme this app and the suite file last agreed on.
    static SYNCED: RefCell<Option<(ThemeMode, String)>> = const { RefCell::new(None) };
}

fn config_prefs() -> ThemePrefs {
    let prefs = STYLE.with(|s| s.borrow().as_ref().map(|s| (s.prefs)())).unwrap_or(ThemePrefs {
        mode: ThemeMode::Omarchy,
        theme: "tokyo-night".into(),
        glow: true,
        reduce_motion: false,
    });
    effective_prefs(prefs, read_suite().as_ref(), shares_theme())
}

/// Whether this app stores the suite's theme in its own preferences, and so
/// may also change it for every app.
fn shares_theme() -> bool {
    STYLE.with(|s| s.borrow().as_ref().is_some_and(|s| s.set_theme.is_some()))
}

/// This app's theme preferences as they apply now (the suite's theme when the app
/// shares it), for the settings card's Appearance group.
pub fn app_prefs() -> ThemePrefs {
    config_prefs()
}

/// Whether this app can choose the theme (it stores a theme choice, see
/// [`Style::set_theme`]); otherwise it only follows the suite's.
pub fn can_choose_theme() -> bool {
    shares_theme()
}

/// Store a theme choice through the app's own preferences (and so for the suite),
/// then restyle.
pub fn choose_theme(mode: ThemeMode, theme: &str) {
    if let Some(set) = STYLE.with(|s| s.borrow().as_ref().and_then(|s| s.set_theme)) {
        set(mode, theme.to_owned());
    }
    apply();
}

/// The theme an app shows. An app that can't store a theme choice shows the
/// suite's, whatever its own preferences say.
fn effective_prefs(mut prefs: ThemePrefs, suite: Option<&Suite>, shares: bool) -> ThemePrefs {
    if !shares && let Some(suite) = suite {
        prefs.mode = suite.mode.clone();
        prefs.theme = suite.theme.clone();
    }
    prefs.theme = themes::current_id(&prefs.theme).to_owned();
    prefs
}

/// The palette currently on screen (cached; graphs read it on every frame).
pub fn palette() -> Palette {
    ACTIVE.with(|a| a.borrow_mut().get_or_insert_with(current_palette).clone())
}

/// The look currently on screen: accents, fonts, radii, material (cached).
pub fn look() -> Look {
    ACTIVE_LOOK.with(|a| a.borrow_mut().get_or_insert_with(|| current_theme().look).clone())
}

/// The accent for app `app` (`files`, `mail`, …): the swatch picked in its
/// settings, else the theme's colour for it, else the theme accent.
pub fn app_accent(app: &str) -> String {
    resolve_app_accent(app, &palette(), &look(), &read_suite().unwrap_or_default())
}

fn resolve_app_accent(app: &str, palette: &Palette, look: &Look, suite: &Suite) -> String {
    suite
        .app_accents
        .get(app)
        .and_then(|name| look.swatches.iter().find(|(n, _)| n == name))
        .map(|(_, c)| c.clone())
        .or_else(|| look.accents.get(app).cloned())
        .unwrap_or_else(|| palette.accent.clone())
}

/// The accents to offer in an app's settings: `(name, colour)` from the active theme.
pub fn swatches() -> Vec<(String, String)> {
    look().swatches
}

/// The swatch picked for `app`, if any (`None` is *Theme default*).
pub fn app_swatch(app: &str) -> Option<String> {
    read_suite().and_then(|s| s.app_accents.get(app).cloned())
}

/// Pick a swatch for `app`, or `None` for the theme default. Every open app
/// shows the change within a second.
pub fn set_app_swatch(app: &str, name: Option<&str>) {
    update_suite(|s| match name {
        Some(n) => {
            s.app_accents.insert(app.to_string(), n.to_string());
        }
        None => {
            s.app_accents.remove(app);
        }
    });
}

/// The accent for a section of an app (Settings → Privacy), else the app's.
pub fn section_accent(app: &str, section: &str) -> String {
    look().sections.get(app).and_then(|s| s.get(section)).cloned().unwrap_or_else(|| app_accent(app))
}

/// Call `f` whenever the palette changes, for as long as `owner` lives.
pub fn subscribe(owner: &impl IsA<gtk::Widget>, f: impl Fn() + 'static) {
    LISTENERS.with(|l| l.borrow_mut().push((owner.upcast_ref::<gtk::Widget>().downgrade(), Box::new(f))));
}

/// Call `f` after every restyle for the rest of the run.
pub fn on_change(f: impl Fn() + 'static) {
    GLOBAL.with(|g| g.borrow_mut().push(Box::new(f)));
}

/// Move this app to Nyx 2.0: the 2.0 stylesheet on top of the shared one, the
/// app's own accent as `@theme_accent`, and 2.0 sizes for the suite's density.
/// Call it before [`install`].
pub fn use_design_2() {
    DESIGN_2.with(|d| d.set(true));
}

/// Tell the theme which section of the app is on screen (`Some("privacy")`), so
/// its section accent applies; `None` for none. Restyles when it changes. A
/// swatch picked in the app's Accent row wins over section accents.
pub fn set_section(section: Option<&str>) {
    let new = section.map(str::to_string);
    if SECTION.with(|s| *s.borrow() != new) {
        SECTION.with(|s| *s.borrow_mut() = new);
        apply();
    }
}

/// Whether this app uses Nyx 2.0.
pub fn design_2() -> bool {
    DESIGN_2.with(|d| d.get())
}

/// Install the app's stylesheet and the current palette on the default display.
pub fn install(style: Style) {
    // Start with the suite's theme if another app has chosen one.
    let mine = {
        let p = (style.prefs)();
        (p.mode, themes::current_id(&p.theme).to_owned())
    };
    let synced = match read_suite() {
        Some(suite) if (suite.mode.clone(), suite.theme.clone()) != mine => {
            if let Some(set) = style.set_theme {
                set(suite.mode.clone(), suite.theme.clone());
            }
            (suite.mode, suite.theme)
        }
        Some(_) => mine,
        // Only an app that stores a theme choice may start the shared file.
        None if style.set_theme.is_none() => mine,
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
    // A theme chosen in this app goes to every app. An app that can't store a
    // choice only follows the suite's, never writes its own back.
    let mine = (prefs.mode.clone(), prefs.theme.clone());
    if shares_theme() && SYNCED.with(|s| s.borrow().as_ref() != Some(&mine)) {
        update_suite(|s| {
            s.mode = mine.0.clone();
            s.theme = mine.1.clone();
        });
        SYNCED.with(|s| *s.borrow_mut() = Some(mine));
    }
    let Theme { mut palette, mut look, .. } = current_theme();
    let v2 = design_2();
    let suite = read_suite().unwrap_or_default();
    let app_accent = STYLE.with(|s| {
        s.borrow().as_ref().map(|style| {
            let app = style.window_class.strip_suffix("-window").unwrap_or(style.window_class);
            let section = SECTION.with(|s| s.borrow().clone());
            let picked = suite.app_accents.contains_key(app);
            section
                .filter(|_| !picked)
                .and_then(|sec| look.sections.get(app).and_then(|m| m.get(&sec)).cloned())
                .unwrap_or_else(|| resolve_app_accent(app, &palette, &look, &suite))
        })
    });
    if v2 {
        // In 2.0 the app's own accent is the accent everything uses.
        if let Some(accent) = &app_accent {
            palette.accent = accent.clone();
            palette.glow = accent.clone();
        }
        if look.material.blended {
            look.panel = palette.bg.clone();
        }
    }
    ACTIVE.with(|a| *a.borrow_mut() = Some(palette.clone()));
    ACTIVE_LOOK.with(|a| *a.borrow_mut() = Some(look.clone()));
    let mut css = palette_css(&palette);
    STYLE.with(|s| {
        if let Some(style) = s.borrow().as_ref() {
            let accent = app_accent.clone().unwrap_or_else(|| palette.accent.clone());
            if v2 {
                css.push_str(&look_css(style.window_class, &accent, &look));
                css.push_str(&defines2(&look));
            }
            if let Some(extra) = style.extra_colors {
                css.push_str(&extra(&palette));
            }
            let base = base_css(style.window_class, style.base_skip);
            if v2 {
                css.push_str(&without_defines(&base));
                css.push('\n');
                css.push_str(&BASE2_CSS.replace("@APP@", style.window_class));
                if !look.material.blended {
                    css.push_str(UNBLENDED);
                }
            } else {
                css.push_str(&base);
            }
            css.push_str(style.css);
            if v2 {
                // Again after the app's sheet, so the theme's fonts win over an app's own.
                css.push_str(&look_css(style.window_class, &accent, &look));
                css.push_str(&suite2_css(style.window_class, &suite));
            } else {
                css.push_str(&look_css(style.window_class, &accent, &look));
                css.push_str(&suite_css(style.window_class, &suite));
            }
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
        if let Some(property) = settings.find_property("gtk-interface-color-scheme") {
            if let Some(value) = glib::EnumClass::with_type(property.value_type())
                .and_then(|class| class.to_value_by_nick(if palette.light { "light" } else { "dark" }))
            {
                settings.set_property_from_value("gtk-interface-color-scheme", &value);
            }
        } else {
            settings.set_gtk_application_prefer_dark_theme(!palette.light);
        }

        settings.set_gtk_enable_animations(!prefs.reduce_motion);
    }
    notify_listeners();
}

/// Run every listener with the lists taken out, so a listener may subscribe (a page
/// rebuilt on a theme change does) without borrowing them twice.
fn notify_listeners() {
    let mut taken = LISTENERS.with(|l| std::mem::take(&mut *l.borrow_mut()));
    taken.retain(|(w, _)| w.upgrade().is_some());
    for (_, f) in &taken {
        f();
    }
    LISTENERS.with(|l| {
        let mut l = l.borrow_mut();
        let added = std::mem::take(&mut *l);
        *l = taken;
        l.extend(added);
        l.retain(|(w, _)| w.upgrade().is_some());
    });
    let global = GLOBAL.with(|g| std::mem::take(&mut *g.borrow_mut()));
    for f in &global {
        f();
    }
    GLOBAL.with(|g| {
        let mut g = g.borrow_mut();
        let added = std::mem::take(&mut *g);
        *g = global;
        g.extend(added);
    });
}

fn stamp() -> Option<(SystemTime, String)> {
    let colors = paths::omarchy_colors();
    let modified = std::fs::metadata(&colors).and_then(|m| m.modified()).ok()?;
    let name = std::fs::read_to_string(paths::state_home().join("omarchy/current/theme.name")).unwrap_or_default();
    Some((modified, name))
}

/// Omarchy swaps the theme by rewriting its state directory; a light poll is the
/// most robust way to notice every variant of that. The same poll picks up a
/// saved theme file and the shared settings.
fn watch_omarchy() {
    let last = RefCell::new(stamp());
    let suite_stamp = || std::fs::metadata(suite_file()).and_then(|m| m.modified()).ok();
    let last_suite = RefCell::new(suite_stamp());
    let last_themes = RefCell::new(themes_stamp());
    glib::timeout_add_seconds_local(1, move || {
        let now = stamp();
        if now != *last.borrow() {
            *last.borrow_mut() = now;
            if config_prefs().mode == ThemeMode::Omarchy {
                apply();
            }
        }
        // Another Nyx app changed the shared theme.
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
        // A theme file was saved: show the change if a theme (not Omarchy) is in use.
        let now = themes_stamp();
        if now != *last_themes.borrow() {
            *last_themes.borrow_mut() = now;
            if config_prefs().mode == ThemeMode::Theme {
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
    fn prefs(mode: ThemeMode, theme: &str) -> ThemePrefs {
        ThemePrefs { mode, theme: theme.into(), glow: true, reduce_motion: false }
    }

    #[test]
    fn an_app_without_a_theme_choice_shows_the_suites() {
        let suite = Suite { mode: ThemeMode::Theme, theme: "nord".into(), ..Suite::default() };
        let shown = effective_prefs(prefs(ThemeMode::Omarchy, ""), Some(&suite), false);
        assert_eq!((shown.mode, shown.theme.as_str()), (ThemeMode::Theme, "nord"));
        // An app that stores its choice shows its own until it syncs.
        let own = effective_prefs(prefs(ThemeMode::Omarchy, ""), Some(&suite), true);
        assert_eq!(own.mode, ThemeMode::Omarchy);
        // Without a shared file, the app's own preferences apply.
        let alone = effective_prefs(prefs(ThemeMode::Theme, "nixe-dark"), None, false);
        assert_eq!(alone.theme, "nyx-dark");
    }
    use super::*;

    #[test]
    fn listener_may_subscribe_while_notified() {
        thread_local!(static RUNS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) });
        on_change(|| {
            RUNS.with(|r| r.set(r.get() + 1));
            on_change(|| RUNS.with(|r| r.set(r.get() + 10)));
        });
        notify_listeners();
        assert_eq!(RUNS.with(|r| r.get()), 1);
        notify_listeners();
        // The first listener again (and it adds another), plus the one it added.
        assert_eq!(RUNS.with(|r| r.get()), 12);
    }

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
    fn design_2_replaces_the_v1_colours() {
        let base = without_defines(&base_css("files-window", &[]));
        assert!(!base.contains("@define-color"));
        let look = bundled().into_iter().find(|t| t.id == "nyx-dark").unwrap().look;
        let defines = defines2(&look);
        for name in ["hairline", "divider", "nav_active", "row_hover", "lift", "lift_active", "veil", "field", "track"] {
            assert!(defines.contains(&format!("@define-color {name} ")), "{name}");
        }
        let used = ["@field", "@track", "@theme_panel", "@theme_card", "@theme_raised", "@theme_popover", "@theme_text_tertiary"];
        for name in used {
            assert!(BASE2_CSS.contains(name), "2.0 sheet uses {name}");
        }
    }

    #[test]
    fn density_sizes_follow_the_suite() {
        let mut suite = Suite::default();
        assert!(suite2_css("files-window", &suite).contains(".app-header { min-height: 46px; }"));
        suite.density = "compact".into();
        assert!(suite2_css("files-window", &suite).contains(".app-header { min-height: 42px; }"));
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
