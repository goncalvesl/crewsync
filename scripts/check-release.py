#!/usr/bin/env python3
"""Alpha autorizada para testes; versão estável exige aceite físico ampliado."""
import importlib.util
import json
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]

def validate(root, tag):
    version = re.search(r'^version = "([^"]+)"', (root / 'Cargo.toml').read_text(), re.M)[1]
    if tag != 'v' + version or re.fullmatch(r'\d+\.\d+\.\d+(?:-alpha\.[1-9][0-9]*)?', version) is None:
        raise ValueError('Tag deve corresponder à versão publicável no Cargo.toml')
    if not (root / 'LICENSE').exists():
        raise ValueError('Licença ausente')
    acceptance = json.loads((root / 'docs/reports/physical-acceptance.json').read_text())
    if '-alpha.' not in version and not acceptance['extended_matrix_accepted']:
        raise ValueError('Versão estável exige aceite físico ampliado')
    return version

if __name__ == '__main__':
    try:
        version = validate(ROOT, sys.argv[1])
        spec = importlib.util.spec_from_file_location('prepare_release', ROOT / 'scripts/prepare-release.py')
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        module.prepare(ROOT, version, check=True)
    except (ValueError, IndexError) as error:
        sys.exit(str(error))
    print('Versão, licença e downloads conferidos; alpha não declara aceite físico.')
