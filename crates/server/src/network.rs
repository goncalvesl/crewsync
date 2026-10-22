//! Worker serializado por trás da ponte HTTPS.
use crate::{access::AuthorizedStore, auth::AuthStore, config::Config};
use crewsync_protocol::{Frame, Operation, Response, invalid, transport};
use std::fs::File;
use std::io::BufRead;
use std::io::{self, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

// EOF antes do tamanho declarado precisa falhar antes de publicar o manifesto.
struct ExactBody<'a, R> {
    input: &'a mut R,
    remaining: u64,
}
impl<R: Read> Read for ExactBody<'_, R> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        if self.remaining == 0 || bytes.is_empty() {
            return Ok(0);
        }
        let length = (bytes.len() as u64).min(self.remaining) as usize;
        let count = self.input.read(&mut bytes[..length])?;
        if count == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "corpo interrompido",
            ));
        }
        self.remaining -= count as u64;
        Ok(count)
    }
}

enum Payload {
    Json(Vec<u8>),
    File(File, u64),
}
fn json(value: &impl serde::Serialize) -> io::Result<Payload> {
    Ok(Payload::Json(
        serde_json::to_vec(value).map_err(io::Error::other)?,
    ))
}

fn startup_context(stage: &str, error: io::Error) -> io::Error {
    if stage == "armazenamento" && error.kind() == io::ErrorKind::WouldBlock {
        return io::Error::new(
            error.kind(),
            "armazenamento já está em uso por outro processo",
        );
    }
    io::Error::new(error.kind(), format!("{stage}: {error}"))
}

fn open_worker(path: &Path) -> io::Result<(AuthStore, AuthorizedStore)> {
    let config = Config::load(path).map_err(|e| startup_context("configuração", e))?;
    let auth = AuthStore::open(config.storage_root())
        .map_err(|e| startup_context("raiz de armazenamento/autenticação", e))?;
    auth.devices()
        .map_err(|e| startup_context("estado de autenticação", e))?;
    let store = AuthorizedStore::open(path).map_err(|e| startup_context("armazenamento", e))?;
    Ok((auth, store))
}

