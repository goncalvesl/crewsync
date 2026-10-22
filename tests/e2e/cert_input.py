#!/usr/bin/env python3
"""Endereços inválidos não criam diretórios ou certificados, inclusive entrada multilinha."""
import pathlib
import subprocess
import tempfile
import unittest

SCRIPT = pathlib.Path(__file__).resolve().parents[2] / 'scripts/pilot-cert.sh'
class CertificateInput(unittest.TestCase):
    def test_invalid_addresses_do_not_create_output(self):
        with tempfile.TemporaryDirectory() as root:
            for index, address in enumerate(['', '1.2.3', '256.1.2.3', '01.2.3.4', '1.2.3.4\n5.6.7.8', '1.2.3.4;echo unsafe', 'sync', 'https://sync.example.com', 'sync.example.com:8443', '*.example.com', '-sync.example.com', 'sync-.example.com', 'sync..example.com', 'sync.example.com.', 'sync.example.com,DNS:evil.example', 'sync.example.com\nDNS:evil.example', 'a' * 64 + '.example.com', ('a.' * 127) + 'com', 'sync_1.example.com', '2001:db8::1']):
                target = pathlib.Path(root) / str(index)
                result = subprocess.run([str(SCRIPT), str(target), address], capture_output=True, timeout=5)
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse(target.exists())

    def test_validation_mode_accepts_domains_and_ips_without_creating_files(self):
        for host in ['sync.example.com', 'SYNC.Example.COM', 'xn--exemplo-9za.com', 'localhost', '192.0.2.10']:
            result = subprocess.run([str(SCRIPT), '--check-address', host], capture_output=True, timeout=5)
            self.assertEqual(result.returncode, 0, result.stderr)

if __name__ == '__main__':
    unittest.main()
