"""Painel HTTP/TLS real. PLAYWRIGHT_MODULE habilita também o navegador."""
import html
import http.client
import importlib.util
import json
import os
from pathlib import Path
import re
import socket
import ssl
import subprocess
import unittest
from urllib.parse import urlencode, urlsplit

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('admin_setup_fixture', ROOT / 'tests/packaging/test_setup.py')
fixture = importlib.util.module_from_spec(spec)
spec.loader.exec_module(fixture)


def hidden(body, name):
    match = re.search(r'name="' + re.escape(name) + r'" value="([^"]*)"', body)
    if not match:
        raise AssertionError('Campo ausente: ' + name)
    return html.unescape(match.group(1))


class BrowserClient:
    def __init__(self, url, ca=None):
        self.url = url
        self.parsed = urlsplit(url)
        self.cookie = ''
        self.context = ssl.create_default_context(cafile=str(ca)) if ca else None
        self.last_headers = {}

    def request(self, method, path='/', fields=None, origin=None):
        connection = (http.client.HTTPSConnection(self.parsed.hostname, self.parsed.port, context=self.context, timeout=10)
                      if self.parsed.scheme == 'https' else http.client.HTTPConnection(self.parsed.hostname, self.parsed.port, timeout=10))
        headers = {'Cookie': self.cookie}
        body = None
        if method == 'POST':
            body = urlencode(fields or {})
            headers.update({'Origin': self.url if origin is None else origin, 'Content-Type': 'application/x-www-form-urlencoded'})
        try:
            connection.request(method, path, body=body, headers=headers)
            response = connection.getresponse()
            self.last_headers = dict(response.getheaders())
            if response.getheader('Set-Cookie'):
                self.cookie = response.getheader('Set-Cookie').split(';')[0]
            return response.status, response.read().decode()
        finally:
            connection.close()

    def login(self, key):
        status, body = self.request('GET')
        assert status == 200 and 'Entrar no painel' in body
        guest_cookie = self.cookie
        status, _ = self.request('POST', '/admin/login', {'csrf': hidden(body, 'csrf'), 'key': key})
        assert status == 303
        assert self.cookie != guest_cookie
        return self.request('GET')

    def action(self, path, **fields):
        status, page = self.request('GET')
        assert status == 200
        fields.update(csrf=hidden(page, 'csrf'), revision=hidden(page, 'revision'))
        return self.request('POST', path, fields)


