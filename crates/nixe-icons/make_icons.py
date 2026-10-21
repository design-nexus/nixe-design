"""Writes the suite's app icons from one template: a dark rounded tile with a
hairline, and each app's glyph drawn as round-capped lines in the accent
gradient. Run it after changing a glyph: python3 make_icons.py"""
import pathlib

TEMPLATE = '''<svg xmlns="http://www.w3.org/2000/svg" width="128" height="128" viewBox="0 0 128 128">
  <defs>
    <linearGradient id="tile" x1="8" y1="8" x2="120" y2="120" gradientUnits="userSpaceOnUse">
      <stop offset="0" stop-color="#2a2f45"/>
      <stop offset="1" stop-color="#16182a"/>
    </linearGradient>
    <linearGradient id="ink" x1="32" y1="32" x2="96" y2="96" gradientUnits="userSpaceOnUse">
      <stop offset="0" stop-color="#7aa2f7"/>
      <stop offset="1" stop-color="#c58bd6"/>
    </linearGradient>
  </defs>
  <rect x="8" y="8" width="112" height="112" rx="26" fill="url(#tile)"/>
  <rect x="8.5" y="8.5" width="111" height="111" rx="25.5" fill="none" stroke="#7aa2f7" stroke-opacity="0.4"/>
  <g fill="none" stroke="url(#ink)" stroke-width="6" stroke-linecap="round" stroke-linejoin="round">
{glyph}
  </g>
</svg>
'''

# Shapes filled with the tile colour cut a gap into the lines behind them.
CUT = '#1f2337'

