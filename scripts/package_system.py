#!/usr/bin/env python3
"""Constrói os formatos instaláveis do servidor Linux a partir dos binários musl."""
import argparse
import pathlib
import shutil
import subprocess
import tempfile
import re

ROOT = pathlib.Path(__file__).resolve().parents[1]
VERSION = re.search(r'^version = "([^"]+)"', (ROOT / 'Cargo.toml').read_text(), re.M)[1]
DEB_VERSION = VERSION.replace('-alpha.', '~alpha.')
RPM_VERSION = VERSION.replace('-alpha.', '~alpha.')
# GitHub normalizes '~' in asset names to '.', so use that spelling in
# published filenames while preserving the package's embedded version.
PUBLIC_VERSION = VERSION.replace('-alpha.', '.alpha.')

def run(args, **kwargs):
    subprocess.run(args, check=True, **kwargs)

def add_payload(tree, bindir):
    def install(src, dst, mode=0o644):
        target = tree / dst.lstrip('/')
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(src, target)
        target.chmod(mode)
    install(bindir / 'crewsync-server', '/usr/bin/crewsync-server', 0o755)
    install(bindir / 'crewsync-client', '/usr/bin/crewsync-client', 0o755)
    install(ROOT / 'scripts/server-system-setup.sh', '/usr/bin/crewsync-server-setup', 0o755)
    install(ROOT / 'scripts/pilot-cert.sh', '/usr/share/crewsync/pilot-cert.sh', 0o755)
    install(ROOT / 'config/server.example.toml', '/usr/share/crewsync/server.example.toml')
    install(ROOT / 'packaging/system/crewsync-server.service', '/usr/lib/systemd/system/crewsync-server.service')
    install(ROOT / 'LICENSE', '/usr/share/licenses/crewsync/LICENSE')
    (tree / 'etc/crewsync').mkdir(parents=True, exist_ok=True)
    (tree / 'var/lib/crewsync').mkdir(parents=True, exist_ok=True)

def make_deb(tree, out, arch):
    debian_arch = {'x86_64':'amd64', 'aarch64':'arm64'}[arch]
    with tempfile.TemporaryDirectory(prefix='crewsync-deb-') as tmp:
        root = pathlib.Path(tmp)/'root'; shutil.copytree(tree,root)
        deb = root / 'DEBIAN'; deb.mkdir()
        (deb / 'control').write_text(f'''Package: crewsync\nVersion: {DEB_VERSION}-1\nSection: net\nPriority: optional\nArchitecture: {debian_arch}\nDepends: adduser, systemd, util-linux, openssl\nMaintainer: CrewSync maintainers\nDescription: CrewSync Linux folder synchronizer\n Rust server with a dedicated systemd service and client tools.\n''')
        (deb / 'postinst').write_text('''#!/bin/sh\nset -e\nif ! getent group crewsync >/dev/null; then addgroup --system crewsync; fi\nif ! getent passwd crewsync >/dev/null; then adduser --system --ingroup crewsync --home /var/lib/crewsync --no-create-home --disabled-login --shell /usr/sbin/nologin crewsync; fi\nchown crewsync:crewsync /var/lib/crewsync\nchmod 0750 /var/lib/crewsync\nif command -v systemctl >/dev/null; then systemctl daemon-reload || true; systemctl try-restart crewsync-server.service || true; fi\n''')
        (deb / 'postrm').write_text('''#!/bin/sh\nset -e\nif command -v systemctl >/dev/null; then systemctl daemon-reload || true; fi\n# Configuration and synchronized data are intentionally preserved.\n''')
        (deb / 'prerm').write_text('''#!/bin/sh\nset -e\nif [ "$1" = remove ] && command -v systemctl >/dev/null; then systemctl disable --now crewsync-server.service >/dev/null 2>&1 || true; fi\n''')
        for name in ('postinst','postrm','prerm'): (deb / name).chmod(0o755)
        run(['dpkg-deb','--root-owner-group','--build',str(root),str(out)])

def make_rpm(tree, out, arch):
    rpm_arch = {'x86_64':'x86_64', 'aarch64':'aarch64'}[arch]
    with tempfile.TemporaryDirectory(prefix='crewsync-rpm-') as tmp:
        top = pathlib.Path(tmp); (top/'BUILD').mkdir(); (top/'RPMS').mkdir(); (top/'SOURCES').mkdir(); (top/'SPECS').mkdir(); (top/'SRPMS').mkdir()
        spec = (ROOT/'packaging/system/crewsync.spec.in').read_text().replace('@VERSION@',RPM_VERSION).replace('@RPM_ARCH@',rpm_arch)
        (top/'SPECS/crewsync.spec').write_text(spec)
        payload = top/'BUILDROOT'; shutil.copytree(tree,payload)
        # rpmbuild consumes the staged tree with an explicit file list.
        packaged_dirs = ['etc/crewsync', 'var/lib/crewsync', 'usr/share/crewsync', 'usr/share/licenses/crewsync', 'usr/lib/systemd/system']
        entries = ['%dir %attr(0750,root,crewsync) /etc/crewsync', '%dir %attr(0750,crewsync,crewsync) /var/lib/crewsync']
        entries += ['%dir /'+directory for directory in packaged_dirs[2:]]
        for path in sorted(payload.rglob('*')):
            if path.is_file():
                relative='/'+str(path.relative_to(payload))
                mode='0755' if path.stat().st_mode & 0o111 else '0644'
                entries.append(f'%attr({mode},root,root) {relative}')
        with (top/'SPECS/crewsync.spec').open('a') as specfile: specfile.write('\n%files\n'+'\n'.join(entries)+'\n')
        run(['rpmbuild','-bb','--define',f'_topdir {top}','--buildroot',str(payload),str(top/'SPECS/crewsync.spec')])
        produced = next((top/'RPMS'/rpm_arch).glob('*.rpm'))
        shutil.copyfile(produced,out)

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--bin-dir',type=pathlib.Path,required=True)
    parser.add_argument('--arch',choices=['x86_64','aarch64'],required=True)
    parser.add_argument('--output',type=pathlib.Path,default=ROOT/'target/packages')
    args=parser.parse_args(); args.output.mkdir(parents=True,exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='crewsync-system-pkg-') as temp:
        tree=pathlib.Path(temp)/'root'; tree.mkdir(); add_payload(tree,args.bin_dir)
        products=[('deb',f'crewsync_{PUBLIC_VERSION}-1_{"amd64" if args.arch=="x86_64" else "arm64"}.deb',make_deb),('rpm',f'crewsync-{PUBLIC_VERSION}-1.{args.arch}.rpm',make_rpm)]
        for _,filename,builder in products:
            output=args.output/filename; builder(tree,output,args.arch)
            subprocess.run(['sha256sum',output.name],cwd=args.output,check=True,stdout=(args.output/(output.name+'.sha256')).open('w'))
            print(output)

if __name__=='__main__': main()
