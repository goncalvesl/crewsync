"""Instalações reais isoladas: HTTPS, ativação, watch e bootstrap sem rede externa."""
import io
import os
from pathlib import Path
import pty
import select
import shutil
import signal
import socket
import subprocess
import sys
import tarfile
import tempfile
import time
import unittest

ROOT = Path(__file__).resolve().parents[2]

class Setup(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='crewsync setup % $ \' " ' )
        self.root = Path(self.temp.name)
        self.env = dict(os.environ, HOME=str(self.root / 'home'), XDG_CONFIG_HOME=str(self.root / 'units'))
        Path(self.env['HOME']).mkdir()
        runtime = self.root / 'runtime'
        runtime.mkdir(mode=0o700)
        self.env['XDG_RUNTIME_DIR'] = str(runtime)
        self.mock = self.root / 'mock'
        self.mock.mkdir()
        self.executable('systemctl', '#!/bin/sh\nexit 1\n')
        self.env['PATH'] = str(self.mock) + ':' + os.environ['PATH']
        self.package = self.root / 'package'
        for folder in ['bin', 'scripts', 'docs', 'config']:
            (self.package / folder).mkdir(parents=True, exist_ok=True)
        for source, dest in [('scripts/setup.sh', 'setup.sh'), ('scripts/install.sh', 'install.sh'), ('scripts/pilot-cert.sh', 'scripts/pilot-cert.sh'), ('LICENSE', 'LICENSE'), ('docs/MANUAL.md', 'docs/MANUAL.md'), ('config/server.example.toml', 'config/server.example.toml')]:
            shutil.copyfile(ROOT / source, self.package / dest)
        for role in ['server', 'client']:
            source = Path(os.environ.get('CREWSYNC_BIN_DIR', ROOT / 'target/debug')) / ('crewsync-' + role)
            shutil.copyfile(source, self.package / 'bin' / source.name)
        if os.environ.get('CREWSYNC_PACKAGE_DIR'):
            self.package = Path(os.environ['CREWSYNC_PACKAGE_DIR'])
        self.configs = []

    @staticmethod
    def stop_service(pid):
        # O serviço e seu worker podem sobreviver ao shell do assistente.
        # Fixar identidades via pidfd e esperar ambos antes de remover os dados.
        descriptors = []
        def collect(process):
            try:
                descriptor = os.pidfd_open(process)
            except ProcessLookupError:
                return
            descriptors.append(descriptor)
            children = Path(f'/proc/{process}/task/{process}/children')
            try:
                for child in children.read_text().split():
                    collect(int(child))
            except FileNotFoundError:
                pass
        try:
            collect(pid)
            for descriptor in descriptors:
                try:
                    signal.pidfd_send_signal(descriptor, signal.SIGTERM)
                except ProcessLookupError:
                    pass
            pending = list(descriptors)
            deadline = time.monotonic() + 5
            while pending:
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    raise TimeoutError('Serviço de teste não encerrou')
                ready, _, _ = select.select(pending, [], [], remaining)
                pending = [item for item in pending if item not in ready]
        finally:
            for descriptor in descriptors:
                os.close(descriptor)

    def tearDown(self):
        for config in reversed(self.configs):
            pid = config / 'service.pid'
            if pid.exists():
                self.stop_service(int(pid.read_text()))
        self.temp.cleanup()

    def test_service_cleanup_stops_and_waits_for_workers(self):
        worker = "import signal; signal.signal(signal.SIGTERM, lambda *_: exit(0)); print('ready', flush=True); signal.pause()"
        parent = "import subprocess, sys, signal; child = subprocess.Popen([sys.executable, '-c', sys.argv[1]], stdout=subprocess.PIPE, text=True); child.stdout.readline(); print(child.pid, flush=True); signal.pause()"
        process = subprocess.Popen([sys.executable, '-c', parent, worker], stdout=subprocess.PIPE, text=True)
        try:
            ready, _, _ = select.select([process.stdout], [], [], 5)
            self.assertTrue(ready, 'Worker de teste não iniciou')
            worker_pid = int(process.stdout.readline())
            descriptor = os.pidfd_open(worker_pid)
            try:
                self.stop_service(process.pid)
                self.assertEqual(select.select([descriptor], [], [], 0)[0], [descriptor])
                process.wait(timeout=5)
            finally:
                os.close(descriptor)
        finally:
            if process.poll() is None:
                self.stop_service(process.pid)
                process.wait(timeout=5)
            process.stdout.close()

    def executable(self, name, text):
        path = self.mock / name
        path.write_text(text)
        path.chmod(0o755)

    def setup(self, role, answers):
        return subprocess.run(['sh', str(self.package / 'setup.sh'), role], input='\n'.join(map(str, answers)) + '\n', text=True, capture_output=True, env=self.env, timeout=45)

    def setup_terminal(self, role, answers, invite):
        pid, fd = pty.fork()
        if pid == 0:
            os.execvpe('sh', ['sh', str(self.package / 'setup.sh'), role], self.env)
        os.write(fd, ('\n'.join(map(str, answers)) + '\n').encode())
        output = b''
        sent = False
        deadline = time.monotonic() + 45
        try:
            while time.monotonic() < deadline:
                if select.select([fd], [], [], 0.1)[0]:
                    try:
                        chunk = os.read(fd, 65536)
                    except OSError:
                        break
                    if not chunk:
                        break
                    output += chunk
                    if not sent and b'(entrada oculta):' in output:
                        os.write(fd, (invite + '\n').encode())
                        sent = True
            else:
                os.kill(pid, signal.SIGKILL)
                self.fail('Terminal setup timeout')
        finally:
            os.close(fd)
            _, status = os.waitpid(pid, 0)
        self.assertTrue(sent, output.decode(errors='replace'))
        return subprocess.CompletedProcess([], os.waitstatus_to_exitcode(status), output.decode(errors='replace'), '')

    def test_server_and_two_clients_ready_and_existing_config_preserved(self):
        prefix = self.root / 'programs with spaces'
        config = self.root / 'server config'
        data = self.root / 'server data'
        self.configs.append(config)
        with socket.socket() as sock:
            sock.bind(('127.0.0.1', 0))
            port = sock.getsockname()[1]
        answers = [prefix, config, data, '127.0.0.1', port, 'ana', 'Obras']
        result = self.setup('server', answers)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn('Servidor pronto:', result.stdout)
        self.assertEqual((config / 'tls/server.key').stat().st_mode & 0o777, 0o600)
        before = (config / 'server.toml').read_bytes()
        again = self.setup('server', answers)
        self.assertNotEqual(again.returncode, 0)
        self.assertEqual((config / 'server.toml').read_bytes(), before)
        updated = subprocess.run(['sh', str(self.package / 'install.sh'), '--server'], env=dict(self.env, PREFIX=str(prefix)), capture_output=True, text=True)
        self.assertEqual(updated.returncode, 0, updated.stderr)
        os.kill(int((config / 'service.pid').read_text()), 0)
        clients = []
        for index in [1, 2]:
            cfg = self.root / f'client {index}'
            folder = self.root / f'files {index}'
            self.configs.append(cfg)
            invite = subprocess.check_output([str(prefix / 'bin/crewsync-server'), 'invite', str(config / 'server.toml'), 'ana', '3600'], text=True).strip()
            answers = [prefix, cfg, folder, f'https://localhost:{port}', config / 'tls/ca.crt', f'pc{index}']
            result = self.setup_terminal('client', answers, invite) if index == 1 else self.setup('client', answers + [invite])
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertNotIn(invite, result.stdout + result.stderr)
            self.assertEqual((cfg / 'session.json').stat().st_mode & 0o777, 0o600)
            clients.append(folder / "Obras")
        (clients[0] / 'hello.txt').write_text('ready')
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline:
            if (clients[1] / 'hello.txt').exists():
                break
            time.sleep(0.1)
        self.assertEqual((clients[1] / 'hello.txt').read_text(), 'ready')
        # Uma nova permissão é descoberta pelo watch sem reconfigurar os clientes.
        with (config / 'server.toml').open('a') as settings:
            settings.write("\n[shares.Plantas.permissions]\nana='write'\n[shares.Privado.permissions]\n")
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline:
            if all((folder.parent / 'Plantas').is_dir() for folder in clients):
                break
            time.sleep(0.1)
        for folder in clients:
            self.assertTrue((folder.parent / 'Plantas').is_dir())
            self.assertFalse((folder.parent / 'Privado').exists())
        (clients[0].parent / 'Plantas/second.txt').write_text('another share')
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline:
            if (clients[1].parent / 'Plantas/second.txt').exists():
                break
            time.sleep(0.1)
        self.assertEqual((clients[1].parent / 'Plantas/second.txt').read_text(), 'another share')

    def test_nohup_survives_hangup_before_exec(self):
        # Reproduz fechamento do terminal antes de nohup instalar seu handler.
        real_nohup = shutil.which('nohup')
        self.executable('nohup', '#!/usr/bin/env python3\n' +
                        "import os, signal, sys\nos.kill(os.getpid(), signal.SIGHUP)\n" +
                        f"os.execv({real_nohup!r}, [{real_nohup!r}] + sys.argv[1:])\n")
        config = self.root / 'hangup server'
        self.configs.append(config)
        with socket.socket() as sock:
            sock.bind(('127.0.0.1', 0))
            port = sock.getsockname()[1]
        result = self.setup('server', [self.root / 'prefix', config, self.root / 'store', '127.0.0.1', port, 'ana', 'Obras'])
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        os.kill(int((config / 'service.pid').read_text()), 0)

    def test_remote_domain_certificate_and_endpoint_without_public_dns(self):
        config = self.root / 'remote config'
        self.configs.append(config)
        with socket.socket() as sock:
            sock.bind(('127.0.0.1', 0))
            port = sock.getsockname()[1]
        result = self.setup('server', [self.root / 'prefix', config, self.root / 'store', 'sync.example.test', port, 'ana', 'Obras'])
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn(f'https://sync.example.test:{port}', result.stdout)
        tls = config / 'tls'
        command = ['openssl', 'verify', '-CAfile', str(tls / 'ca.crt'), '-verify_hostname']
        good = subprocess.run(command + ['sync.example.test', str(tls / 'server.crt')], capture_output=True)
        self.assertEqual(good.returncode, 0, good.stderr)
        bad = subprocess.run(command + ['wrong.example.test', str(tls / 'server.crt')], capture_output=True)
        self.assertNotEqual(bad.returncode, 0)
        # SNI e verificação HTTPS no domínio, roteando apenas este teste para loopback.
        response = subprocess.run(['curl', '--noproxy', '*', '--cacert', str(tls / 'ca.crt'), '--resolve', f'sync.example.test:{port}:127.0.0.1', '--max-time', '5', '-sS', f'https://sync.example.test:{port}/'], capture_output=True)
        self.assertEqual(response.returncode, 0, response.stderr)

    def test_systemd_unit_preserves_special_characters_in_paths(self):
        self.executable('systemctl', '#!/usr/bin/env python3\n' + r'''import os, shlex, subprocess, sys
from pathlib import Path
if 'enable' in sys.argv:
    unit = Path(os.environ['XDG_CONFIG_HOME']) / 'systemd/user/crewsync-server.service'
    line = next(line for line in unit.read_text().splitlines() if line.startswith('ExecStart='))
    args = shlex.split(line.split('=', 1)[1].replace('%%', '%').replace('$$', '$'))
    config = Path(args[1]).parent
    with (config / 'service.log').open('w') as log:
        proc = subprocess.Popen(args, stdin=subprocess.DEVNULL, stdout=log, stderr=log, start_new_session=True)
    (config / 'service.pid').write_text(str(proc.pid))
sys.exit(0)
''')
        config = self.root / 'server'
        self.configs.append(config)
        with socket.socket() as sock:
            sock.bind(('127.0.0.1', 0))
            port = sock.getsockname()[1]
        result = self.setup('server', [self.root / 'prefix', config, self.root / 'store', '127.0.0.1', port, 'ana', 'Obras'])
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        unit = Path(self.env['XDG_CONFIG_HOME']) / 'systemd/user/crewsync-server.service'
        self.assertIn('%%', unit.read_text())
        self.assertIn('$$', unit.read_text())
        self.assertIn('systemctl --user status', result.stdout)
        if shutil.which('systemd-analyze'):
            verified = subprocess.run(['systemd-analyze', '--user', 'verify', str(unit)], capture_output=True, text=True, env=self.env)
            self.assertEqual(verified.returncode, 0, verified.stderr)

    def test_invalid_address_and_overlapping_directories_leave_installation_untouched(self):
        prefix = self.root / 'programs'
        cfg = self.root / 'config'
        for answers in [[prefix, cfg, self.root / 'data', value] for value in ['bad-ip', '', 'https://sync.example.com', 'sync.example.com:8443', '256.1.2.3', 'sync.example.com,DNS:evil.example']] + [[prefix, cfg, cfg / 'data']]:
            result = self.setup('server', answers)
            self.assertNotEqual(result.returncode, 0)
            self.assertFalse(prefix.exists())
            self.assertFalse(cfg.exists())

    def bootstrap(self, corrupt=False):
        import hashlib
        name = 'crewsync-0.2.0-alpha.3-linux-x86_64'
        archive = self.root / 'fixture.tar.gz'
        with tarfile.open(archive, 'w:gz') as tar:
            content = b'#!/bin/sh\nprintf "SETUP-EXECUTED:%s\\n" "$1"\n'
            info = tarfile.TarInfo(name + '/setup.sh')
            info.size = len(content)
            tar.addfile(info, io.BytesIO(content))
        digest = '0' * 64 if corrupt else hashlib.sha256(archive.read_bytes()).hexdigest()
        (self.root / 'checksum').write_text(digest + '  fixture.tar.gz\n')
        self.executable('uname', '#!/bin/sh\ncase "$1" in -s) echo Linux;; -m) echo x86_64;; esac\n')
        self.executable('curl', '''#!/bin/sh
case "$*" in *api.github.com*) printf '{\n  "tag_name": "v0.2.0-alpha.3",\n}\n'; exit;; esac
source="$FIXTURE/fixture.tar.gz"
case "$*" in *sha256*) source="$FIXTURE/checksum";; esac
while [ "$#" -gt 0 ]; do if [ "$1" = -o ]; then cp "$source" "$2"; exit; fi; shift; done
exit 1
''')
        self.env['FIXTURE'] = str(self.root)
        pid, fd = pty.fork()
        if pid == 0:
            os.execvpe('sh', ['sh', '-c', 'cat "$1" | sh -s -- client', 'bootstrap', str(ROOT / 'scripts/setup-online.sh')], self.env)
        output = b''
        deadline = time.monotonic() + 10
        try:
            while time.monotonic() < deadline:
                ready, _, _ = select.select([fd], [], [], 0.1)
                if ready:
                    try:
                        chunk = os.read(fd, 4096)
                    except OSError:
                        break
                    if not chunk:
                        break
                    output += chunk
            else:
                os.kill(pid, signal.SIGKILL)
                self.fail('Bootstrap timeout')
        finally:
            os.close(fd)
            _, status = os.waitpid(pid, 0)
        return os.waitstatus_to_exitcode(status), output.decode(errors='replace')

    def test_online_bootstrap_selects_latest_alpha_and_checks_hash_before_execution(self):
        status, output = self.bootstrap()
        self.assertEqual(status, 0, output)
        self.assertIn('SETUP-EXECUTED:client', output)
        status, output = self.bootstrap(corrupt=True)
        self.assertNotEqual(status, 0, output)
        self.assertNotIn('SETUP-EXECUTED:', output)

if __name__ == '__main__':
    unittest.main()
