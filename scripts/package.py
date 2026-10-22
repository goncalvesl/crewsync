#!/usr/bin/env python3
"""Empacota somente arquivos permitidos; nunca credenciais nem dados de trabalho."""
import argparse
import hashlib
import io
import json
import pathlib
import re
import tarfile
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[1]

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--bin-dir', type=pathlib.Path, default=ROOT / 'target/release')
    parser.add_argument('--target', required=True, choices=['x86_64-unknown-linux-musl', 'aarch64-unknown-linux-musl', 'x86_64-unknown-linux-gnu'])
    parser.add_argument('--output', type=pathlib.Path, default=ROOT / 'target/packages')
    args = parser.parse_args()
    version = re.search(r'^version = "([^"]+)"', (ROOT / 'Cargo.toml').read_text(), re.M)[1]
    name = f'crewsync-{version}-{args.target}'
    args.output.mkdir(parents=True, exist_ok=True)
    archive = args.output / f'{name}.tar.gz'
    files = [('bin/' + n, args.bin_dir / n) for n in ['crewsync-client', 'crewsync-server']]
    files += [('README.md', ROOT / 'README.md'), ('install.sh', ROOT / 'scripts/install.sh'), ('scripts/pilot-cert.sh', ROOT / 'scripts/pilot-cert.sh')]
    files += [('config/' + name, ROOT / 'config' / name) for name in ['README.md', 'server.example.toml', 'systemd/crewsync-client.service', 'systemd/crewsync-server.service']]
    files += [(str(p.relative_to(ROOT)), p) for p in (ROOT / 'docs').rglob('*.md')]
    files += [(str(p.relative_to(ROOT)), p) for p in (ROOT / 'docs/reports').glob('*.json')]
    files += [('LICENSE', ROOT / 'LICENSE')] if (ROOT / 'LICENSE').exists() else [('LICENSE-STATUS.md', ROOT / 'LICENSE-STATUS.md')]
    with tarfile.open(archive, 'w:gz', format=tarfile.PAX_FORMAT) as tar:
        for relative, source in sorted(files):
            if source.is_symlink() or not source.is_file():
                raise RuntimeError('Arquivo de pacote inválido: ' + str(source))
            info = tar.gettarinfo(str(source), arcname=name + '/' + relative)
            info.uid = info.gid = 0
            info.uname = info.gname = ''
            info.mode = 0o755 if relative.startswith('bin/') or relative.endswith('.sh') else 0o644
            with source.open('rb') as data:
                tar.addfile(info, data)
        # Manter avisos completos das dependências, inclusive alternativas SPDX.
        metadata_graph = json.loads(subprocess.check_output(['cargo', 'metadata', '--format-version', '1', '--locked', '--offline', '--filter-platform', args.target], cwd=ROOT))
        notices = []
        for package in sorted(metadata_graph['packages'], key=lambda p: (p['name'], p['version'])):
            if package['source'] is None:
                continue
            directory = pathlib.Path(package['manifest_path']).parent
            notices.append(f"\n{'=' * 72}\n{package['name']} {package['version']} — {package.get('license')}\n")
            license_files = {p for p in directory.iterdir() if p.is_file() and p.name.upper().startswith(('LICENSE', 'LICENCE', 'COPYING', 'NOTICE'))}
            if package.get('license_file'):
                license_files.add(directory / package['license_file'])
            for license_file in sorted(license_files):
                notices.append(license_file.name + '\n' + license_file.read_text(errors='replace'))
        data = '\n'.join(notices).encode()
        info = tarfile.TarInfo(name + '/THIRD-PARTY-NOTICES.txt')
        info.mode = 0o644
        info.size = len(data)
        tar.addfile(info, io.BytesIO(data))
        metadata = json.dumps(dict(version=version, target=args.target, kernel_minimum='5.6', runtime='Rust nativo; sem Python/Cargo; /proc e filesystem local necessários'), ensure_ascii=False).encode()
        info = tarfile.TarInfo(name + '/package.json')
        info.mode = 0o644
        info.size = len(metadata)
        tar.addfile(info, io.BytesIO(metadata))
    digest = hashlib.file_digest(archive.open('rb'), 'sha256').hexdigest()
    archive.with_name(archive.name + '.sha256').write_text(f'{digest}  {archive.name}\n')
    print(archive)

if __name__ == '__main__':
    main()
