#!/usr/bin/env python3
"""Verifica hash, lista permitida, instalação isolada e execução do pacote."""
import argparse
import hashlib
import json
import os
import pathlib
import subprocess
import sys
import tarfile
import tempfile

parser = argparse.ArgumentParser()
parser.add_argument('archive', type=pathlib.Path)
args = parser.parse_args()
checksum = args.archive.with_name(args.archive.name + '.sha256').read_text().split()[0]
assert hashlib.file_digest(args.archive.open('rb'), 'sha256').hexdigest() == checksum
with tempfile.TemporaryDirectory(prefix='crewsync-package-') as root:
    root = pathlib.Path(root)
    with tarfile.open(args.archive) as archive:
        members = archive.getmembers()
        prefixes = {m.name.split('/')[0] for m in members}
        assert len(prefixes) == 1
        for item in members:
            path = pathlib.PurePosixPath(item.name)
            assert not path.is_absolute() and '..' not in path.parts and item.isfile()
            relative = '/'.join(path.parts[1:])
            assert relative.startswith(('bin/', 'docs/', 'config/', 'scripts/')) or relative in ['README.md', 'LICENSE', 'install.sh', 'setup.sh', 'package.json', 'THIRD-PARTY-NOTICES.txt']
            assert '.key' not in relative and 'session.json' not in relative
        archive.extractall(root, filter='data')
    package = root / next(iter(prefixes))
    assert 'Version 3, 29 June 2007' in (package / 'LICENSE').read_text()
    metadata = json.loads((package / 'package.json').read_text())
    assert metadata['license'] == 'GPL-3.0-only'
    assert (package / 'setup.sh').is_file()
    prefix = root / 'prefix'
    subprocess.run([str(package / 'install.sh')], env={**os.environ, 'PREFIX': str(prefix)}, check=True, capture_output=True)
    for binary in ['crewsync-client', 'crewsync-server']:
        output = subprocess.check_output([str(prefix / 'bin' / binary), '--version'], text=True)
        assert output.strip().endswith(metadata['version'])
        print(output.strip())
        subprocess.run([str(prefix / 'bin' / binary), '--help'], stdout=subprocess.DEVNULL, check=True)
    subprocess.run([str(prefix / 'bin/crewsync-server'), 'check-config', str(package / 'config/server.example.toml')], check=True)
    assert (prefix / 'share/crewsync/MANUAL.md').is_file()
    assert (prefix / 'share/crewsync/server.example.toml').is_file()
    subprocess.run([sys.executable, str(pathlib.Path(__file__).resolve().parents[1] / 'tests/packaging/test_setup.py')], env={**os.environ, 'CREWSYNC_PACKAGE_DIR': str(package), 'CREWSYNC_BIN_DIR': str(package / 'bin')}, check=True)
    print('Pacote, checksum, instalação e comandos básicos: OK')
