# Nixe App Style — 2.0

How Nixe apps look and behave. Most of it comes from the shared crates, so an app gets it
by using them rather than by copying another app:

| Crate | What it gives an app |
| --- | --- |
| `nixe-core` | Identity, XDG paths, preferences store, commands, atomic writes |
| `nixe-theme` | Theme files, Follow Omarchy, the suite settings, the shared stylesheet |
| `nixe-ui` | Window frame and header, sidebar, rows, cards, controls, dialogs, palette, keys |
| `nixe-icons` | App icons and symbolic header icons |

Apps move to 2.0 one at a time. Until an app has moved it follows
[docs/STYLE-v1.md](docs/STYLE-v1.md). An app moves by calling
`nixe_theme::use_design_2()` before `nixe_theme::install`: the 2.0 sheet
(`crates/nixe-theme/src/base2.css`) then loads on top of the shared one, the app's own
accent becomes `@theme_accent`, and sizes follow the 2.0 density table. Check that GTK
accepts the sheets with `cargo run -p nixe-theme --example check_css`. The mockups this guide comes from are the
*Nixe Design 2.0* canvas (Files, Notes, Settings, Photos, Mail, Calendar, Icons).

---

## 1. Principles

- **Compact, structured, breathable.** Dense enough to use a tiled half-screen window,
  with spacing doing the work lines used to do.
- **One surface ("blended").** Header, sidebar, tabs, content and status bar share one
  background. No border lines or different fills between areas. Structure comes from
  spacing, alignment, the selection pill and cards.
- **Solid.** Opaque surfaces by default. Frosted is a theme option, not the default.
- **Neutral, with accent only for state.** Near-black (dark) or near-white (light) with
  no colour tint. The app's accent marks where you are and what is on: the selected nav
  item and row, focus, switches, the primary button, progress. Never whole surfaces.
- **Plain type.** Inter, regular weights. No glows, no mono-for-style, no wide tracking.
- **Icon buttons.** Actions are icon buttons with a tooltip naming the action and its key
  ("New folder (Ctrl+Shift+N)"). Text only where an icon can't say it: a dialog's
  confirm/cancel pair, a banner's single action, the second click of a dangerous action.
- **Works tiled at 800 px** with no horizontal scroll (see Responsive).
- **Everything from tokens.** No hard-coded colours in components; every colour is a
  theme token or a mix of them.

---

## 2. Themes

A theme is one TOML file. Every key is optional; what a file leaves out comes from the
theme it overrides or `inherits`, or is derived from its core colours.

- **Shipped themes** live in `crates/nixe-theme/themes/` and are built in.
  `nixe-dark.toml` sets every key and is the template. Default themes: **Nixe Dark** and
  **Nixe Light**.
- **User themes** go in `~/.config/nixe/themes/` (every app) or
  `~/.config/<app>/themes/` (one app). A file named like a shipped theme overrides it
  with only the keys it sets; any other name is a new theme, `custom:<file stem>`.
- **Live:** saving a theme file restyles every open Nixe app within a second.
- **Safe:** an unknown key rejects the file (with the error on stderr); a value that
  isn't a colour falls back with a warning. An app always starts.
- `nixe_theme::edit_copy(id)` writes a full copy of a theme to edit.

### What a theme sets

| Keys | Meaning | In CSS |
| --- | --- | --- |
| `name`, `light`, `inherits` | Display name, light or dark, theme to start from | |
| `bg` | Window and content | `@theme_bg` |
| `panel` | Sidebar and header (equal to `bg` when blended) | `@theme_panel` |
| `card`, `raised` | Cards and grouped rows; raised controls (selected segment) | `@theme_card`, `@theme_raised` |
| `surface`, `popover` | Toasts; popovers, menus, dialogs | `@theme_surface`, `@theme_popover` |
| `text`, `dim_text`, `text_tertiary` | Primary, secondary, hint text | `@theme_text`, … |
| `border`, `divider` | Outlines where one is needed; lines inside a card | `@theme_border`, `@theme_divider` |
| `accent` | The theme accent, used when an app has none | `@theme_accent` |
| `danger`, `warning`, `success` | Status colours | `@theme_danger`, … |
| `muted`, `highlight`, `glow`, `shadow` | Neutral fills, spare selection tone, kept for v1, shadows | |
| `hover`, `selection` | Accent strength on hovered and selected rows, 0–1 | `--nixe-hover`, `--nixe-selection` |
| `[accents]` | Default accent per app (`files = "#5b9cff"`) | `@theme_app_accent` |
| `[sections.<app>]` | Accent per section of an app (`[sections.settings] privacy = …`) | |
| `[swatches]` | Named accent choices offered in each app's settings (see below) | |
| `[font]` | `ui`, `mono`, `weight`, `heading_weight` | `font-family` on the window |
| `[shape]` | `radius_small`, `radius_card`, `radius_dialog` (px) | `--nixe-radius-*` |
| `[material]` | `surface` (`solid` / `frosted`), `see_through`, `blended` | |
| `[icons]` | `pack`: the icon theme it recommends | |