pub fn worker(path: &Path) -> io::Result<()> {
    let mut output = io::stdout().lock();
    let (auth, mut store) = match open_worker(path) {
        Ok(value) => value,
        Err(error) => {
            // Apenas erros locais sanitizados; nunca repassar stderr arbitrário
            // ou conteúdo de configuração/credenciais pela ponte.
            serde_json::to_writer(
                &mut output,
                &serde_json::json!({"startup_error": error.to_string()}),
            )
            .map_err(io::Error::other)?;
            output.write_all(b"\n")?;
            output.flush()?;
            return Err(error);
        }
    };
    let mut input = BufReader::new(io::stdin().lock());
    output.write_all(b"READY\n")?;
    output.flush()?;
    loop {
        let line = transport::read_line(&mut input)?;
        let frame: Frame =
            serde_json::from_slice(&line).map_err(|_| invalid("requisição inválida"))?;
        if frame.body_bytes > (1u64 << 40) {
            return Err(invalid("corpo longo demais"));
        }
        // Configuração inválida mantém a última válida; não imprimir seu conteúdo.
        if store.reload_config().is_err() {
            eprintln!("Recarga recusada; mantendo configuração válida anterior.");
        }
        let mut body = ExactBody {
            input: &mut input,
            remaining: frame.body_bytes,
        };
        let mut authentication_failed = false;
        let result = (|| {
            if let Operation::Activate { code, name } = &frame.request.operation {
                if frame.body_bytes != 0 {
                    return Err(invalid("corpo inesperado"));
                }
                let value = auth.activate(code, name, |user| store.user_enabled(user));
                if value.is_err() {
                    authentication_failed = true;
                }
                return json(&value?);
            }
            let user = auth.authenticate(frame.request.token.as_deref().unwrap_or(""));
            let user = match user {
                Ok(user) if store.user_enabled(&user) => user,
                _ => {
                    authentication_failed = true;
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "credencial inválida",
                    ));
                }
            };
            if !matches!(frame.request.operation, Operation::Put { .. }) && frame.body_bytes != 0 {
                return Err(invalid("corpo inesperado"));
            }
            match frame.request.operation {
                Operation::Shares => json(&store.shares(&user)),
                Operation::Manifest { share } => json(&store.manifest(&user, &share)?),
                Operation::History { share, path } => json(&store.history(&user, &share, &path)?),
                Operation::Restore {
                    share,
                    path,
                    revision,
                    base,
                } => json(&store.restore(&user, &share, &path, revision, base)?),
                Operation::Get { share, path } => {
                    let file = store.file(&user, &share, &path)?;
                    let size = file.metadata()?.len();
                    Ok(Payload::File(file, size))
                }
                Operation::Put {
                    share,
                    path,
                    base,
                    hash,
                    size,
                } => {
                    if size != frame.body_bytes {
                        return Err(invalid("tamanho inconsistente"));
                    }
                    json(&store.put_file(&user, &share, &path, base, &hash, &mut body)?)
                }
                Operation::Mkdir { share, path, base } => {
                    json(&store.create_directory(&user, &share, &path, base)?)
                }
                Operation::Delete { share, path, base } => {
                    json(&store.delete(&user, &share, &path, base)?)
                }
                Operation::Activate { .. } => Err(invalid("operação inválida")),
            }
        })();
        let remaining = body.remaining;
        if io::copy(&mut body, &mut io::sink())? != remaining {
            return Err(invalid("upload interrompido"));
        }
        let (status, payload) = match result {
            Ok(payload) => (200, payload),
            Err(error) => {
                let status = if authentication_failed {
                    401
                } else {
                    match error.kind() {
                        io::ErrorKind::PermissionDenied => 403,
                        io::ErrorKind::NotFound => 404,
                        io::ErrorKind::AlreadyExists => 409,
                        io::ErrorKind::InvalidData | io::ErrorKind::InvalidInput => 400,
                        _ => 500,
                    }
                };
                (
                    status,
                    Payload::Json(b"{\"error\":\"operation_failed\"}".to_vec()),
                )
            }
        };
        let length = match &payload {
            Payload::Json(bytes) => bytes.len() as u64,
            Payload::File(_, size) => *size,
        };
        serde_json::to_writer(&mut output, &Response { status, length })
            .map_err(io::Error::other)?;
        output.write_all(b"\n")?;
        match payload {
            Payload::Json(bytes) => output.write_all(&bytes)?,
            Payload::File(mut file, _) => {
                io::copy(&mut file, &mut output)?;
            }
        }
        output.flush()?;
    }
}

struct WorkerProcess {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
}
impl Drop for WorkerProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
impl WorkerProcess {
    fn open(config: &Path) -> io::Result<Self> {
        let mut child = Command::new(std::env::current_exe()?)
            .arg("worker")
            .arg(config)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;
        let input = child
            .stdin
            .take()
            .ok_or_else(|| invalid("worker indisponível"))?;
        let output = BufReader::new(
            child
                .stdout
                .take()
                .ok_or_else(|| invalid("worker indisponível"))?,
        );
        let mut worker = Self {
            child,
            input,
            output,
        };
        let line = transport::read_line(&mut worker.output)?;
        if line != b"READY\n" {
            let error: serde_json::Value =
                serde_json::from_slice(&line).map_err(|_| invalid("worker não iniciou"))?;
            return Err(invalid(
                error["startup_error"]
                    .as_str()
                    .unwrap_or("worker não iniciou"),
            ));
        }
        Ok(worker)
    }
    fn respond(&mut self, client: &mut (impl BufRead + Write), head: &[u8]) -> io::Result<()> {
        let length = transport::request_length(head)?;
        if length > (1u64 << 40) + crewsync_protocol::MAX_FRAME {
            return Err(invalid("corpo longo demais"));
        }
        let line = transport::read_line(&mut client.take(length))?;
        let frame: Frame = serde_json::from_slice(&line).map_err(|_| invalid("frame inválido"))?;
        if frame.body_bytes.checked_add(line.len() as u64) != Some(length) {
            return Err(invalid("tamanho HTTP inconsistente"));
        }
        self.input.write_all(&line)?;
        if io::copy(&mut client.take(frame.body_bytes), &mut self.input)? != frame.body_bytes {
            return Err(invalid("upload interrompido"));
        }
        self.input.flush()?;
        let response: Response = serde_json::from_slice(&transport::read_line(&mut self.output)?)
            .map_err(|_| invalid("worker inválido"))?;
        write!(
            client,
            "HTTP/1.1 {} Result\r\nContent-Length: {}\r\nConnection: close\r\nContent-Type: application/octet-stream\r\n\r\n",
            response.status, response.length
        )?;
        if io::copy(&mut self.output.by_ref().take(response.length), client)? != response.length {
            return Err(invalid("resposta interrompida"));
        }
        client.flush()?;
        // Somente códigos e métricas; sem token, código, usuário, caminho ou conteúdo.
        eprintln!(
            "event=request status={} response_bytes={}",
            response.status, response.length
        );
        Ok(())
    }
}

