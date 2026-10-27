//! Theme files: the shipped themes, the user's own and overrides of shipped
//! ones, all in one TOML format where every key is optional. A theme sets the
//! look: colours, accents (per app and per section), fonts, corner radii,
//! material and a recommended icon pack. Sizes and layout stay with the apps;
//! density, text size and motion are preferences.

use super::{Palette, Theme};
use gtk::gdk;
use nixe_core::paths;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::SystemTime;

/// Everything a theme sets beyond the core colours in [`Palette`].
#[derive(Debug, Clone, PartialEq)]
pub struct Look {
    /// Sidebar and header.
    pub panel: String,
    /// Cards and grouped rows.
    pub card: String,
    /// Raised controls: the selected segment, buttons on cards.
    pub raised: String,
    /// Popovers, menus and dialogs.
    pub popover: String,
    /// Hints and placeholders, below `dim_text`.
    pub text_tertiary: String,
    /// Lines between rows inside a card.
    pub divider: String,
    pub warning: String,
    pub success: String,
    /// How much accent tints a hovered row (0–1).
    pub hover: f64,
    /// How much accent tints a selected row or nav item (0–1).
    pub selection: f64,
    /// Accent per app, by app name (`files`, `mail`, …).
    pub accents: BTreeMap<String, String>,
    /// Accent per section of an app (`settings` → `privacy` → colour).
    pub sections: BTreeMap<String, BTreeMap<String, String>>,
    /// The accents offered in each app's settings, by name, in [`SWATCH_NAMES`] order first.
    pub swatches: Vec<(String, String)>,
    pub font: Font,
    pub shape: Shape,
    pub material: Material,
    /// Icon pack the theme recommends; the icon theme stays the user's choice.
    pub icons: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Font {
    /// Interface font family; `None` leaves the app's own.
    pub ui: Option<String>,
    /// Code and readout font family.
    pub mono: Option<String>,
    pub weight: Option<u16>,
    pub heading_weight: Option<u16>,
}

/// Corner radii in px.
#[derive(Debug, Clone, PartialEq)]
pub struct Shape {
    pub radius_small: u8,
    pub radius_card: u8,
    pub radius_dialog: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Surface {
    Solid,
    /// Translucent over the compositor's blur (Hyprland), solid without it.
    Frosted,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    pub surface: Surface,
    /// Frosted panels show the blurred desktop; off keeps the frost opaque.
    pub see_through: bool,
    /// One surface: no lines or different fills between header, sidebar and content.
    pub blended: bool,
}

/// The last resort when a theme leaves a colour out or gets one wrong:
/// Tokyo Night, compiled in so an app always has every colour.
pub(crate) fn fallback_palette() -> Palette {
    Palette {
        bg: "#1a1b26".into(),
        surface: "#16161e".into(),
        text: "#c0caf5".into(),
        dim_text: "#565f89".into(),
        accent: "#7aa2f7".into(),
        border: "#292e42".into(),
        muted: "#3b4261".into(),
        highlight: "#283457".into(),
        danger: "#f7768e".into(),
        glow: "#7aa2f7".into(),
        shadow: "#0f0f14".into(),
        light: false,
    }
}

/// Accent names every theme offers, so an app's pick survives a theme switch.
pub const SWATCH_NAMES: &[&str] = &["blue", "violet", "pink", "red", "coral", "amber", "green", "teal", "graphite"];

/// Swatches for themes that don't define their own, for dark and light backgrounds.
const DARK_SWATCHES: [&str; 9] =
    ["#5b9cff", "#9d7bff", "#ff6fae", "#ff6b6b", "#ff7a59", "#f2b545", "#46d08a", "#4cc9d6", "#a3a3ab"];
const LIGHT_SWATCHES: [&str; 9] =
    ["#2f6fea", "#6f4de0", "#c93a7b", "#d93a3a", "#d4532f", "#b7791f", "#1f9e5c", "#1a8fa0", "#5a5a64"];

/// The themes Nixe ships. They are the same TOML a user writes, kept as files in
/// `themes/` and built in, so they never go missing.
const SHIPPED: &[(&str, &str)] = &[
    ("nixe-dark", include_str!("../themes/nixe-dark.toml")),
    ("nixe-light", include_str!("../themes/nixe-light.toml")),
    ("dracula", include_str!("../themes/dracula.toml")),
    ("catppuccin-mocha", include_str!("../themes/catppuccin-mocha.toml")),
    ("catppuccin-macchiato", include_str!("../themes/catppuccin-macchiato.toml")),
    ("catppuccin-frappe", include_str!("../themes/catppuccin-frappe.toml")),
    ("catppuccin-latte", include_str!("../themes/catppuccin-latte.toml")),
    ("tokyo-night", include_str!("../themes/tokyo-night.toml")),
    ("tokyo-night-storm", include_str!("../themes/tokyo-night-storm.toml")),
    ("tokyo-night-moon", include_str!("../themes/tokyo-night-moon.toml")),
    ("one-dark-pro", include_str!("../themes/one-dark-pro.toml")),
    ("nord", include_str!("../themes/nord.toml")),
    ("gruvbox-dark", include_str!("../themes/gruvbox-dark.toml")),
    ("rose-pine", include_str!("../themes/rose-pine.toml")),
    ("everforest", include_str!("../themes/everforest.toml")),
    ("solarized-dark", include_str!("../themes/solarized-dark.toml")),
    ("kanagawa", include_str!("../themes/kanagawa.toml")),
];

/// A theme file, shipped or the user's. Every key is optional: what a file
/// leaves out comes from the theme it overrides, the one it `inherits`, or a
/// value derived from its colours.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ThemeFile {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    inherits: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    light: Option<bool>,
    bg: Option<String>,
    surface: Option<String>,
    text: Option<String>,
    dim_text: Option<String>,
    accent: Option<String>,
    border: Option<String>,
    muted: Option<String>,
    highlight: Option<String>,
    danger: Option<String>,
    glow: Option<String>,
    shadow: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    panel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    card: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    raised: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    popover: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text_tertiary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    divider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    warning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    success: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hover: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    selection: Option<f64>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    accents: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    sections: BTreeMap<String, BTreeMap<String, String>>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    swatches: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "FontFile::is_empty")]
    font: FontFile,
    #[serde(default, skip_serializing_if = "ShapeFile::is_empty")]
    shape: ShapeFile,
    #[serde(default, skip_serializing_if = "MaterialFile::is_empty")]
    material: MaterialFile,
    #[serde(default, skip_serializing_if = "IconsFile::is_empty")]
    icons: IconsFile,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FontFile {
    #[serde(skip_serializing_if = "Option::is_none")]
    ui: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mono: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    weight: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    heading_weight: Option<u16>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ShapeFile {
    #[serde(skip_serializing_if = "Option::is_none")]
    radius_small: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    radius_card: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    radius_dialog: Option<u8>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct MaterialFile {
    #[serde(skip_serializing_if = "Option::is_none")]
    surface: Option<Surface>,
    #[serde(skip_serializing_if = "Option::is_none")]
    see_through: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    blended: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct IconsFile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pack: Option<String>,
}

/// `top`'s values over `base`'s, field by field (and key by key in tables).
macro_rules! layer {
    ($base:expr, $top:expr; $($field:ident),*) => {
        $( if $top.$field.is_some() { $base.$field = $top.$field.clone(); } )*
    };
}

impl FontFile {
    fn is_empty(&self) -> bool {
        self.ui.is_none() && self.mono.is_none() && self.weight.is_none() && self.heading_weight.is_none()
    }
}

impl ShapeFile {
    fn is_empty(&self) -> bool {
        self.radius_small.is_none() && self.radius_card.is_none() && self.radius_dialog.is_none()
    }
}

impl MaterialFile {
    fn is_empty(&self) -> bool {
        self.surface.is_none() && self.see_through.is_none() && self.blended.is_none()
    }
}

impl IconsFile {
    fn is_empty(&self) -> bool {
        self.pack.is_none()
    }
}

impl ThemeFile {
    pub(crate) fn parse(text: &str, origin: &str) -> Option<ThemeFile> {
        toml::from_str(text).map_err(|e| eprintln!("nixe-theme: {origin}: {e}")).ok()
    }

    /// This file over `base`.
    fn over(&self, base: &ThemeFile) -> ThemeFile {
        let mut out = base.clone();
        layer!(out, self; name, light, bg, surface, text, dim_text, accent, border, muted, highlight, danger, glow, shadow,
            panel, card, raised, popover, text_tertiary, divider, warning, success, hover, selection);
        out.inherits = None;
        out.accents.extend(self.accents.clone());
        out.swatches.extend(self.swatches.clone());
        for (app, sections) in &self.sections {
            out.sections.entry(app.clone()).or_default().extend(sections.clone());
        }
        layer!(out.font, self.font; ui, mono, weight, heading_weight);
        layer!(out.shape, self.shape; radius_small, radius_card, radius_dialog);
        layer!(out.material, self.material; surface, see_through, blended);
        layer!(out.icons, self.icons; pack);
        out
    }

    /// The core colours, each checked; a missing or unreadable one comes from
    /// the fallback with a warning for the latter.
    pub(crate) fn palette(&self, origin: &str) -> Palette {
        let base = fallback_palette();
        let pick =
            |key: &str, value: &Option<String>, fallback: &String| color(origin, key, value).unwrap_or_else(|| fallback.clone());
        Palette {
            bg: pick("bg", &self.bg, &base.bg),
            surface: pick("surface", &self.surface, &base.surface),
            text: pick("text", &self.text, &base.text),
            dim_text: pick("dim_text", &self.dim_text, &base.dim_text),
            accent: pick("accent", &self.accent, &base.accent),
            border: pick("border", &self.border, &base.border),
            muted: pick("muted", &self.muted, &base.muted),
            highlight: pick("highlight", &self.highlight, &base.highlight),
            danger: pick("danger", &self.danger, &base.danger),
            glow: pick("glow", &self.glow, &base.glow),
            shadow: pick("shadow", &self.shadow, &base.shadow),
            light: self.light.unwrap_or(base.light),
        }
    }

    /// The rest of the look; what the file leaves out is derived from `p`.
    pub(crate) fn look(&self, p: &Palette, origin: &str) -> Look {
        let pick = |key: &str, value: &Option<String>, derived: String| color(origin, key, value).unwrap_or(derived);
        let swatches = swatches(&self.swatches, p.light);
        // Accents may name a swatch (`files = "blue"`), so they follow the theme's
        // own version of that colour, Omarchy's included.
        let colors = |map: &BTreeMap<String, String>, scope: &str| -> BTreeMap<String, String> {
            map.iter()
                .filter_map(|(k, v)| {
                    let named = swatches.iter().find(|(n, _)| n == v).map(|(_, c)| c.clone());
                    named.or_else(|| color(origin, &format!("{scope}.{k}"), &Some(v.clone()))).map(|c| (k.clone(), c))
                })
                .collect()
        };
        Look {
            panel: pick("panel", &self.panel, mix(&p.bg, &p.text, 0.025)),
            card: pick("card", &self.card, mix(&p.bg, &p.text, 0.055)),
            raised: pick("raised", &self.raised, mix(&p.bg, &p.text, 0.09)),
            popover: pick("popover", &self.popover, p.surface.clone()),
            text_tertiary: pick("text_tertiary", &self.text_tertiary, mix(&p.dim_text, &p.bg, 0.2)),
            divider: pick("divider", &self.divider, mix(&p.bg, &p.text, 0.08)),
            warning: pick("warning", &self.warning, if p.light { "#b7791f" } else { "#e5a50a" }.into()),
            success: pick("success", &self.success, if p.light { "#1f9e5c" } else { "#46d08a" }.into()),
            hover: self.hover.unwrap_or(0.07).clamp(0.0, 1.0),
            selection: self.selection.unwrap_or(0.18).clamp(0.0, 1.0),
            accents: colors(&self.accents, "accents"),
            sections: self.sections.iter().map(|(app, m)| (app.clone(), colors(m, &format!("sections.{app}")))).collect(),
            swatches,
            font: Font {
                ui: self.font.ui.clone(),
                mono: self.font.mono.clone(),
                weight: self.font.weight,
                heading_weight: self.font.heading_weight,
            },
            shape: Shape {
                radius_small: self.shape.radius_small.unwrap_or(6),
                radius_card: self.shape.radius_card.unwrap_or(10),
                radius_dialog: self.shape.radius_dialog.unwrap_or(14),
            },
            material: Material {
                surface: self.material.surface.unwrap_or(Surface::Solid),
                see_through: self.material.see_through.unwrap_or(false),
                blended: self.material.blended.unwrap_or(false),
            },
            icons: self.icons.pack.clone(),
        }
    }

    /// A complete file for `theme`, to edit.
    fn of(theme: &Theme) -> ThemeFile {
        let (p, l) = (&theme.palette, &theme.look);
        ThemeFile {
            name: Some(theme.name.clone()),
            inherits: None,
            light: Some(p.light),
            bg: Some(p.bg.clone()),
            surface: Some(p.surface.clone()),
            text: Some(p.text.clone()),
            dim_text: Some(p.dim_text.clone()),
            accent: Some(p.accent.clone()),
            border: Some(p.border.clone()),
            muted: Some(p.muted.clone()),
            highlight: Some(p.highlight.clone()),
            danger: Some(p.danger.clone()),
            glow: Some(p.glow.clone()),
            shadow: Some(p.shadow.clone()),
            panel: Some(l.panel.clone()),
            card: Some(l.card.clone()),
            raised: Some(l.raised.clone()),
            popover: Some(l.popover.clone()),
            text_tertiary: Some(l.text_tertiary.clone()),
            divider: Some(l.divider.clone()),
            warning: Some(l.warning.clone()),
            success: Some(l.success.clone()),
            hover: Some(l.hover),
            selection: Some(l.selection),
            accents: l.accents.clone(),
            sections: l.sections.clone(),
            swatches: l.swatches.iter().cloned().collect(),
            font: FontFile {
                ui: l.font.ui.clone(),
                mono: l.font.mono.clone(),
                weight: l.font.weight,
                heading_weight: l.font.heading_weight,
            },
            shape: ShapeFile {
                radius_small: Some(l.shape.radius_small),
                radius_card: Some(l.shape.radius_card),
                radius_dialog: Some(l.shape.radius_dialog),
            },
            material: MaterialFile {
                surface: Some(l.material.surface),
                see_through: Some(l.material.see_through),
                blended: Some(l.material.blended),
            },
            icons: IconsFile { pack: l.icons.clone() },
        }
    }

    pub(crate) fn theme(&self, id: &str, origin: &str) -> Theme {
        let palette = self.palette(origin);
        let look = self.look(&palette, origin);
        Theme { id: id.into(), name: self.name.clone().unwrap_or_else(|| id.into()), palette, look }
    }
}

/// `value` if GTK can read it as a colour; a value it can't is reported.
fn color(origin: &str, key: &str, value: &Option<String>) -> Option<String> {
    let v = value.as_ref()?;
    if gdk::RGBA::parse(v.as_str()).is_ok() {
        Some(v.clone())
    } else {
        eprintln!("nixe-theme: {origin}: {key} = {v:?} is not a colour, using the default");
        None
    }
}

/// `a` moved towards `b` by `t`, as `#rrggbb`.
pub(crate) fn mix(a: &str, b: &str, t: f64) -> String {
    let (Ok(a), Ok(b)) = (gdk::RGBA::parse(a), gdk::RGBA::parse(b)) else { return a.to_string() };
    let ch = |x: f32, y: f32| ((f64::from(x) + (f64::from(y) - f64::from(x)) * t) * 255.0).round().clamp(0.0, 255.0) as u8;
    format!("#{:02x}{:02x}{:02x}", ch(a.red(), b.red()), ch(a.green(), b.green()), ch(a.blue(), b.blue()))
}

/// A theme's swatches in [`SWATCH_NAMES`] order, the defaults filling any name it
/// leaves out, then any extra names it adds.
fn swatches(own: &BTreeMap<String, String>, light: bool) -> Vec<(String, String)> {
    let own: BTreeMap<String, String> =
        own.iter().filter(|(k, v)| color("swatches", k, &Some((*v).clone())).is_some()).map(|(k, v)| (k.clone(), v.clone())).collect();
    let defaults = if light { LIGHT_SWATCHES } else { DARK_SWATCHES };
    let mut out: Vec<(String, String)> = SWATCH_NAMES
        .iter()
        .zip(defaults)
        .map(|(name, c)| ((*name).to_string(), own.get(*name).cloned().unwrap_or_else(|| c.to_string())))
        .collect();
    out.extend(own.iter().filter(|(k, _)| !SWATCH_NAMES.contains(&k.as_str())).map(|(k, v)| (k.clone(), v.clone())));
    out
}

/// Swatches from an Omarchy theme's named colours, where it has them.
fn omarchy_swatches(table: &toml::Table) -> BTreeMap<String, String> {
    const MAP: &[(&str, &str)] = &[
        ("blue", "blue"),
        ("violet", "magenta"),
        ("pink", "bright_magenta"),
        ("red", "red"),
        ("coral", "orange"),
        ("amber", "yellow"),
        ("green", "green"),
        ("teal", "cyan"),
        ("graphite", "dark_foreground"),
    ];
    MAP.iter()
        .filter_map(|(name, key)| {
            let c = table.get(*key)?.as_str()?;
            gdk::RGBA::parse(c).ok().map(|_| ((*name).to_string(), c.to_string()))
        })
        .collect()
}

fn shipped_files() -> Vec<(&'static str, ThemeFile)> {
    SHIPPED.iter().map(|(id, text)| (*id, ThemeFile::parse(text, &format!("{id}.toml (shipped)")).unwrap_or_default())).collect()
}

/// The themes Nixe ships, before any user file changes them.
pub fn bundled() -> Vec<Theme> {
    shipped_files().iter().map(|(id, f)| f.theme(id, &format!("{id}.toml (shipped)"))).collect()
}

/// Where user themes live: `~/.config/nixe/themes/` for every app, then
/// `~/.config/<app>/themes/` for this one (a file there wins over one of the
/// same name in the shared folder).
pub fn theme_dirs() -> [PathBuf; 2] {
    [paths::suite_dir().join("themes"), paths::custom_themes_dir()]
}

/// The user's theme files by name (file stem), the app's own winning.
fn user_files() -> Vec<(String, PathBuf, ThemeFile)> {
    let mut by_stem: BTreeMap<String, (PathBuf, ThemeFile)> = BTreeMap::new();
    for dir in theme_dirs() {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for path in entries.flatten().map(|e| e.path()) {
            if path.extension().is_none_or(|x| x != "toml") {
                continue;
            }
            let Some(stem) = path.file_stem().map(|s| s.to_string_lossy().into_owned()) else { continue };
            let Ok(text) = std::fs::read_to_string(&path) else { continue };
            if let Some(file) = ThemeFile::parse(&text, &path.display().to_string()) {
                by_stem.insert(stem, (path, file));
            }
        }
    }
    by_stem.into_iter().map(|(stem, (path, file))| (stem, path, file)).collect()
}

/// Every theme on offer: the shipped ones with the user's changes applied
/// (a user file named like a shipped theme overrides it), then the user's own
/// themes as `custom:<file stem>`, sorted by name.
pub fn all() -> Vec<Theme> {
    let shipped = shipped_files();
    let mut files: Vec<(String, ThemeFile, String)> =
        shipped.iter().map(|(id, f)| ((*id).to_string(), f.clone(), format!("{id}.toml (shipped)"))).collect();
    let mut custom = Vec::new();
    for (stem, path, file) in user_files() {
        let origin = path.display().to_string();
        let inherited = file.inherits.as_ref().and_then(|id| {
            let found = shipped.iter().find(|(s, _)| s == id).map(|(_, f)| f.clone());
            if found.is_none() {
                eprintln!("nixe-theme: {origin}: inherits unknown theme {id:?}");
            }
            found
        });
        if let Some(slot) = files.iter_mut().find(|(id, _, _)| *id == stem) {
            let base = inherited.unwrap_or_else(|| slot.1.clone());
            *slot = (stem, file.over(&base), origin);
        } else {
            let merged = file.over(&inherited.unwrap_or_default());
            let theme = merged.theme(&format!("custom:{stem}"), &origin);
            let named = file.name.is_some() || file.inherits.is_some();
            custom.push(if named { theme } else { Theme { name: stem.clone(), ..theme } });
        }
    }
    custom.sort_by(|a, b| a.name.cmp(&b.name));
    let mut themes: Vec<Theme> = files.iter().map(|(id, f, origin)| f.theme(id, origin)).collect();
    themes.extend(custom);
    themes
}

/// The user's own themes (not overrides of shipped ones), sorted by name.
pub fn custom() -> Vec<Theme> {
    all().into_iter().filter(|t| t.id.starts_with("custom:")).collect()
}

/// The file to edit for theme `id`, written out in full first if the user
/// has none yet, so a shipped theme can be tuned without touching the app.
/// Saving it restyles every open Nixe app within a second.
pub fn edit_copy(id: &str) -> Option<PathBuf> {
    let stem = id.strip_prefix("custom:").unwrap_or(id);
    if let Some((_, path, _)) = user_files().into_iter().find(|(s, _, _)| s == stem) {
        return Some(path);
    }
    let theme = all().into_iter().find(|t| t.id == id)?;
    let path = paths::suite_dir().join("themes").join(format!("{stem}.toml"));
    let text = toml::to_string_pretty(&ThemeFile::of(&theme)).ok()?;
    nixe_core::cmd::atomic_write(&path, text).ok()?;
    Some(path)
}

/// When each user theme file last changed, to notice edits.
pub(crate) fn themes_stamp() -> Vec<(PathBuf, Option<SystemTime>)> {
    let mut files: Vec<_> = theme_dirs()
        .iter()
        .filter_map(|dir| std::fs::read_dir(dir).ok())
        .flat_map(|entries| entries.flatten())
        .map(|e| {
            let modified = e.metadata().and_then(|m| m.modified()).ok();
            (e.path(), modified)
        })
        .filter(|(p, _)| p.extension().is_some_and(|x| x == "toml"))
        .collect();
    files.sort();
    files
}

/// The look under Follow Omarchy: Omarchy's colours, with everything else
/// (fonts, radii, material, tint strengths) from Nixe Dark or Light, and app
/// accents and swatches from Omarchy's named colours.
pub(crate) fn omarchy_look(palette: &Palette, table: &toml::Table) -> Look {
    let id = if palette.light { "nixe-light" } else { "nixe-dark" };
    let nixe = shipped_files().into_iter().find(|(s, _)| *s == id).map(|(_, f)| f).unwrap_or_default();
    let mut swatches = nixe.swatches.clone();
    swatches.extend(omarchy_swatches(table));
    let file = ThemeFile {
        hover: nixe.hover,
        selection: nixe.selection,
        accents: nixe.accents,
        sections: nixe.sections,
        swatches,
        font: nixe.font,
        shape: nixe.shape,
        material: nixe.material,
        icons: nixe.icons,
        ..ThemeFile::default()
    };
    let mut look = file.look(palette, "omarchy");
    for (app, c) in omarchy_accents(table) {
        look.accents.entry(app).or_insert(c);
    }
    look
}

/// App accents taken from an Omarchy theme's named colours, so Follow Omarchy
/// still gives each app its own colour.
fn omarchy_accents(table: &toml::Table) -> BTreeMap<String, String> {
    const MAP: &[(&str, &str)] = &[
        ("files", "blue"),
        ("mail", "magenta"),
        ("notes", "yellow"),
        ("photos", "orange"),
        ("calendar", "red"),
        ("music", "magenta"),
        ("home", "green"),
        ("weather", "cyan"),
        ("chat", "cyan"),
        ("tasks", "green"),
    ];
    MAP.iter()
        .filter_map(|(app, key)| {
            let c = table.get(*key)?.as_str()?;
            gdk::RGBA::parse(c).ok().map(|_| ((*app).to_string(), c.to_string()))
        })
        .collect()
}

/// The look as CSS for one app: extra colours as `@theme_*`, the app's accent
/// as `@theme_app_accent`, radii and tint strengths as `--nixe-*` properties on
/// its window, and its fonts when the theme sets them.
pub fn look_css(window_class: &str, app_accent: &str, look: &Look) -> String {
    let mut css = format!(
        "@define-color theme_panel {};\n@define-color theme_card {};\n@define-color theme_raised {};\n\
         @define-color theme_popover {};\n@define-color theme_text_tertiary {};\n@define-color theme_divider {};\n\
         @define-color theme_warning {};\n@define-color theme_success {};\n@define-color theme_app_accent {};\n",
        look.panel,
        look.card,
        look.raised,
        look.popover,
        look.text_tertiary,
        look.divider,
        look.warning,
        look.success,
        app_accent,
    );
    css.push_str(&format!(
        "window.{window_class} {{ --nixe-radius-small: {}px; --nixe-radius-card: {}px; --nixe-radius-dialog: {}px; \
         --nixe-hover: {}; --nixe-selection: {};",
        look.shape.radius_small, look.shape.radius_card, look.shape.radius_dialog, look.hover, look.selection,
    ));
    if let Some(ui) = &look.font.ui {
        css.push_str(&format!(" font-family: \"{}\", sans-serif;", ui.replace('"', "")));
    }
    if let Some(weight) = look.font.weight {
        css.push_str(&format!(" font-weight: {weight};"));
    }
    css.push_str(" }\n");
    // Each section's colour on what carries its class (`.section-network`): the
    // sidebar's group heading, its line, and the divider of the icon-only sidebar.
    let app = window_class.strip_suffix("-window").unwrap_or(window_class);
    for (name, c) in look.sections.get(app).into_iter().flatten() {
        let name: String = name.chars().filter(|ch| ch.is_ascii_alphanumeric() || *ch == '-').collect();
        css.push_str(&format!(
            "window.{window_class} .section-{name} {{ color: {c}; }}\n\
             window.{window_class} .nav-group.section-{name} {{ border-bottom-color: alpha({c}, 0.35); }}\n\
             window.{window_class} separator.nav-divider.section-{name} {{ background: alpha({c}, 0.6); }}\n"
        ));
    }
    if let Some(mono) = &look.font.mono {
        css.push_str(&format!(
            "window.{window_class} .monospace, window.{window_class} .mono, window.{window_class} .code-block {{ font-family: \"{}\", monospace; }}\n",
            mono.replace('"', "")
        ));
    }
    css
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_themes_set_every_colour() {
        for (id, text) in SHIPPED {
            let f = ThemeFile::parse(text, id).expect("shipped theme parses");
            let all = [
                &f.bg,
                &f.surface,
                &f.text,
                &f.dim_text,
                &f.accent,
                &f.border,
                &f.muted,
                &f.highlight,
                &f.danger,
                &f.glow,
                &f.shadow,
            ];
            assert!(f.name.is_some() && f.light.is_some() && all.iter().all(|c| c.is_some()), "{id} leaves a key out");
            assert!(all.iter().all(|c| gdk::RGBA::parse(c.as_deref().unwrap()).is_ok()), "{id} has a bad colour");
        }
    }

    #[test]
    fn partial_file_keeps_the_rest() {
        let base = shipped_files().into_iter().find(|(id, _)| *id == "nixe-dark").unwrap().1;
        let top =
            ThemeFile::parse("accent = \"#ff0000\"\nbg = \"not a colour\"\n[accents]\nfiles = \"#00ff00\"", "test").unwrap();
        let t = top.over(&base).theme("nixe-dark", "test");
        let shipped = base.theme("nixe-dark", "test");
        assert_eq!(t.palette.accent, "#ff0000");
        assert_eq!(t.palette.bg, fallback_palette().bg, "a bad colour falls back");
        assert_eq!(t.palette.text, shipped.palette.text, "a missing key keeps the shipped value");
        assert_eq!(t.look.accents["files"], "#00ff00");
        assert_eq!(t.look.accents.get("mail"), shipped.look.accents.get("mail"), "other accents stay");
        assert_eq!(t.look.font, shipped.look.font);
    }

    #[test]
    fn derives_what_a_theme_leaves_out() {
        let t = ThemeFile::parse("bg = \"#000000\"\ntext = \"#ffffff\"", "test").unwrap().theme("x", "test");
        assert_eq!(t.look.card, "#0e0e0e");
        assert_eq!(t.look.shape.radius_card, 10);
        assert_eq!(t.look.material.surface, Surface::Solid);
        assert!(t.look.font.ui.is_none(), "no font unless the theme sets one");
    }

    #[test]
    fn full_copy_round_trips() {
        for theme in bundled() {
            let text = toml::to_string_pretty(&ThemeFile::of(&theme)).unwrap();
            let back = ThemeFile::parse(&text, "copy").unwrap().theme(&theme.id, "copy");
            assert_eq!(back.palette, theme.palette, "{}", theme.id);
            assert_eq!(back.look, theme.look, "{}", theme.id);
        }
    }

    #[test]
    fn swatches_keep_their_names_across_themes() {
        for theme in bundled() {
            let names: Vec<_> = theme.look.swatches.iter().take(SWATCH_NAMES.len()).map(|(n, _)| n.as_str()).collect();
            assert_eq!(names, SWATCH_NAMES, "{}", theme.id);
        }
        let light = bundled().into_iter().find(|t| t.id == "nixe-light").unwrap();
        assert_eq!(light.look.swatches[0], ("blue".to_string(), "#2f6fea".to_string()));
    }

    #[test]
    fn follow_omarchy_keeps_the_nixe_look() {
        let table: toml::Table = toml::from_str("blue = \"#89b4fa\"\nmagenta = \"#f5c2e7\"").unwrap();
        let look = omarchy_look(&fallback_palette(), &table);
        assert!(look.material.blended);
        assert_eq!(look.font.ui.as_deref(), Some("Inter"));
        assert_eq!(look.accents["files"], "#89b4fa");
        assert!(look.swatches.iter().any(|(n, c)| n == "violet" && c == "#f5c2e7"));
    }

    #[test]
    fn section_accents_name_swatches() {
        let dark = bundled().into_iter().find(|t| t.id == "nixe-dark").unwrap();
        let blue = dark.look.swatches.iter().find(|(n, _)| n == "blue").unwrap().1.clone();
        assert_eq!(dark.look.sections["settings"]["connections"], blue);
        let css = look_css("settings-window", &blue, &dark.look);
        assert!(css.contains(".nav-group.section-connections { border-bottom-color: alpha("));
        let table: toml::Table = toml::from_str("blue = \"#89b4fa\"").unwrap();
        let omarchy = omarchy_look(&fallback_palette(), &table);
        assert_eq!(omarchy.sections["settings"]["connections"], "#89b4fa");
    }

    #[test]
    fn unknown_keys_are_rejected() {
        assert!(ThemeFile::parse("acent = \"#ff0000\"", "test").is_none());
    }

    #[test]
    fn app_accent_and_font_in_css() {
        let t = bundled().into_iter().find(|t| t.id == "nixe-dark").unwrap();
        let css = look_css("files-window", &t.look.accents["files"], &t.look);
        assert!(css.contains(&format!("@define-color theme_app_accent {};", t.look.accents["files"])));
        assert!(css.contains("font-family: \"Inter\""));
        let plain = bundled().into_iter().find(|t| t.id == "nord").unwrap();
        assert!(!look_css("files-window", &plain.palette.accent, &plain.look).contains("font-family"));
    }
}
