"""Ponte TLS do piloto; regras e dados pertencem aos binários Rust."""
import http.client
import http.server
import json
import socket
import ssl
import subprocess
import sys
import urllib.parse

MAX_FRAME = 65536
MAX_BODY = 1099511627776

def transfer(source, target, size):
    while size:
        data = source.read(min(size, 65536))
        if not data:
            raise OSError('transferencia incompleta')
        target.write(data)
        size -= len(data)

def context_client(ca):
    context = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
    context.minimum_version = ssl.TLSVersion.TLSv1_2
    context.load_verify_locations(cafile=ca)
    return context

def client(url, ca):
    address = urllib.parse.urlsplit(url)
    if address.scheme != 'https' or not address.hostname or address.username or address.password or address.query or address.fragment or address.path not in ('', '/'):
        raise ValueError('endereco HTTPS invalido')
    head = sys.stdin.buffer.readline(MAX_FRAME + 1)
    if len(head) > MAX_FRAME or not head.endswith(b'\n'):
        raise ValueError('frame invalido')
    frame = json.loads(head)
    size = frame['body_bytes']
    if type(size) is not int or not 0 <= size <= MAX_BODY:
        raise ValueError('tamanho invalido')
    connection = http.client.HTTPSConnection(address.hostname, address.port or 443, timeout=30, context=context_client(ca))
    try:
        connection.putrequest('POST', '/v1/rpc')
        connection.putheader('Content-Type', 'application/octet-stream')
        connection.putheader('Content-Length', str(len(head) + size))
        connection.endheaders()
        connection.send(head)
        while size:
            data = sys.stdin.buffer.read(min(size, 65536))
            if not data:
                raise OSError('envio incompleto')
            connection.send(data)
            size -= len(data)
        response = connection.getresponse()
        lengths = response.headers.get_all('Content-Length', [])
        if len(lengths) != 1 or response.getheader('Transfer-Encoding'):
            raise ValueError('resposta invalida')
        length = int(lengths[0])
        if not 0 <= length <= MAX_BODY:
            raise ValueError('resposta excede limite')
        sys.stdout.buffer.write((json.dumps({'status': response.status, 'length': length}) + '\n').encode())
        sys.stdout.buffer.flush()
        transfer(response, sys.stdout.buffer, length)
        sys.stdout.buffer.flush()
    finally:
        connection.close()

def serve(binary, config, host, port, cert, key):
    context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    context.minimum_version = ssl.TLSVersion.TLSv1_2
    context.load_cert_chain(cert, key)
    class Server(http.server.HTTPServer):
        request_queue_size = 8
        def start_worker(self):
            self.worker = subprocess.Popen([binary, 'worker', config], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, bufsize=65536)
            # Worker só confirma após abrir e validar armazenamento e identidade.
            if self.worker.stdout.readline(MAX_FRAME + 1) != b'READY\n':
                self.worker.kill()
                self.worker.wait()
                raise OSError('worker indisponivel')
        def reset_worker(self):
            self.worker.kill()
            self.worker.wait()
            self.start_worker()
        def get_request(self):
            connection, address = self.socket.accept()
            connection.settimeout(30)
            try:
                return context.wrap_socket(connection, server_side=True), address
            except Exception:
                connection.close()
                raise
        def handle_error(self, request, address):
            pass  # Não registrar requisição, código, token ou conteúdo.
    class Handler(http.server.BaseHTTPRequestHandler):
        protocol_version = 'HTTP/1.0'
        def log_message(self, *args):
            pass
        def do_POST(self):
            if self.path != '/v1/rpc':
                self.send_error(404)
                return
            lengths = self.headers.get_all('Content-Length', [])
            if len(lengths) != 1 or self.headers.get('Transfer-Encoding'):
                self.send_error(411)
                return
            try:
                total = int(lengths[0])
                if not 1 <= total <= MAX_BODY + MAX_FRAME:
                    raise ValueError()
                head = self.rfile.readline(min(total + 1, MAX_FRAME + 1))
                if len(head) > MAX_FRAME or not head.endswith(b'\n'):
                    raise ValueError()
                frame = json.loads(head)
                size = frame['body_bytes']
                if type(size) is not int or not 0 <= size <= MAX_BODY or len(head) + size != total:
                    raise ValueError()
            except Exception:
                self.send_error(400)
                return
            try:
                worker = self.server.worker
                worker.stdin.write(head)
                transfer(self.rfile, worker.stdin, size)
                worker.stdin.flush()
                line = worker.stdout.readline(MAX_FRAME + 1)
                response = json.loads(line)
                size = response['length']
                status = response['status']
                if type(size) is not int or not 0 <= size <= MAX_BODY or status not in (200, 400, 401, 403, 404, 409, 500):
                    raise ValueError()
                self.send_response(status)
                self.send_header('Content-Length', str(size))
                self.send_header('Content-Type', 'application/octet-stream')
                self.end_headers()
                transfer(worker.stdout, self.wfile, size)
                self.wfile.flush()
            except Exception:
                # Reinício recupera temporários; nunca continua IPC dessincronizado.
                self.server.reset_worker()
                self.close_connection = True
    server = Server((host, int(port)), Handler)
    try:
        server.start_worker()
        print('CrewSync HTTPS ready ' + str(server.server_address[1]), flush=True)
        server.serve_forever(poll_interval=0.2)
    finally:
        if hasattr(server, 'worker'):
            server.worker.kill()
            server.worker.wait()
        server.server_close()

def main():
    try:
        if sys.argv[1] == 'request':
            client(sys.argv[2], sys.argv[3])
        elif sys.argv[1] == 'serve':
            serve(*sys.argv[2:])
        else:
            raise ValueError()
    except Exception as error:
        if len(sys.argv) > 1 and sys.argv[1] == 'request':
            status = 496 if isinstance(error, ssl.SSLError) else (400 if isinstance(error, ValueError) else 503)
            try:
                sys.stdout.buffer.write((json.dumps({'status': status, 'length': 0}) + '\n').encode())
                sys.stdout.buffer.flush()
            except Exception:
                pass
        print('Falha no transporte HTTPS; verifique certificado, conexao e configuracao.', file=sys.stderr)
        sys.exit(1)

if __name__ == '__main__':
    main()