Sizes, density, text size and motion are **not** theme keys: density and text size are
suite preferences, motion follows Reduce motion, and layout belongs to the apps.

### App accents

- Each app has an **Accent** row in its settings: the active theme's `[swatches]` plus
  *Theme default* (the theme's `[accents]` entry for the app, else `accent`).
- The choice is stored **by swatch name** in `~/.config/nixe/theme.toml`
  (`[app_accents] files = "violet"`), so it survives a theme switch: every theme defines
  the same names, tuned for its background (Nixe Light's are deeper).
- Swatch names: `blue`, `violet`, `pink`, `red`, `coral`, `amber`, `green`, `teal`,
  `graphite`.
- Sections (Settings) use `[sections.<app>]`; a section without one uses the app's accent.
- In code: `nixe_theme::app_accent(app)`, `section_accent(app, section)`, `swatches()`.

### Follow Omarchy

The default mode. Read `~/.local/state/omarchy/current/theme/colors.toml` (respect
`$XDG_STATE_HOME`) and map it onto the core colours; the other surfaces are derived.
App defaults come from Omarchy's named colours (Files `blue`, Mail `magenta`, Notes
`yellow`, Photos `orange`, Calendar `red`, Home `green`, …) and the swatches from its
palette. Omarchy's theme is polled once a second and restyles in place. If
`colors.toml` is missing or invalid, use the chosen Nixe theme.

### Suite settings

`~/.config/nixe/theme.toml` holds what every app shares: `mode` (`omarchy` / `theme`),
`theme`, `density`, `text_scale` (0.9–1.25), `start` (`home` / `last`) and
`[app_accents]`. Changing any of it in one app reaches every open app within a second.

---

## 3. Typography

- **Font:** the theme's `[font] ui` — Inter in Nixe Dark and Light, with `sans-serif`
  as fallback. Code and readouts that need alignment use `[font] mono` (JetBrains Mono).
  Numbers that line up use tabular figures (`font-feature-settings: "tnum"`).
- **Weights:** 400 regular, 500 for row titles and emphasis, 600 for titles. Nothing bolder.
- **Base size:** 13 px; everything else relative.

| Element | Size | Weight | Colour |
| --- | --- | --- | --- |
| App name in the header | 13.5 px | 600 | app accent |
| Header subtitle (section, folder) | 12.5 px | 400 | `dim_text` |
| Nav item, row title, body | 13 px | 400 / 500 | `text` |
| Row description, secondary | 12–12.5 px | 400 | `dim_text` |
| Group label (sidebar, page) | 11 px | 600 | `text_tertiary`, UPPERCASE, 0.06em |
| Card title | 13 px | 600 | `text` |
| Status bar, counts, times | 11.5 px | 400 | `dim_text` / `text_tertiary` |
| Document title (editor h1) | 26 px | 600 | `text` |
| Big readout (gauge) | 22 px | 600 | `text` |

No page titles: the header shows the app and where you are. Copy is sentence case,
American spelling; descriptions say what an option does for the user in one sentence.

---

## 4. Shape, density, motion

| Thing | Value |
| --- | --- |
| Radii | `radius_small` 7 (buttons, inputs, chips, nav items, rows) · `radius_card` 11 (cards, banners, toasts) · `radius_dialog` 15 (popovers, dialogs, preview windows). Only switches are pills. |
| Spacing | multiples of 2 and 4; content padding 14–16 px; gap between cards 12 px |
| Sidebar | 212 px (236 for trees: Notes, Calendar); 56 px icon-only when compact |
| Inspector / drawer | 268 px (Notes) · 288 px (Photos info) |
| Transitions | 120 ms ease-out for hover and colour; 200 ms for panels and width changes |
| Reduce motion | every duration 0; toolkit animations off |

Density is a suite preference; **Balanced** is the default.

| Size | Compact | Balanced | Relaxed |
| --- | --- | --- | --- |
| Header | 42 | 46 | 50 |
| Nav item | 30 | 34 | 38 |
| List row | 30 | 36 | 42 |
| Option row (settings) | 44 | 52 | 60 |
| Control (button, input, segmented) | 30 | 32 | 34 |
| Page padding (top) | 12 | 16 | 22 |

---

## 5. Window and header

```
┌──────────────────────────────────────────────────────────────────────────┐
│ ▣ Files  ‹ ›  ken › Projects › nixe-design      [⌕ Search  Ctrl F] ☰ ▦ ◉ ⋯ │ header 46
│ ⌂ Recent          nixe-design ×  Downloads ×  +                           │
│ ★ Starred         Name ▾                     Size   Modified     Kind     │
│ ▌Projects ◀──     ▌README.md                 4.8 KB Today 13:40  Markdown │
│ DEVICES           …                                                       │
│ ▤ Samsung T7                                                              │
│ 13 items · 1 selected               ● Copying 3 files ▬▬▬▬▭ 64%  212 GB free│ status 28
└──────────────────────────────────────────────────────────────────────────┘
```

- **One unified header** (46 px balanced) instead of a title bar plus toolbar:
  - **Left:** the symbolic app icon (20 px) and the app name in the app accent; an
    optional subtitle in `dim_text` (section, folder, account).
  - **Middle:** what the app navigates by: a path, the document title, a view switch
    (Day/Week/Month), the selection toolbar while items are selected, or search.
  - **Right:** contextual icon actions, then the `⋯` menu.
  - **Window controls** (minimize, maximize, close) only on **floating** windows, after
    a small gap; hidden when tiled.
  - The empty parts of the header drag the window; double-click maximizes.
- **Status bar** (28 px): counts and selection on the left; live activity (copying,
  syncing, backing up) with a pulsing dot and a thin progress bar on the right.
- **Tabs** (Files, Notes) sit in a 34 px row under the header; the active tab is a pill
  in the hover tone; `+` opens one.
- **Default size** 1280 × 800 floating; the layout must work at 800 px tiled.

---

## 6. Layouts

Pick the archetype that fits; don't force an app into another's layout.

| Archetype | Apps | Structure |
| --- | --- | --- |
| **Explorer** | Files, Mail, Contacts | Sidebar · list (or list column) · detail. Files opens previews in a **separate floating window** (Space, Esc closes, ↑/↓ follows the selection), not a side panel. Mail adds a 360 px message list and a reading pane. |
| **Workspace** | Notes, Write, Notepad | Tree or no sidebar · tabbed editor with a ~680 px reading column · optional inspector (Outline / Links / Info). Split editor on demand. |
| **Dashboard** | Settings, Home, Pulse | Sidebar of sections · cards in flowing rows (`flex: n 1 <min>px`), pages up to ~760 px wide for option lists. |
| **Media grid** | Photos, Music, Theater | Sidebar · justified grid with date groups and a timeline scrubber · info drawer. Selecting turns the header's middle into a selection toolbar. |
| **Time grid** | Calendar, Reminders | Sidebar with mini month and calendars · day columns with an all-day row, hour lines and a now line · event popover. Quick add in the header ("Lunch with Sam tomorrow at 1"). |

### Sidebar

- 212 px, `bg` (blended), no line beside it. Rows are nav items: icon (17 px), label,
  optional count in `text_tertiary` or a badge (accent tint) for unread.
- Group labels: small UPPERCASE `text_tertiary`, 14 px above, optional `+` at the right.
- **Selected:** a pill in `mix(bg, app accent, selection)` with a 2 px accent bar at the
  left edge and the icon in the accent. Hover: the `--nixe-hover` tint.
- Collapses to 56 px icons below 980 px window width or with <kbd>Ctrl</kbd>+<kbd>B</kbd>
  (saved as `sidebar_collapsed`); labels, counts and group labels hide, tooltips stay.
  Tree sidebars (Notes, Calendar) hide entirely instead.
- Construction: the sidebar, its list and scroller are `hexpand(false)`; only the page
  column takes spare width.

### Responsive

At 800 px: the sidebar is icons (or hidden), inspectors and drawers float over the
content or hide, list columns drop the least useful column, Calendar shows five days.
Nothing is cut off and nothing scrolls sideways. Keep a page's minimum width under ~580 px.

---

## 7. Components

**Card:** `@theme_card` fill, `radius_card`, no outline when blended (a hairline in
`@theme_border` otherwise), 14 px inside. Header row: an accent icon (16 px), a 600 title,
an optional `dim_text` note or tag, actions at the right. Use cards for groups of related
controls or a summary, not around every item.

**Option row:** title (500) and one-line description (`dim_text`) on the left, the control
on the right, vertically centred; option-row height from density. Rows inside a card are
split by spacing when blended, by a `@theme_divider` line otherwise.

**List row:** list-row height, `radius_small`, hover tint, selected
`mix(bg, accent, selection)`. No zebra stripes, no row lines.

**Buttons:**
- Icon button: 32 px square, transparent, `dim_text` icon, hover tint and `text`.
- Toggled icon button: accent tint background, icon in the accent.
- Primary: a solid accent square with the icon in `on_accent`; at most one per view.
- Danger: hovers to `danger`; destructive actions need two clicks (arms for 4 s) or an
  Undo toast.

**Segmented control:** a `field` track (barely lighter than `bg`), 2 px inside; the
selected segment is `@theme_raised` with a soft shadow. Prefer it to a dropdown when the
options fit.

**Switch:** 36 × 20 pill; off `track`, on the accent with an `on_accent` knob.

**Search field:** `field` fill, `radius_small`, a search icon and the key hint (`Ctrl F`)
at the right; focus shows an accent outline.

**Chip / tag:** 24 px chip with `field` fill; accent chips for tags (`#design`).
Tags for row status ("Restart", "5 available") are 11 px, 600, accent or warning tint.

**Popover, dialog, preview window:** `@theme_popover`, `radius_dialog`, a soft shadow.
Dialogs keep their text confirm/cancel pair, safe choice first.

**Toast:** bottom-centre over the content, `@theme_surface`, `radius_card`; replaces any
toast showing; 3.5 s, 6 s with Undo. Confirm actions and report errors with toasts, not
alerts.

**Progress and meters:** 3–6 px tracks in `track` with an accent fill; activity in the
status bar gets a pulsing dot (still when Reduce motion is on).

**Gauges and graphs:** 64 px ring gauges and thin sparklines in the accent on `track`.

**Empty, loading and error states:** an icon, one line of what's going on and, if useful,
one action — centred in the content area.

---

## 8. Keys every app shares

| Key | Does |
| --- | --- |
| <kbd>Ctrl</kbd>+<kbd>K</kbd> | Command palette: every page, command and named button |
| <kbd>Ctrl</kbd>+<kbd>F</kbd> | Search |
| <kbd>Ctrl</kbd>+<kbd>,</kbd> | Settings |
| <kbd>Ctrl</kbd>+<kbd>B</kbd> | Collapse the sidebar |
| <kbd>F1</kbd> / <kbd>?</kbd> | Keyboard shortcuts |
| <kbd>Esc</kbd> | Clear, dismiss, close transient UI |
| <kbd>Space</kbd> | Preview where an app supports it |
| <kbd>F10</kbd> | The `⋯` menu |
| <kbd>Ctrl</kbd>+<kbd>W</kbd> / <kbd>Ctrl</kbd>+<kbd>Q</kbd> | Close the tab or document / quit |

Right-click opens a context menu wherever there are item actions. Check an app's
existing shortcuts before adding these; an established app key wins and the conflict is
noted in its shortcuts list.

---

## 9. Icons

- **App icons:** a 128 px canvas, a 112 px rounded square (radius 28) in a dark satin
  gradient (`#26262a` → `#121214`) with a faint accent glow at the top-left and an accent
  hairline at 0.3. One layered geometric glyph in the app's accent (a light-to-mid
  gradient over a 35% back layer). Must read at 24 px.
- **Symbolic icons:** 24 px grid, 1.6 px round-capped strokes, `currentColor`; the header
  icon is the app's symbolic at 20 px in the app accent.
- Freedesktop icon-theme layout, falling back to the user's icon theme; third-party logos
  are never recoloured.

---

## 10. Materials

- **Solid** (default): opaque surfaces.
- **Frosted** (`[material] surface = "frosted"`): the window is translucent and
  Hyprland's blur shows a darkened, desaturated desktop behind it. `see_through = false`
  keeps the frosted texture without showing the desktop. Without compositor blur it is
  solid. GTK can't blur its own content, so there is no per-panel glass.
- **Blended** (`blended = true`, the default): one surface, as described in Principles.