GLYPHS = {
    # A commit lane branching into a second lane.
    'Git': '''    <path d="M46 44V84M82 44V56C82 68 46 62 46 76"/>
    <circle cx="46" cy="38" r="7"/>
    <circle cx="82" cy="38" r="7"/>
    <circle cx="46" cy="90" r="7"/>''',
    # A shopping bag with a handle.
    'Store': '''    <path d="M40 48H88L94 94H34Z"/>
    <path d="M52 50V42A12 12 0 0 1 76 42V50"/>
    <path d="M54 72H74M64 62V82"/>''',
    # An address book with a person.
    'Contacts': '''    <rect x="40" y="32" width="50" height="64" rx="6"/>
    <path d="M32 46H44M32 64H44M32 82H44"/>
    <circle cx="65" cy="53" r="8"/>
    <path d="M51 80V77A14 14 0 0 1 79 77V80"/>''',
    # Sliders
    'Settings': f'''    <path d="M36 44H92M36 64H92M36 84H92"/>
    <circle cx="52" cy="44" r="7" fill="{CUT}"/>
    <circle cx="76" cy="64" r="7" fill="{CUT}"/>
    <circle cx="58" cy="84" r="7" fill="{CUT}"/>''',
    # A pulse line
    'Tasks': '''    <path d="M32 66H46L54 44L66 88L74 58L80 66H96"/>''',
    # Down into a tray
    'Torrents': '''    <path d="M64 34V70M50 56L64 70L78 56"/>
    <path d="M38 74V90H90V74"/>''',
    # A page with a folded corner and a bookmark
    'Pdf': '''    <path d="M44 32H72L88 48V96H44Z"/>
    <path d="M72 32V48H88"/>
    <path d="M54 32V58L61 52L68 58V32"/>
    <path d="M54 74H78M54 86H70"/>''',
    # A page and a pencil
    'Notepad': '''    <path d="M76 34H44V94H66"/>
    <path d="M52 50H68M52 62H64"/>
    <path d="M70 92L72 82L90 64L98 72L80 90Z"/>''',
    # A note
    'Music': '''    <path d="M56 84V42L88 36V76"/>
    <circle cx="47" cy="84" r="9"/>
    <circle cx="79" cy="76" r="9"/>''',
    # Play in a frame
    'MediaPlayer': '''    <rect x="34" y="40" width="60" height="48" rx="8"/>
    <path d="M58 54L76 64L58 74Z"/>''',
    # A cinema screen with two seats
    'Theater': '''    <path d="M34 36H94V70H34Z"/>
    <path d="M58 44L74 53L58 62Z"/>
    <path d="M40 86V94M52 86V94M76 86V94M88 86V94M38 82H54M74 82H90"/>''',
    # A speech bubble
    'Chat': '''    <path d="M44 36H84A8 8 0 0 1 92 44V70A8 8 0 0 1 84 78H60L46 90V78H44A8 8 0 0 1 36 70V44A8 8 0 0 1 44 36Z"/>
    <path d="M52 57H52.01M64 57H64.01M76 57H76.01"/>''',
    # Aa
    'Fonts': '''    <path d="M32 88L50 40L68 88M39 72H61"/>
    <circle cx="83" cy="78" r="10"/>
    <path d="M93 66V88"/>''',
    # A box with a zip
    'Archive': '''    <rect x="32" y="34" width="64" height="14" rx="4"/>
    <path d="M38 48V86A6 6 0 0 0 44 92H84A6 6 0 0 0 90 86V48"/>
    <path d="M64 52V58M58 64H70M64 70V76M58 82H70"/>''',
    # A page of text
    'Write': '''    <rect x="40" y="32" width="48" height="64" rx="6"/>
    <path d="M50 48H78M50 60H78M50 72H78M50 84H68"/>''',
    # A grid
    'Sheets': '''    <rect x="34" y="38" width="60" height="52" rx="6"/>
    <path d="M34 56H94M34 72H94M54 38V90M74 38V90"/>''',
    # A board on an easel
    'Present': '''    <rect x="34" y="34" width="60" height="40" rx="4"/>
    <path d="M46 62L56 52L66 58L80 46"/>
    <path d="M54 74L46 94M74 74L82 94M64 74V82"/>''',
    # Mountains and a sun in a frame
    'Photos': '''    <rect x="34" y="40" width="60" height="48" rx="8"/>
    <path d="M40 82L56 64L66 74L74 66L88 82"/>
    <circle cx="78" cy="54" r="4"/>''',
    # A photo between the corners of a screen
    'PhotoViewer': '''    <path d="M34 48V38H44M84 38H94V48M94 80V90H84M44 90H34V80"/>
    <path d="M46 78L57 64L65 72L71 66L82 78"/>''',
    # Pages in a stack
    'Office': '''    <rect x="48" y="30" width="42" height="54" rx="6"/>
    <path d="M40 42V92A6 6 0 0 0 46 98H78"/>
    <path d="M58 46H80M58 58H80M58 70H72"/>''',
    # A cloud and sun with rain
    'Weather': '''    <path d="M46 54A12 12 0 1 1 66 44"/>
    <path d="M44 76H82A12 12 0 0 0 82 52A16 16 0 0 0 52 48A14 14 0 0 0 44 76Z"/>
    <path d="M50 86L46 94M64 86L60 94M78 86L74 94"/>''',
    # A folded page of headlines
    'News': '''    <path d="M36 40H92V92H36Z"/>
    <path d="M46 54H82M46 66H74M46 78H68"/>''',
    # A crop frame around a screen
    'Capture': '''    <rect x="40" y="44" width="48" height="36" rx="4"/>
    <path d="M34 40V32H42M86 32H94V40M94 88V96H86M42 96H34V88"/>''',
    # A padlock with a keyhole.
    'Passwords': '''    <path d="M48 56V46A16 16 0 0 1 80 46V56"/>
    <rect x="38" y="56" width="52" height="38" rx="8"/>
    <path d="M64 70V80"/>''',
}

out = pathlib.Path(__file__).parent / 'apps'
out.mkdir(exist_ok=True)
for name, glyph in GLYPHS.items():
    (out / f'io.github.design_nexus.nixe.{name}.svg').write_text(TEMPLATE.format(glyph=glyph))
print(f'{len(GLYPHS)} icons in {out}')
