"""Proxy Caddy real: TLS na frente, backend HTTP privado e transferências em streaming.

CREWSYNC_CADDY_BIN=/caminho/caddy python3 tests/e2e/proxy.py
Sem ACME/Internet: certificados sintéticos, portas efêmeras e diretórios isolados.
"""
import importlib.util
import json
import os
from pathlib import Path
import socket
import subprocess
import time
import unittest

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('setup_fixture', ROOT / 'tests/packaging/test_setup.py')
fixture = importlib.util.module_from_spec(spec)
spec.loader.exec_module(fixture)


def port():
    with socket.socket() as sock:
        sock.bind(('127.0.0.1', 0))
        return sock.getsockname()[1]


class Proxy(fixture.Setup):
    def test_https_frontend_with_http_backend_and_system_trust(self):
        caddy = os.environ['CREWSYNC_CADDY_BIN']
        prefix = self.root / 'prefix'
        config = self.root / 'backend'
        self.configs.append(config)
        backend_port, frontend_port = port(), port()
        while frontend_port == backend_port:
            frontend_port = port()
        result = self.setup('server', [prefix, config, self.root / 'store', 'localhost', backend_port, 'ana', 'Obras'], mode='2')
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        tls = self.root / 'tls'
        subprocess.run(['sh', str(ROOT / 'scripts/pilot-cert.sh'), str(tls), 'localhost'], check=True, capture_output=True)
        # Validar também o exemplo gerado pelo assistente, sem iniciar ACME.
        subprocess.run([caddy, 'adapt', '--config', str(config / 'Caddyfile'), '--adapter', 'caddyfile'], check=True, capture_output=True)
        proxy_config = self.root / 'proxy'
        proxy_config.mkdir()
        self.configs.append(proxy_config)
        settings = {
            'admin': {'disabled': True},
            'apps': {'http': {'servers': {'test': {
                'listen': [f'127.0.0.1:{frontend_port}'],
                'automatic_https': {'disable': True},
                'tls_connection_policies': [{}],
                'routes': [{'handle': [{'handler': 'reverse_proxy', 'upstreams': [{'dial': f'127.0.0.1:{backend_port}'}]}]}],
            }}}, 'tls': {'certificates': {'load_files': [{'certificate': str(tls / 'server.crt'), 'key': str(tls / 'server.key')}]}}},
        }
        settings_path = proxy_config / 'caddy.json'
        settings_path.write_text(json.dumps(settings))
        with (proxy_config / 'service.log').open('w') as log:
            process = subprocess.Popen([caddy, 'run', '--config', str(settings_path)], stdout=log, stderr=log, env=self.env)
        (proxy_config / 'service.pid').write_text(str(process.pid))
        self.addCleanup(lambda: process.wait(timeout=5) if process.poll() is not None else None)
        self.env['SSL_CERT_FILE'] = str(tls / 'ca.crt')
        empty = self.root / 'empty-ca-dir'
        empty.mkdir()
        self.env['SSL_CERT_DIR'] = str(empty)
        url = f'https://localhost:{frontend_port}'
        admin_key = subprocess.check_output([str(prefix / 'bin/crewsync-server'), 'admin-key', str(config / 'server.toml'), url], text=True).strip()
        deadline = time.monotonic() + 10
        while True:
            response = subprocess.run(['curl', '--noproxy', '*', '--cacert', str(tls / 'ca.crt'), '--connect-timeout', '1', '--max-time', '2', '-sS', url], capture_output=True)
            if response.returncode == 0:
                break
            self.assertIsNone(process.poll(), (proxy_config / 'service.log').read_text())
            self.assertLess(time.monotonic(), deadline, 'Proxy não iniciou')
            time.sleep(0.05)
        admin_spec = importlib.util.spec_from_file_location('admin_proxy_helpers', ROOT / 'tests/e2e/admin.py')
        admin_helpers = importlib.util.module_from_spec(admin_spec)
        admin_spec.loader.exec_module(admin_helpers)
        panel = admin_helpers.BrowserClient(url, tls / 'ca.crt')
        status, page = panel.login(admin_key)
        self.assertEqual(status, 200)
        self.assertIn('Pastas e permissões', page)
        self.assertTrue(panel.cookie.startswith('__Host-crewsync_admin='))
        self.assertEqual(panel.action('/admin/user', user='bruno', enabled='true')[0], 303)
        clients = []
        for index in [1, 2]:
            cfg = self.root / f'client-{index}'
            self.configs.append(cfg)
            folder = self.root / f'files-{index}'
            invite = subprocess.check_output([str(prefix / 'bin/crewsync-server'), 'invite', str(config / 'server.toml'), 'ana', '3600'], text=True).strip()
            result = self.setup('client', [prefix, cfg, folder, url, 'system', f'pc{index}', invite])
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertFalse((cfg / 'ca.crt').exists())
            clients.append(folder / 'Obras')
        # O mesmo proxy recusa certificado não confiável; nenhuma queda para HTTP.
        unrelated = self.root / 'unrelated-tls'
        subprocess.run(['sh', str(ROOT / 'scripts/pilot-cert.sh'), str(unrelated), 'localhost'], check=True, capture_output=True)
        untrusted = dict(self.env, SSL_CERT_FILE=str(unrelated / 'ca.crt'))
        rejected = subprocess.run([str(prefix / 'bin/crewsync-client'), 'shares', str(self.root / 'client-1/session.json')], env=untrusted, capture_output=True)
        self.assertNotEqual(rejected.returncode, 0)
        payload = bytes(range(256)) * 4096
        (clients[0] / 'large.bin').write_bytes(payload)
        destination = clients[1] / 'large.bin'
        deadline = time.monotonic() + 20
        while not destination.exists():
            self.assertLess(time.monotonic(), deadline, 'Transferência via proxy não convergiu')
            time.sleep(0.1)
        self.assertEqual(destination.read_bytes(), payload)
        # Revogação continua sendo aplicada pelo servidor atrás do proxy.
        session = json.loads((self.root / 'client-2/session.json').read_text())
        subprocess.run([str(prefix / 'bin/crewsync-server'), 'revoke', str(config / 'server.toml'), session['device_id']], check=True, capture_output=True)
        denied = subprocess.run([str(prefix / 'bin/crewsync-client'), 'shares', str(self.root / 'client-2/session.json')], env=self.env, capture_output=True)
        self.assertNotEqual(denied.returncode, 0)


if __name__ == '__main__':
    if not os.environ.get('CREWSYNC_CADDY_BIN'):
        raise SystemExit('Informe CREWSYNC_CADDY_BIN para testar o proxy real.')
    result = unittest.TextTestRunner(verbosity=2).run(unittest.TestSuite([Proxy('test_https_frontend_with_http_backend_and_system_trust')]))
    raise SystemExit(not result.wasSuccessful())
