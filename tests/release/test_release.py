import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
def module(name, filename):
    spec = importlib.util.spec_from_file_location(name, ROOT / 'scripts' / filename)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result
prepare = module('prepare_release', 'prepare-release.py')
gate = module('check_release', 'check-release.py')

class Release(unittest.TestCase):
    def fixture(self, root):
        for name in ['docs/reports', 'site', 'crates/client', 'crates/server']:
            (root / name).mkdir(parents=True, exist_ok=True)
        for name in ['README.md', 'docs/MANUAL.md', 'site/index.html']:
            (root / name).write_text('before\n<!-- downloads:start -->\nold\n<!-- downloads:end -->\nafter\n')
        (root / 'Cargo.toml').write_text('[workspace.package]\nversion = "0.1.0-dev"\n')
        for name in ['client', 'server']:
            (root / f'crates/{name}/Cargo.toml').write_text('crewsync-protocol = { version = "0.1.0-dev", path = "../protocol" }\n')
        lock = '\n'.join(f'[[package]]\nname = "{name}"\nversion = "0.1.0-dev"\n' for name in ['crewsync-client', 'crewsync-server', 'crewsync-protocol'])
        (root / 'Cargo.lock').write_text(lock)
        (root / 'LICENSE').write_text('GPL')
        (root / 'docs/reports/physical-acceptance.json').write_text(json.dumps({'extended_matrix_accepted': False}))

    def test_alpha_updates_all_architectures_and_detects_stale_downloads(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.fixture(root)
            prepare.prepare(root, '0.1.0-alpha.1')
            prepare.prepare(root, '0.1.0-alpha.1', check=True)
            self.assertEqual(gate.validate(root, 'v0.1.0-alpha.1'), '0.1.0-alpha.1')
            old = (root / 'docs/MANUAL.md').read_text()
            self.assertIn('linux-x86_64.tar.gz', old)
            self.assertIn('linux-arm64.tar.gz.sha256', old)
            self.assertIn('source.tar.gz', old)
            prepare.prepare(root, '0.1.0-alpha.2')
            self.assertNotIn('alpha.1', (root / 'README.md').read_text())
            self.assertIn('before', (root / 'README.md').read_text())
            (root / 'docs/MANUAL.md').write_text(old)
            with self.assertRaisesRegex(ValueError, 'desatualizados'):
                prepare.prepare(root, '0.1.0-alpha.2', check=True)

    def test_stable_requires_physical_acceptance_and_tag_license_must_match(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.fixture(root)
            prepare.prepare(root, '0.1.0')
            with self.assertRaisesRegex(ValueError, 'aceite físico'):
                gate.validate(root, 'v0.1.0')
            (root / 'docs/reports/physical-acceptance.json').write_text(json.dumps({'extended_matrix_accepted': True}))
            self.assertEqual(gate.validate(root, 'v0.1.0'), '0.1.0')
            with self.assertRaises(ValueError):
                gate.validate(root, 'v0.2.0')
            (root / 'LICENSE').unlink()
            with self.assertRaisesRegex(ValueError, 'Licença'):
                gate.validate(root, 'v0.1.0')

    def test_invalid_versions_and_missing_markers_do_not_replace_content(self):
        for version in ['0.1.0-dev', 'v0.1.0', '0.1', '0.1.0-alpha.0', '0.01.0', '../bad', '1.2.3;echo']:
            with self.assertRaises(ValueError):
                prepare.validate_version(version)
        with self.assertRaises(ValueError):
            prepare.replace_block('unrelated content', 'downloads', 'new')

    def test_stale_local_crates_in_lock_are_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            packages = '\n'.join(f'[[package]]\nname = "{name}"\nversion = "0.1.0-dev"' for name in ['crewsync-client', 'crewsync-server', 'crewsync-protocol'])
            (root / 'Cargo.lock').write_text(packages)
            with self.assertRaisesRegex(ValueError, 'Cargo.lock desatualizado'):
                prepare.check_lock(root, '0.1.0-alpha.1')
            (root / 'Cargo.lock').write_text(packages.replace('0.1.0-dev', '0.1.0-alpha.1'))
            prepare.check_lock(root, '0.1.0-alpha.1')
