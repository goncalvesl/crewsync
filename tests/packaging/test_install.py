#!/usr/bin/env python3
"""Instalação por função, destino com espaços e preservação da configuração."""
from pathlib import Path
import os
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]

class Installation(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.package = self.root / 'pacote com espaços'
        self.prefix = self.root / 'destino com espaços'
        for directory in ['bin', 'docs', 'config', 'scripts']:
            (self.package / directory).mkdir(parents=True, exist_ok=True)
        shutil.copy2(ROOT / 'scripts/install.sh', self.package / 'install.sh')
        shutil.copy2(ROOT / 'LICENSE', self.package / 'LICENSE')
        (self.package / 'docs/MANUAL.md').write_text('Manual do usuário')
        for name in ['crewsync-client', 'crewsync-server']:
            path = self.package / 'bin' / name
            path.write_text('#!/bin/sh\nexit 0\n')
            path.chmod(0o755)
        (self.package / 'config/server.example.toml').write_text('exemplo')
        (self.package / 'scripts/pilot-cert.sh').write_text('#!/bin/sh\nexit 0\n')

    def install(self, *args, prefix=None):
        return subprocess.run([str(self.package / 'install.sh'), *args], env={**os.environ, 'PREFIX': str(prefix or self.prefix)}, capture_output=True, text=True, timeout=5)

    def test_client_is_a_single_command_and_preserves_existing_configuration(self):
        config = self.prefix / 'config/session.json'
        config.parent.mkdir(parents=True)
        config.write_text('preservar')
        for _ in range(2):
            result = self.install('--client')
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn(str(self.prefix / 'bin/crewsync-client'), result.stdout)
        self.assertTrue((self.prefix / 'bin/crewsync-client').is_file())
        self.assertFalse((self.prefix / 'bin/crewsync-server').exists())
        self.assertTrue((self.prefix / 'share/crewsync/LICENSE').is_file())
        self.assertTrue((self.prefix / 'share/crewsync/MANUAL.md').is_file())
        self.assertEqual(config.read_text(), 'preservar')

    def test_server_installs_its_helpers_without_the_client(self):
        result = self.install('--server')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue((self.prefix / 'bin/crewsync-server').is_file())
        self.assertFalse((self.prefix / 'bin/crewsync-client').exists())
        self.assertTrue(os.access(self.prefix / 'share/crewsync/pilot-cert.sh', os.X_OK))
        self.assertTrue((self.prefix / 'share/crewsync/server.example.toml').is_file())

    def test_default_still_installs_both_and_incomplete_package_is_refused(self):
        self.assertEqual(self.install().returncode, 0)
        self.assertTrue((self.prefix / 'bin/crewsync-client').is_file())
        self.assertTrue((self.prefix / 'bin/crewsync-server').is_file())
        (self.package / 'bin/crewsync-server').unlink()
        target = self.root / 'incompleto'
        self.assertNotEqual(self.install(prefix=target).returncode, 0)
        self.assertFalse(target.exists())

    def test_invalid_options_and_relative_prefix_do_not_install(self):
        for args in [('--invalid',), ('--client', '--server')]:
            self.assertEqual(self.install(*args).returncode, 2)
            self.assertFalse(self.prefix.exists())
        self.assertEqual(self.install('--client', prefix='relativo').returncode, 2)
        self.assertFalse(self.prefix.exists())
        self.assertEqual(self.install('--help').returncode, 0)
        self.assertFalse(self.prefix.exists())

if __name__ == '__main__':
    unittest.main()
