#!/usr/bin/env python3
"""Verifica hash, lista permitida, instalação isolada e execução do pacote."""
import argparse
import hashlib
import os
import pathlib
import subprocess
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
            assert relative.startswith(('bin/', 'docs/', 'config/', 'scripts/')) or relative in ['README.md', 'LICENSE', 'LICENSE-STATUS.md', 'install.sh', 'package.json', 'THIRD-PARTY-NOTICES.txt']
            assert '.key' not in relative and 'session.json' not in relative
        archive.extractall(root, filter='data')
    package = root / next(iter(prefixes))
    prefix = root / 'prefix'
    subprocess.run([str(package / 'install.sh')], env={**os.environ, 'PREFIX': str(prefix)}, check=True, capture_output=True)
    for binary in ['crewsync-client', 'crewsync-server']:
        subprocess.run([str(prefix / 'bin' / binary), '--version'], check=True)
        subprocess.run([str(prefix / 'bin' / binary), '--help'], stdout=subprocess.DEVNULL, check=True)
    subprocess.run([str(prefix / 'bin/crewsync-server'), 'check-config', str(package / 'config/server.example.toml')], check=True)
    print('Pacote, checksum, instalação e comandos básicos: OK')
