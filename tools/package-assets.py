#!/usr/bin/env python3
"""Package canonical design resources with reproducible archive metadata."""
import gzip
import os
from pathlib import Path
import subprocess
import tarfile
root = Path(__file__).resolve().parents[1]
revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip()
epoch = int(os.environ.get('SOURCE_DATE_EPOCH', subprocess.check_output(['git', 'log', '-1', '--format=%ct'], cwd=root, text=True).strip()))
(root / 'dist').mkdir(exist_ok=True)
output = root / 'dist' / f'nyx-design-{revision}.tar.gz'
def normalize(info):
    info.uid = info.gid = 0
    info.uname = info.gname = ''
    info.mtime = epoch
    info.pax_headers = {}
    info.mode = 0o755 if info.isdir() or info.mode & 0o111 else 0o644
    return info
with output.open('wb') as raw, gzip.GzipFile(filename='', fileobj=raw, mode='wb', mtime=0) as compressed:
    with tarfile.open(fileobj=compressed, mode='w') as archive:
        for name in ['STYLE.md', 'LICENSE', 'crates/nyx-icons/apps', 'crates/nyx-icons/make_icons.py', 'crates/nyx-icons/app-glyphs.json', 'crates/nyx-icons/third-party-glyphs.json',
                     'crates/nyx-theme/src', 'crates/nyx-theme/themes', 'crates/nyx-ui/src', 'crates/nyx-playback/data',
                     'crates/nyx-playback/src/controls.css']:
            archive.add(root / name, arcname=name, filter=normalize)
print(output)
