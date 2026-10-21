#!/usr/bin/env python3
"""Package canonical design resources without altering generated icons."""
from pathlib import Path
import subprocess
import tarfile
root = Path(__file__).resolve().parents[1]
revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip()
(root / 'dist').mkdir(exist_ok=True)
with tarfile.open(root / 'dist' / f'nixe-design-{revision}.tar.gz', 'w:gz') as archive:
    for name in ['STYLE.md', 'LICENSE', 'crates/nixe-icons/apps', 'crates/nixe-icons/make_icons.py',
                 'crates/nixe-theme/src', 'crates/nixe-ui/src', 'crates/nixe-playback/data',
                 'crates/nixe-playback/src/controls.css']:
        archive.add(root / name, arcname=name)
print(root / 'dist' / f'nixe-design-{revision}.tar.gz')
