//! Worker serializado por trás da ponte HTTPS.
use crate::{access::AuthorizedStore, auth::AuthStore, config::Config};
use crewsync_protocol::{Frame, Operation, Response, invalid, transport};
use std::fs::File;
use std::io::{self, BufReader, Read, Write};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;

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

pub fn serve(config: &Path, host: &str, port: &str, cert: &Path, key: &Path) -> io::Result<()> {
    let binary = std::env::current_exe()?;
    // Substituir o supervisor evita deixar uma ponte órfã ao encerrar este PID.
    Err(Command::new("/usr/bin/python3")
        .args(["-I", "-c", transport::BRIDGE, "serve"])
        .arg(binary)
        .arg(config)
        .args([host, port])
        .arg(cert)
        .arg(key)
        .exec())
}
