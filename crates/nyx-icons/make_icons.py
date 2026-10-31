"""Generate Nyx 2.0 launcher icons from app-glyphs.json.

Files, Notes, Settings, Photos, Mail, Music and Home come from Claude's
Nyx Design 2.0 Icons board. Companion glyphs follow the same satin tile,
app accent and layered geometric treatment. Run this file to regenerate all assets.

Nyx Dark is the reference look: the files in apps/ are rendered with it. The
helpers below restyle the same glyphs for another theme's background and
accents (`nyx icons sync`).
"""
import base64
import colorsys
import json
import pathlib
from xml.sax.saxutils import escape

TEMPLATE = '''<svg xmlns="http://www.w3.org/2000/svg" width="128" height="128" viewBox="0 0 128 128">
  <defs>
    <linearGradient id="tile" x1="0" y1="0" x2="1" y2="1"><stop stop-color="{tile_top}"/><stop offset="1" stop-color="{tile_bottom}"/></linearGradient>
    <radialGradient id="glow" cx=".2" cy=".1" r=".9"><stop stop-color="{light}" stop-opacity=".22"/><stop offset="1" stop-color="{light}" stop-opacity="0"/></radialGradient>
    <linearGradient id="ink" x1="0" y1="0" x2="1" y2="1"><stop stop-color="{light}"/><stop offset="1" stop-color="{mid}"/></linearGradient>
  </defs>
  <rect x="8" y="8" width="112" height="112" rx="28" fill="url(#tile)"/>
  <rect x="8" y="8" width="112" height="112" rx="28" fill="url(#glow)"/>
  <rect x="8.5" y="8.5" width="111" height="111" rx="27.5" fill="none" stroke="{light}" stroke-opacity=".3"/>
  <path d="{back}" fill="{light}" fill-opacity=".35"/>
  <path d="{front}" fill="url(#ink)"/>
  <path d="{detail}" fill="none" stroke="{cut}" stroke-width="5" stroke-linecap="round" stroke-linejoin="round"/>
</svg>
'''

# The Nyx Dark tile: gradient stops and the colour detail strokes are cut in.
TILE = {"tile_top": "#26262a", "tile_bottom": "#121214", "cut": "#151517"}
NYX_DARK_BG, NYX_DARK_TEXT = "#09090a", "#e8e8ea"

# Nyx Dark's swatches and the hand-tuned light/mid glyph pair for each.
SWATCHES = {
    "blue": "#5b9cff", "violet": "#9d7bff", "pink": "#ff6fae", "red": "#ff6b6b",
    "coral": "#ff7a59", "amber": "#f2b545", "green": "#46d08a", "teal": "#4cc9d6",
    "graphite": "#a3a3ab",
}
TUNED = {
    "blue": ("#86baff", "#4b8ef5"), "violet": ("#bba6ff", "#8f6bff"),
    "pink": ("#ff97c6", "#fa5d9f"), "red": ("#ff9a9e", "#ee5864"),
    "coral": ("#ffa184", "#fb6f4a"), "amber": ("#ffd27a", "#f0a83a"),
    "green": ("#79e6ad", "#3cc57f"), "teal": ("#7ee2ee", "#35afc9"),
    "graphite": ("#a7b1c7", "#8590a8"),
}


def _rgb(color):
    color = color.lstrip("#")
    return tuple(int(color[i:i + 2], 16) / 255 for i in (0, 2, 4))


def _hex(rgb):
    return "#" + "".join(f"{round(max(0, min(1, c)) * 255):02x}" for c in rgb)


def mix(a, b, t):
    """`a` moved towards `b` by `t`, as #rrggbb."""
    return _hex(x + (y - x) * t for x, y in zip(_rgb(a), _rgb(b)))


def tile_for(bg, text, light):
    """The tile for a theme's background and text colours."""
    bg, text = bg.lower(), text.lower()
    if (bg, text) == (NYX_DARK_BG, NYX_DARK_TEXT):
        return dict(TILE)
    if light:
        return {"tile_top": mix(bg, "#ffffff", .7), "tile_bottom": mix(bg, text, .07),
                "cut": mix(bg, "#ffffff", .5)}
    return {"tile_top": mix(bg, text, .13), "tile_bottom": mix(bg, text, .04), "cut": mix(bg, text, .05)}


def family(color):
    """The Nyx swatch name a glyph colour belongs to, by hue."""
    h, s, _ = colorsys.rgb_to_hsv(*_rgb(color))
    if s < .25:
        return "graphite"
    def distance(name):
        other = colorsys.rgb_to_hsv(*_rgb(TUNED[name][0]))[0]
        return min(abs(h - other), 1 - abs(h - other))
    return min((n for n in TUNED if n != "graphite"), key=distance)


def glyph_colors(accent):
    """The light/mid glyph pair for one accent colour."""
    accent = accent.lower()
    for name, swatch in SWATCHES.items():
        if swatch == accent:
            return TUNED[name]
    return mix(accent, "#ffffff", .28), mix(accent, "#000000", .08)


def themed(glyph, accent=None, tile=None):
    """`glyph` in `accent` on `tile`. A glyph keeps its own colours while
    the accent is still the Nyx Dark swatch it was drawn for."""
    glyph = dict(glyph)
    if accent and accent.lower() != SWATCHES[family(glyph["light"])]:
        glyph["light"], glyph["mid"] = glyph_colors(accent)
    return render_glyph(glyph, tile)


def render_glyph(glyph, tile=None):
    """Render a curated glyph on the canonical Nyx tile, or on `tile`."""
    values = {key: escape(value, {chr(34): "&quot;"}) for key, value in glyph.items()}
    return TEMPLATE.format(**{**TILE, **(tile or {}), **values})


def render_logo(png, light="#a7b1c7", mid="#8590a8", tile=None):
    """Embed a rasterized original logo without recoloring third-party artwork."""
    svg = render_glyph(dict(light=light, mid=mid, back="", front="", detail=""), tile)
    logo = base64.b64encode(png).decode("ascii")
    return svg.replace("</svg>", '<image x="28" y="28" width="72" height="72" '
                       'preserveAspectRatio="xMidYMid meet" href="data:image/png;base64,'
                       + logo + '"/></svg>')


def main():
    base = pathlib.Path(__file__).resolve().parent
    out = base / "apps"
    out.mkdir(exist_ok=True)
    icons = json.loads((base / "app-glyphs.json").read_text())
    for name, glyph in icons.items():
        (out / f"io.github.design_nexus.nyx.{name}.svg").write_text(
            render_glyph(glyph))
    print(f"{len(icons)} Nyx 2.0 icons in {out}")

if __name__ == "__main__":
    main()
