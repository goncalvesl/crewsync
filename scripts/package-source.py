#!/usr/bin/env python3
"""Fonte correspondente, scripts de build e dependências vendorizadas da release."""
import argparse
import hashlib
from pathlib import Path
import re
import shutil
import subprocess
import tarfile
import tempfile

ROOT = Path(__file__).resolve().parents[1]

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--output', type=Path, default=ROOT / 'target/packages')
    args = parser.parse_args()
    version = re.search(r'^version = "([^"]+)"', (ROOT / 'Cargo.toml').read_text(), re.M)[1]
    name = f'crewsync-{version}-source'
    args.output.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='crewsync-source-') as temporary:
        source = Path(temporary) / name
        source.mkdir()
        tracked = subprocess.check_output(['git', 'ls-files', '-z'], cwd=ROOT).decode().split('\0')
        for relative in filter(None, tracked):
            path = ROOT / relative
            if path.is_symlink() or not path.is_file():
                raise RuntimeError('Fonte versionada inválida: ' + relative)
            target = source / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(path, target)
        config = subprocess.check_output(['cargo', 'vendor', '--locked', '--versioned-dirs', str(source / 'vendor')], cwd=ROOT).decode()
        # Cargo emite o caminho absoluto usado na preparação; tornar relocável.
        config = config.replace(str(source / 'vendor'), 'vendor')
        (source / '.cargo').mkdir(exist_ok=True)
        (source / '.cargo/config.toml').write_text(config)
        (source / 'BUILDING-SOURCE.md').write_text('''# Construir a fonte da release

Este pacote contém o projeto, Cargo.lock e dependências em vendor/.
Com Rust stable e compilador C instalados, execute na raiz:

    cargo build --release --workspace --locked --offline

Para musl, instale também o target Rust e musl-tools correspondentes.
Os executáveis ficam em target/release (ou target/ALVO/release).
LICENSE contém a GPL-3.0-only. Dependências preservam seus avisos originais.
''')
        archive = args.output / (name + '.tar.gz')
        def clean(info):
            if not (info.isfile() or info.isdir()):
                raise RuntimeError('Link ou arquivo especial no pacote de fontes')
            info.uid = info.gid = 0
            info.uname = info.gname = ''
            return info
        with tarfile.open(archive, 'w:gz', format=tarfile.PAX_FORMAT) as tar:
            tar.add(source, arcname=name, filter=clean)
    with archive.open('rb') as data:
        digest = hashlib.file_digest(data, 'sha256').hexdigest()
    archive.with_name(archive.name + '.sha256').write_text(f'{digest}  {archive.name}\n')
    print(archive)

if __name__ == '__main__':
    main()
