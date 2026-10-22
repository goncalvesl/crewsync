#!/usr/bin/env python3
"""Benchmark sintético local, sem credenciais/dados reais no relatório."""
import argparse
import datetime
import hashlib
import json
import os
import pathlib
import platform
import selectors
import signal
import subprocess
import tempfile
import threading
import time

REPO = pathlib.Path(__file__).resolve().parents[1]


def measured(args, stdin=None):
    with tempfile.TemporaryFile() as out, tempfile.TemporaryFile() as err:
        start = time.monotonic()
        child = subprocess.Popen([str(a) for a in args], stdin=subprocess.PIPE if stdin else subprocess.DEVNULL, stdout=out, stderr=err)
        timer = threading.Timer(90, child.kill)
        timer.start()
        try:
            if stdin:
                child.stdin.write(stdin)
                child.stdin.close()
            _, status, usage = os.wait4(child.pid, 0)
            child.returncode = os.waitstatus_to_exitcode(status)
        finally:
            timer.cancel()
        out.seek(0)
        err.seek(0)
        if child.returncode:
            raise RuntimeError('Comando do benchmark falhou: ' + err.read().decode(errors='replace'))
        return out.read(), dict(wall_seconds=round(time.monotonic() - start, 4), cpu_seconds=round(usage.ru_utime + usage.ru_stime, 4), peak_rss_kib=usage.ru_maxrss)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--bin-dir', type=pathlib.Path, default=REPO / 'target/release')
    parser.add_argument('--report', type=pathlib.Path, required=True)
    args = parser.parse_args()
    server, client = (args.bin_dir.resolve() / name for name in ['crewsync-server', 'crewsync-client'])
    with tempfile.TemporaryDirectory(prefix='crewsync-bench-') as temporary:
        root = pathlib.Path(temporary)
        config = root / 'server.toml'
        config.write_text(f"version=1\n[server]\nstorage_root='{root}/store'\n[users.teste]\nenabled=true\n[shares.Obras.permissions]\nteste='write'\n")
        measured([REPO / 'scripts/pilot-cert.sh', root / 'tls', '127.0.0.1'])
        with tempfile.TemporaryFile() as logs:
            process = subprocess.Popen([str(server), 'serve', str(config), '127.0.0.1', '0', str(root / 'tls/server.crt'), str(root / 'tls/server.key')], stdout=subprocess.PIPE, stderr=logs, start_new_session=True)
            try:
                selector = selectors.DefaultSelector()
                selector.register(process.stdout, selectors.EVENT_READ)
                if not selector.select(10):
                    raise RuntimeError('Servidor não iniciou')
                line = process.stdout.readline().decode()
                if not line.startswith('CrewSync HTTPS ready '):
                    raise RuntimeError('Inicialização inválida')
                url = 'https://127.0.0.1:' + line.split()[-1]
                sessions = []
                for name in ['a', 'b']:
                    code, _ = measured([server, 'invite', config, 'teste', '600'])
                    session = root / f'{name}.json'
                    measured([client, 'activate', url, root / 'tls/ca.crt', session, name], code)
                    sessions.append(session)
                a, b = root / 'a', root / 'b'
                a.mkdir()
                b.mkdir()
                for i in range(100):
                    (a / f'{i:03}.txt').write_bytes(hashlib.sha256(str(i).encode()).digest() * 8)
                (a / 'large.bin').write_bytes(hashlib.sha256(b'large').digest() * (4 * 1024 * 1024 // 32))
                def sync(session, folder):
                    return measured([client, 'sync', session, 'Obras', folder])
                initial_output, initial = sync(sessions[0], a)
                _, download = sync(sessions[1], b)
                _, idle = sync(sessions[1], b)
                (a / '000.txt').write_bytes(b'changed')
                propagation_start = time.monotonic()
                sync(sessions[0], a)
                sync(sessions[1], b)
                propagation = time.monotonic() - propagation_start
                for file in a.iterdir():
                    if not file.is_file():
                        continue
                    assert hashlib.sha256(file.read_bytes()).digest() == hashlib.sha256((b / file.name).read_bytes()).digest()
                report = dict(schema=1, scope='localhost; sintético; não representa duas máquinas físicas', recorded_at=datetime.datetime.now(datetime.timezone.utc).isoformat(), binary_version=measured([client, '--version'])[0].decode().strip(), environment=dict(cpu_count=os.cpu_count(), cpu_model=next((line.split(':', 1)[1].strip() for line in pathlib.Path('/proc/cpuinfo').read_text().splitlines() if line.startswith('model name')), 'unknown'), kernel=platform.release(), architecture=platform.machine(), distribution=platform.freedesktop_os_release().get('PRETTY_NAME'), cpu=platform.processor() or platform.machine()), files=101, content_bytes=4 * 1024 * 1024 + 100 * 256, initial_upload=initial, initial_download=download, idle_sync=idle, manual_propagation_cycle_seconds=round(propagation, 4), payload_bytes_each_initial_direction=4 * 1024 * 1024 + 100 * 256, network_wire_bytes=None, result='hashes iguais nos dois clientes', cli_output=initial_output.decode().strip())
                args.report.parent.mkdir(parents=True, exist_ok=True)
                args.report.write_text(json.dumps(report, indent=2, ensure_ascii=False) + '\n')
                print(json.dumps(report, indent=2, ensure_ascii=False))
            finally:
                os.killpg(process.pid, signal.SIGTERM)
                process.wait(timeout=10)


if __name__ == '__main__':
    main()
