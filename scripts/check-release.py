#!/usr/bin/env python3
"""Publicação depende de versão explícita e aceite físico registrado pelo mantenedor."""
import json
import pathlib
import re
import sys
root = pathlib.Path(__file__).resolve().parents[1]
version = re.search(r'^version = "([^"]+)"', (root / 'Cargo.toml').read_text(), re.M)[1]
if sys.argv[1] != 'v' + version or version.endswith('-dev'):
    sys.exit('Tag deve corresponder a uma versão publicável no Cargo.toml')
acceptance = json.loads((root / 'docs/reports/physical-acceptance.json').read_text())
if not acceptance['extended_matrix_accepted']:
    sys.exit('Aceite físico ampliado ainda não registrado; pacote de CI disponível, release bloqueada')
if not (root / 'LICENSE').exists():
    sys.exit('Licença pendente')
print('Versão, licença e aceite físico: OK')