// BufReader normalmente não implementa Write; conservar o buffer de leitura
// enquanto as respostas são escritas na mesma conexão TLS.
struct Duplex<T>(BufReader<T>);
impl<T: Read> Read for Duplex<T> {
    fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
        self.0.read(b)
    }
}
impl<T: Read> BufRead for Duplex<T> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.0.fill_buf()
    }
    fn consume(&mut self, n: usize) {
        self.0.consume(n);
    }
}
impl<T: Read + Write> Write for Duplex<T> {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        self.0.get_mut().write(b)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.0.get_mut().flush()
    }
}

pub fn serve(config: &Path, host: &str, port: &str, cert: &Path, key: &Path) -> io::Result<()> {
    let tls = transport::TlsServer::new(cert, key).map_err(|e| {
        if e.kind() == io::ErrorKind::NotFound {
            invalid("Falha ao iniciar HTTPS: certificado ou chave TLS não encontrado")
        } else {
            invalid("Falha ao iniciar HTTPS: certificado/chave TLS inválidos ou incompatíveis")
        }
    })?;
    serve_transport(config, host, port, Some(tls))
}

/// Backend para proxy ou teste explícito; não interpreta cabeçalhos forwarded.
pub fn serve_http(config: &Path, host: &str, port: &str) -> io::Result<()> {
    eprintln!(
        "HTTP sem criptografia; use loopback atrás de proxy HTTPS ou somente testes isolados."
    );
    serve_transport(config, host, port, None)
}

fn serve_transport(
    config: &Path,
    host: &str,
    port: &str,
    tls: Option<transport::TlsServer>,
) -> io::Result<()> {
    let scheme = if tls.is_some() { "HTTPS" } else { "HTTP" };
    let listener = TcpListener::bind(format!("{host}:{port}")).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("Falha ao iniciar {scheme}: porta/endereço indisponível ou já está em uso"),
        )
    })?;
    let mut worker = WorkerProcess::open(config)
        .map_err(|e| invalid(&format!("Falha ao iniciar {scheme}: {e}")))?;
    let mut admin = crate::admin::Admin::new(config)?;
    println!("CrewSync {scheme} ready {}", listener.local_addr()?.port());
    io::stdout().flush()?;
    for tcp in listener.incoming() {
        let tcp = tcp?;
        tcp.set_read_timeout(Some(std::time::Duration::from_secs(30)))?;
        tcp.set_write_timeout(Some(std::time::Duration::from_secs(30)))?;
        let success = match &tls {
            Some(tls) => respond(&mut worker, &mut admin, tls.accept(tcp)?),
            None => respond(&mut worker, &mut admin, tcp),
        };
        if !success {
            // Um corpo truncado pode deixar o IPC desalinhado. Matar antes de
            // reabrir garante liberação do lock e recuperação de temporários.
            drop(worker);
            worker = WorkerProcess::open(config)?;
        }
    }
    Ok(())
}

fn respond(
    worker: &mut WorkerProcess,
    admin: &mut crate::admin::Admin,
    stream: impl Read + Write,
) -> bool {
    let mut client = Duplex(BufReader::new(stream));
    let parsed = (|| {
        let head = transport::read_http_head(&mut client)?;
        let request = crate::admin::Request::parse(&head)?;
        Ok::<_, io::Error>((head, request))
    })();
    let (head, request) = match parsed {
        Ok(request) => request,
        Err(_) => {
            let _ = crate::admin::error(400, "Requisição HTTP inválida.").write(&mut client);
            return true;
        }
    };
    if request.target != "/v1/rpc" {
        let response = admin.respond(&request, &mut client);
        let _ = response.write(&mut client);
        return true;
    }
    if worker.respond(&mut client, &head).is_ok() {
        return true;
    }
    eprintln!("event=request_failed");
    let _ =
        client.write_all(b"HTTP/1.1 400 Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
    let _ = client.flush();
    false
}