class Admin(fixture.Setup):
    def start_admin(self, mode):
        prefix = self.root / 'prefix'
        config = self.root / 'server'
        self.configs.append(config)
        with socket.socket() as sock:
            sock.bind(('127.0.0.1', 0))
            port = sock.getsockname()[1]
        result = self.setup('server', [prefix, config, self.root / 'store', '127.0.0.1', port], mode=mode, provision=False)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        key = re.search(r'Chave administrativa \(guarde em local seguro\):\s*\n([0-9a-f]{64})', result.stdout).group(1)
        url = f'{"https" if mode == "1" else "http"}://127.0.0.1:{port}'
        return config, key, url, config / 'tls/ca.crt' if mode == '1' else None

    def exercise(self, mode):
        config, key, url, ca = self.start_admin(mode)
        client = BrowserClient(url, ca)
        status, login = client.request('GET')
        self.assertEqual(status, 200)
        initial = (config / 'server.toml').read_bytes()
        status, _ = client.request('POST', '/admin/user', {'csrf': hidden(login, 'csrf'), 'user': 'intruso', 'enabled': 'true'})
        self.assertEqual(status, 403)
        self.assertEqual((config / 'server.toml').read_bytes(), initial)
        status, _ = client.request('POST', '/admin/login', {'csrf': hidden(login, 'csrf'), 'key': '0' * 64})
        self.assertEqual(status, 401)
        status, dashboard = client.login(key)
        self.assertEqual(status, 200)
        self.assertNotIn(key, dashboard)
        self.assertEqual(client.last_headers['Cache-Control'], 'no-store')
        self.assertEqual(client.action('/admin/user', user='bruno', enabled='true')[0], 303)
        self.assertEqual(client.action('/admin/share', share='Projetos & Segurança')[0], 303)
        self.assertEqual(client.action('/admin/permission', share='Projetos & Segurança', user='bruno', access='write')[0], 303)
        status, invitation = client.action('/admin/invite', user='bruno', ttl='3600')
        self.assertEqual(status, 200)
        invite = re.search(r'id="invite-code" readonly value="([^"]+)"', invitation).group(1)
        session = self.root / 'device-session.json'
        activated = subprocess.run([str(self.package / 'bin/crewsync-client'), 'activate', url, str(ca) if ca else 'insecure-http', str(session), "PC <script>alert(1)</script>"], input=invite + '\n', capture_output=True, text=True)
        self.assertEqual(activated.returncode, 0, activated.stderr)
        local = self.root / 'local'
        local.mkdir()
        synced = subprocess.run([str(self.package / 'bin/crewsync-client'), 'sync-root', str(session), str(local)], capture_output=True, text=True)
        self.assertEqual(synced.returncode, 0, synced.stderr)
        self.assertTrue((local / 'Projetos & Segurança').is_dir())
        self.assertFalse((local / 'Obras').exists())
        status, dashboard = client.request('GET', '/?user=bruno')
        self.assertEqual(status, 200)
        self.assertIn('PC &lt;script&gt;', dashboard)
        self.assertNotIn('<script>', dashboard)
        credential = json.loads(session.read_text())
        self.assertNotIn(credential['token'], dashboard)
        revision = hidden(dashboard, 'revision')
        csrf = hidden(dashboard, 'csrf')
        status, _ = client.request('POST', '/admin/user', {'csrf': csrf, 'revision': revision, 'user': 'atacante', 'enabled': 'true'}, origin='https://attacker.invalid')
        self.assertEqual(status, 403)
        self.assertEqual(client.action('/admin/revoke', device=credential['device_id'])[0], 303)
        denied = subprocess.run([str(self.package / 'bin/crewsync-client'), 'shares', str(session)], capture_output=True)
        self.assertNotEqual(denied.returncode, 0)
        self.assertTrue((local / 'Projetos & Segurança').is_dir())
        self.assertEqual(client.action('/admin/user', user='bruno', enabled='false')[0], 303)
        status, _ = client.request('POST', '/admin/user', {'csrf': csrf, 'revision': revision, 'user': 'sobrescrever', 'enabled': 'true'})
        self.assertEqual(status, 409)
        status, dashboard = client.request('GET')
        self.assertNotIn('sobrescrever', dashboard)
        cookie = client.cookie
        status, _ = client.request('POST', '/admin/logout', {'csrf': hidden(dashboard, 'csrf')})
        self.assertEqual(status, 303)
        client.cookie = cookie
        self.assertEqual(client.request('POST', '/admin/share', {'csrf': csrf, 'revision': revision, 'share': 'Intruso'})[0], 403)

    def test_http_interface_changes_reach_real_client(self):
        self.exercise('3')

    def test_tls_interface_changes_reach_real_client(self):
        self.exercise('1')

    def test_browser_workflow(self):
        config, key, url, _ = self.start_admin('3')
        settings = self.root / 'browser.json'
        settings.write_text(json.dumps({'key': key, 'url': url, 'client': str(self.package / 'bin/crewsync-client'), 'session': str(self.root / 'browser-session.json')}))
        settings.chmod(0o600)
        browser_path = os.environ.get('PLAYWRIGHT_BROWSERS_PATH', str(Path(os.environ.get('XDG_CACHE_HOME', str(Path.home() / '.cache'))) / 'ms-playwright'))
        result = subprocess.run(['node', str(ROOT / 'tests/e2e/admin-browser.cjs')], env=dict(self.env, PLAYWRIGHT_BROWSERS_PATH=browser_path, CREWSYNC_ADMIN_FIXTURE=str(settings)), capture_output=True, text=True, timeout=90)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)


if __name__ == '__main__':
    names = ['test_http_interface_changes_reach_real_client', 'test_tls_interface_changes_reach_real_client']
    if os.environ.get('PLAYWRIGHT_MODULE'):
        names.append('test_browser_workflow')
    result = unittest.TextTestRunner(verbosity=2).run(unittest.TestSuite([Admin(name) for name in names]))
    raise SystemExit(not result.wasSuccessful())
