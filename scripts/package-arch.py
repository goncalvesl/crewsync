#!/usr/bin/env python3
"""Prepara um PKGBUILD e payload musl para makepkg no Arch Linux x86_64."""
import argparse
import hashlib
import pathlib
import shutil
import subprocess
import tarfile
import tempfile
import re
import sys

ROOT=pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'scripts'))
from package_system import add_payload

VERSION=re.search(r'^version = "([^"]+)"',(ROOT/'Cargo.toml').read_text(),re.M)[1]
PKGVER=VERSION.replace('-alpha.','_alpha.')

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--bin-dir',type=pathlib.Path,required=True)
    parser.add_argument('--output',type=pathlib.Path,default=ROOT/'target/packages/arch-build')
    args=parser.parse_args(); args.output.mkdir(parents=True,exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='crewsync-arch-payload-') as tmp:
        tree=pathlib.Path(tmp)/'payload'; tree.mkdir(); add_payload(tree,args.bin_dir)
        payload=args.output/'payload.tar.gz'
        with tarfile.open(payload,'w:gz',format=tarfile.PAX_FORMAT) as archive:
            for path in sorted(tree.rglob('*')):
                info=archive.gettarinfo(str(path),arcname=path.relative_to(tree).as_posix())
                info.uid=info.gid=0; info.uname=info.gname='root'
                if path.is_file():
                    with path.open('rb') as source: archive.addfile(info,source)
                else: archive.addfile(info)
    digest=hashlib.file_digest(payload.open('rb'),'sha256').hexdigest()
    template=(ROOT/'packaging/arch/PKGBUILD.in').read_text()
    (args.output/'PKGBUILD').write_text(template.replace('@PKGVER@',PKGVER).replace('@SHA256@',digest))
    shutil.copyfile(ROOT/'packaging/arch/crewsync.install',args.output/'crewsync.install')
    print(args.output)

if __name__=='__main__': main()
