#!/usr/bin/env python3
"""Prepara versão e downloads; --check detecta documentação desatualizada."""
import argparse
import json
from pathlib import Path
import re
import tomllib

ROOT = Path(__file__).resolve().parents[1]
TARGETS = ['x86_64-unknown-linux-musl', 'aarch64-unknown-linux-musl']
REPOSITORY = 'https://github.com/goncalvesl/crewsync'
PATTERN = r'(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(?:-alpha\.([1-9][0-9]*))?'

def validate_version(version):
    if re.fullmatch(PATTERN, version) is None:
        raise ValueError('Use MAJOR.MINOR.PATCH ou MAJOR.MINOR.PATCH-alpha.N')
    return version

def metadata(version):
    validate_version(version)
    return dict(version=version, tag='v' + version, prerelease='-' in version,
                repository=REPOSITORY, targets=TARGETS,
                distributions=['arch', 'debian', 'fedora', 'tar.gz'],
                distribution_targets={'arch':['x86_64'], 'debian':['x86_64','aarch64'],
                                      'fedora':['x86_64','aarch64'], 'tar.gz':['x86_64','aarch64']})

def assets(version):
    return [(target, f'{REPOSITORY}/releases/download/v{version}/crewsync-{version}-{'linux-x86_64' if target.startswith('x86_64') else 'linux-arm64'}.tar.gz') for target in TARGETS]

def markdown(version):
    lines = [f'Versão atual: {version}' + (' (alpha, para testes).' if '-' in version else '.'), '']
    root = f'{REPOSITORY}/releases/download/v{version}/'
    public_native = version.replace('-alpha.', '.alpha.')
    variants = [
        ('Arch Linux', [('x86_64', f'crewsync-{version.replace("-alpha.", "_alpha.")}-1-x86_64.pkg.tar.zst')]),
        ('Debian/Ubuntu', [('amd64', f'crewsync_{public_native}-1_amd64.deb'), ('ARM64', f'crewsync_{public_native}-1_arm64.deb')]),
        ('Fedora', [('x86_64', f'crewsync-{public_native}-1.x86_64.rpm'), ('ARM64', f'crewsync-{public_native}-1.aarch64.rpm')]),
        ('Tar.gz independente', [('x86_64', f'crewsync-{version}-linux-x86_64.tar.gz'), ('ARM64', f'crewsync-{version}-linux-arm64.tar.gz')]),
    ]
    for label, packages in variants:
        downloads = []
        for arch, filename in packages:
            url = root + filename
            downloads.append(f'[{arch}]({url}) · [SHA-256]({url}.sha256)')
        lines.append(f'- **{label}:** ' + ' · '.join(downloads))
    lines += ['', f'[Notas da versão]({REPOSITORY}/releases/tag/v{version}) · [Código-fonte e dependências]({REPOSITORY}/releases/download/v{version}/crewsync-{version}-source.tar.gz)']
    return '\n'.join(lines)

def html(version):
    root = f'{REPOSITORY}/releases/download/v{version}/'
    public_native = version.replace('-alpha.', '.alpha.')
    links = []
    variants = [
        ('Arch Linux', [('x86_64', f'crewsync-{version.replace("-alpha.", "_alpha.")}-1-x86_64.pkg.tar.zst')]),
        ('Debian/Ubuntu', [('amd64', f'crewsync_{public_native}-1_amd64.deb'), ('ARM64', f'crewsync_{public_native}-1_arm64.deb')]),
        ('Fedora', [('x86_64', f'crewsync-{public_native}-1.x86_64.rpm'), ('ARM64', f'crewsync-{public_native}-1.aarch64.rpm')]),
        ('Tar.gz independente', [('x86_64', f'crewsync-{version}-linux-x86_64.tar.gz'), ('ARM64', f'crewsync-{version}-linux-arm64.tar.gz')]),
    ]
    for label, packages in variants:
        options = ' · '.join(f'<a href="{root+filename}">{arch}</a> · <a href="{root+filename}.sha256">SHA-256</a>' for arch, filename in packages)
        links.append(f'<p><strong>{label}:</strong> {options}</p>')
    return f'<span class="status">{version}' + (' · alpha' if '-' in version else '') + '</span><h3 style="margin-top:24px">Baixe para seu computador</h3>' + ''.join(links) + f'<a href="manual.html#instalar">Como instalar →</a> · <a href="{REPOSITORY}/releases/tag/v{version}">Notas da versão</a>'

def replace_block(text, name, content):
    start, end = f'<!-- {name}:start -->', f'<!-- {name}:end -->'
    if text.count(start) != 1 or text.count(end) != 1:
        raise ValueError(f'Bloco {name} ausente ou duplicado')
    before, tail = text.split(start)
    _, after = tail.split(end)
    return before + start + '\n' + content + '\n' + end + after

def prepare(root, version, check=False):
    expected = {
        root / 'docs/release.json': json.dumps(metadata(version), indent=2) + '\n',
    }
    for filename in ['README.md', 'docs/MANUAL.md']:
        path = root / filename
        expected[path] = replace_block(path.read_text(), 'downloads', markdown(version))
    path = root / 'site/index.html'
    expected[path] = replace_block(path.read_text(), 'downloads', html(version))
    path = root / 'Cargo.toml'
    expected[path] = re.sub(r'^version = "[^"]+"', f'version = "{version}"', path.read_text(), count=1, flags=re.M)
    for name in ['client', 'server']:
        path = root / f'crates/{name}/Cargo.toml'
        expected[path] = re.sub(r'crewsync-protocol = \{ version = "[^"]+"', f'crewsync-protocol = {{ version = "{version}"', path.read_text())
    path = root / 'Cargo.lock'
    lock, replacements = re.subn(r'(name = "crewsync-(?:client|server|protocol)"\nversion = ")[^"]+("\n)',
        lambda match: match[1] + version + match[2], path.read_text())
    if replacements != 3:
        raise ValueError('Cargo.lock deve conter as três crates locais, sem duplicatas')
    expected[path] = lock
    changed = [str(path.relative_to(root)) for path, content in expected.items() if not path.exists() or path.read_text() != content]
    if check:
        if changed:
            raise ValueError('Versão/downloads desatualizados: ' + ', '.join(changed))
    else:
        for path, content in expected.items():
            path.write_text(content)
    return changed

def check_lock(root, version):
    lock = tomllib.loads((root / 'Cargo.lock').read_text())
    packages = {p['name']: p['version'] for p in lock['package'] if p['name'] in ['crewsync-client', 'crewsync-server', 'crewsync-protocol']}
    if len(packages) != 3 or any(value != version for value in packages.values()):
        raise ValueError('Cargo.lock desatualizado para a versão da release')

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('version', nargs='?')
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    version = args.version or json.loads((ROOT / 'docs/release.json').read_text())['version']
    try:
        prepare(ROOT, validate_version(version), args.check)
    except ValueError as error:
        parser.exit(1, str(error) + '\n')
    try:
        check_lock(ROOT, version)
    except ValueError as error:
        parser.exit(1, str(error) + '\n')
    print(('Conferido: ' if args.check else 'Preparado: ') + version)

if __name__ == '__main__':
    main()
