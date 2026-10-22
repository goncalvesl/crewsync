#!/usr/bin/env python3
"""Valida um pacote Arch efetivamente produzido por makepkg."""
import argparse
import hashlib
import os
import pathlib
import re
import subprocess

parser=argparse.ArgumentParser()
parser.add_argument('package',type=pathlib.Path)
args=parser.parse_args()
root=pathlib.Path(__file__).resolve().parents[1]
version=re.search(r'^version = "([^"]+)"',(root/'Cargo.toml').read_text(),re.M)[1]
pkgver=version.replace('-alpha.','_alpha.')+'-1'
package=args.package
expected=f'crewsync-{pkgver}-x86_64.pkg.tar.zst'

def require(ok,message):
    if not ok: raise SystemExit(message)

require(package.name==expected,f'Nome de pacote inesperado: {package.name}; esperado {expected}')
checksum=package.with_name(package.name+'.sha256').read_text().split()[0]
require(hashlib.file_digest(package.open('rb'),'sha256').hexdigest()==checksum,'Checksum Arch incorreto')
env={**os.environ,'LC_ALL':'C'}
info=subprocess.check_output(['pacman','-Qip',str(package)],text=True,env=env)
require(re.search(r'^Name\s*:\s*crewsync$',info,re.M) is not None,'Nome Arch incorreto')
require(re.search(r'^Version\s*:\s*'+re.escape(pkgver)+r'$',info,re.M) is not None,'Versão Arch incorreta')
require(re.search(r'^Architecture\s*:\s*x86_64$',info,re.M) is not None,'Arquitetura Arch incorreta')
depends=re.search(r'^Depends On\s*:\s*(.*)$',info,re.M)
require(depends is not None and set(depends.group(1).split())=={'systemd','shadow','util-linux','openssl'},'Dependências Arch inesperadas')
files=subprocess.check_output(['pacman','-Qlp',str(package)],text=True,env=env)
for path in ['/usr/bin/crewsync-server','/usr/bin/crewsync-server-setup','/usr/lib/systemd/system/crewsync-server.service','/usr/share/licenses/crewsync/LICENSE']:
    require(path in files,f'Arquivo Arch ausente: {path}')
print('Pacote Arch x86_64: metadados, layout e SHA-256 OK')
