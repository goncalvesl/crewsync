#!/usr/bin/env python3
"""Confere fontes/checksum e compila offline sem cache de crates do usuário."""
import argparse
import hashlib
import os
from pathlib import Path
import subprocess
import tarfile
import tempfile

parser = argparse.ArgumentParser()
parser.add_argument('archive', type=Path)
args = parser.parse_args()
with args.archive.open('rb') as data:
    assert hashlib.file_digest(data, 'sha256').hexdigest() == args.archive.with_name(args.archive.name + '.sha256').read_text().split()[0]
with tempfile.TemporaryDirectory(prefix='crewsync-source-check-') as temporary:
    root = Path(temporary)
    with tarfile.open(args.archive) as tar:
        assert all(item.isfile() or item.isdir() for item in tar.getmembers())
        tar.extractall(root, filter='data')
    source, = [p for p in root.iterdir() if p.is_dir()]
    assert (source / 'vendor').is_dir() and (source / 'LICENSE').is_file()
    subprocess.run(['cargo', 'build', '--workspace', '--locked', '--offline'], cwd=source,
        env={**os.environ, 'CARGO_HOME': str(root / 'empty-cargo-home'), 'CARGO_TARGET_DIR': str(root / 'build')}, check=True, timeout=300)
    print('Fonte correspondente compila offline sem cache: OK')
