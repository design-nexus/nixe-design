"""Generate Nyx 2.0 launcher icons from app-glyphs.json.

Files, Notes, Settings, Photos, Mail, Music and Home come from Claude's
Nyx Design 2.0 Icons board. Companion glyphs follow the same satin tile,
app accent and layered geometric treatment. Run this file to regenerate all assets.
"""
import base64
import json
import pathlib
from xml.sax.saxutils import escape

TEMPLATE = '''<svg xmlns="http://www.w3.org/2000/svg" width="128" height="128" viewBox="0 0 128 128">
  <defs>
    <linearGradient id="tile" x1="0" y1="0" x2="1" y2="1"><stop stop-color="#26262a"/><stop offset="1" stop-color="#121214"/></linearGradient>
    <radialGradient id="glow" cx=".2" cy=".1" r=".9"><stop stop-color="{light}" stop-opacity=".22"/><stop offset="1" stop-color="{light}" stop-opacity="0"/></radialGradient>
    <linearGradient id="ink" x1="0" y1="0" x2="1" y2="1"><stop stop-color="{light}"/><stop offset="1" stop-color="{mid}"/></linearGradient>
  </defs>
  <rect x="8" y="8" width="112" height="112" rx="28" fill="url(#tile)"/>
  <rect x="8" y="8" width="112" height="112" rx="28" fill="url(#glow)"/>
  <rect x="8.5" y="8.5" width="111" height="111" rx="27.5" fill="none" stroke="{light}" stroke-opacity=".3"/>
  <path d="{back}" fill="{light}" fill-opacity=".35"/>
  <path d="{front}" fill="url(#ink)"/>
  <path d="{detail}" fill="none" stroke="#151517" stroke-width="5" stroke-linecap="round" stroke-linejoin="round"/>
</svg>
'''

def render_glyph(glyph):
    """Render a curated glyph using the canonical Nyx tile."""
    return TEMPLATE.format(**{key: escape(value, {chr(34): "&quot;"})
                              for key, value in glyph.items()})


def render_logo(png, light="#a7b1c7", mid="#8590a8"):
    """Embed a rasterized original logo without recoloring third-party artwork."""
    svg = render_glyph(dict(light=light, mid=mid, back="", front="", detail=""))
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
