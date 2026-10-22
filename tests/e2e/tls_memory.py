"""Valida TLS real em MemoryBIO, sem sockets ou conexões externas."""
import pathlib
import ssl
import subprocess
import tempfile
import unittest


class TLS(unittest.TestCase):
    host = "192.0.2.10"
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory()
        root = pathlib.Path(cls.temp.name) / 'tls'
        generator = pathlib.Path(__file__).resolve().parents[2] / 'scripts/pilot-cert.sh'
        subprocess.run([str(generator), str(root), cls.host], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        cls.cert = str(root / 'ca.crt')
        cls.server_cert = str(root / 'server.crt')
        cls.key = str(root / 'server.key')
        assert (root / 'server.key').stat().st_mode & 0o077 == 0
    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()
    def handshake(self, host, trusted):
        client_context = ssl.create_default_context(cafile=self.cert) if trusted else ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
        server_context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        server_context.minimum_version = ssl.TLSVersion.TLSv1_2
        server_context.load_cert_chain(self.server_cert, self.key)
        ci, co, si, so = (ssl.MemoryBIO() for _ in range(4))
        client = client_context.wrap_bio(ci, co, server_hostname=host)
        server = server_context.wrap_bio(si, so, server_side=True)
        done = [False, False]
        for _ in range(100):
            for index, peer in enumerate((client, server)):
                if not done[index]:
                    try:
                        peer.do_handshake()
                        done[index] = True
                    except ssl.SSLWantReadError:
                        pass
            for source, target in ((co, si), (so, ci)):
                data = source.read()
                if data:
                    target.write(data)
            if all(done):
                client.write(b'authenticated data')
                si.write(co.read())
                self.assertEqual(server.read(), b'authenticated data')
                return
        self.fail('handshake não concluiu no limite')
    def test_trusted_certificate_and_matching_host(self):
        self.handshake(self.host, True)
    def test_hostname_mismatch_is_rejected(self):
        with self.assertRaises(ssl.SSLCertVerificationError):
            self.handshake('another-host.invalid', True)
    def test_untrusted_certificate_is_rejected(self):
        with self.assertRaises(ssl.SSLCertVerificationError):
            self.handshake('localhost', False)

class DNS(TLS):
    host = 'sync.example.test'

if __name__ == '__main__':
    unittest.main()
