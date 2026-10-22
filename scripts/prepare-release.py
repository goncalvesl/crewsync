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
                repository=REPOSITORY, targets=TARGETS)

def assets(version):
    return [(target, f'{REPOSITORY}/releases/download/v{version}/crewsync-{version}-{'linux-x86_64' if target.startswith('x86_64') else 'linux-arm64'}.tar.gz') for target in TARGETS]

def markdown(version):
    lines = [f'Versão atual: {version}' + (' (alpha, para testes).' if '-' in version else '.'), '']
    for target, url in assets(version):
        label = 'Linux Intel/AMD (x86_64)' if target.startswith('x86_64') else 'Linux ARM64 (aarch64)'
        lines += [f'- [{label} — baixar pacote]({url}) · [SHA-256]({url}.sha256)']
    lines += ['', f'[Notas da versão]({REPOSITORY}/releases/tag/v{version}) · [Código-fonte e dependências]({REPOSITORY}/releases/download/v{version}/crewsync-{version}-source.tar.gz)']
    return '\n'.join(lines)

def html(version):
    links = []
    for target, url in assets(version):
        label = 'Linux x86_64' if target.startswith('x86_64') else 'Linux ARM64'
        links.append(f'<p><a class="button" href="{url}">Baixar {label} ↓</a> <a href="{url}.sha256">SHA-256</a></p>')
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
