#!/usr/bin/env python3
"""Confere metadados, checksums e layout de pacotes Debian/Fedora."""
import argparse
import hashlib
import pathlib
import re
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[1]
VERSION = re.search(r'^version = "([^"]+)"', (ROOT/'Cargo.toml').read_text(), re.M)[1]
DEB_VERSION = VERSION.replace('-alpha.', '~alpha.') + '-1'
RPM_VERSION = VERSION.replace('-alpha.', '~alpha.')
PUBLIC_VERSION = VERSION.replace('-alpha.', '.alpha.')

def output(args):
    return subprocess.check_output(args, text=True).strip()

def require(condition, message):
    if not condition:
        raise SystemExit(message)

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--directory',type=pathlib.Path,default=ROOT/'target/packages')
    parser.add_argument('--arch',choices=['x86_64','aarch64'],required=True)
    args=parser.parse_args()
    service=(ROOT/'packaging/system/crewsync-server.service').read_text()
    setup=(ROOT/'scripts/server-system-setup.sh').read_text()
    require('User=crewsync' in service and 'Group=crewsync' in service,'O unit do servidor deve usar a conta dedicada')
    require('ReadWritePaths=/var/lib/crewsync /etc/crewsync' in service,'O sandbox deve permitir atualização atômica da configuração pelo painel')
    require('chmod 2770 "$etc_dir"' in setup and 'chmod 660 "$etc_dir/server.toml"' in setup,'Permissões de configuração incompatíveis com o painel')
    require('runuser -u crewsync' in setup,'Convites e chave administrativa devem pertencer à conta de serviço')
    deb_arch='amd64' if args.arch=='x86_64' else 'arm64'
    expected={
        'deb':args.directory/f'crewsync_{PUBLIC_VERSION}-1_{deb_arch}.deb',
        'rpm':args.directory/f'crewsync-{PUBLIC_VERSION}-1.{args.arch}.rpm',
    }
    for fmt,path in expected.items():
        require(path.is_file(),f'Pacote {fmt} ausente: {path.name}')
        checksum=path.with_name(path.name+'.sha256')
        require(checksum.is_file(),f'Checksum ausente: {checksum.name}')
        digest=hashlib.file_digest(path.open('rb'),'sha256').hexdigest()
        require(digest==checksum.read_text().split()[0],f'Checksum inválido: {path.name}')
    deb=expected['deb']
    meta=output(['dpkg-deb','-f',str(deb),'Package','Version','Architecture']).splitlines()
    meta=[line.split(': ',1)[1] for line in meta]
    require(meta==['crewsync',DEB_VERSION,'amd64' if args.arch=='x86_64' else 'arm64'],f'Metadados Debian inesperados: {meta}')
    deb_depends=output(['dpkg-deb','-f',str(deb),'Depends'])
    require(all(item in deb_depends for item in ['adduser','systemd','util-linux','openssl']),'Dependências Debian incompletas')
    deb_files=output(['dpkg-deb','-c',str(deb)])
    require('usr/bin/crewsync-server' in deb_files and 'usr/lib/systemd/system/crewsync-server.service' in deb_files,'Layout Debian incompleto')
    require('usr/bin/crewsync-server-setup' in deb_files and 'var/lib/crewsync' in deb_files,'Provisionamento Debian incompleto')
    rpm=expected['rpm']
    rpm_meta=output(['rpm','-qp','--qf','%{NAME}\n%{VERSION}\n%{RELEASE}\n%{ARCH}',str(rpm)]).splitlines()
    require(rpm_meta==['crewsync',RPM_VERSION,'1',args.arch],f'Metadados RPM inesperados: {rpm_meta}')
    rpm_files=output(['rpm','-qpl',str(rpm)])
    require('/usr/bin/crewsync-server' in rpm_files and '/usr/lib/systemd/system/crewsync-server.service' in rpm_files,'Layout RPM incompleto')
    rpm_requires=output(['rpm','-qp','--requires',str(rpm)])
    require('openssl' in rpm_requires and 'util-linux' in rpm_requires,'Dependências RPM incompletas')
    print(f'Pacotes Debian e Fedora {args.arch}: metadados, layout e SHA-256 OK')

if __name__=='__main__': main()
