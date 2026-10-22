#!/usr/bin/env python3
"""IPs inválidos não criam diretórios ou certificados, inclusive entrada multilinha."""
import pathlib
import subprocess
import tempfile
import unittest

SCRIPT = pathlib.Path(__file__).resolve().parents[2] / 'scripts/pilot-cert.sh'
class CertificateInput(unittest.TestCase):
    def test_invalid_addresses_do_not_create_output(self):
        with tempfile.TemporaryDirectory() as root:
            for index, address in enumerate(['', '1.2.3', '256.1.2.3', '01.2.3.4', '1.2.3.4\n5.6.7.8', '1.2.3.4;echo unsafe']):
                target = pathlib.Path(root) / str(index)
                result = subprocess.run([str(SCRIPT), str(target), address], capture_output=True, timeout=5)
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse(target.exists())

if __name__ == '__main__':
    unittest.main()
