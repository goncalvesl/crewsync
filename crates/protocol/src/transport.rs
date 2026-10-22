//! Ponte HTTPS em processo isolado, sem shell ou opção TLS insegura.
use crate::{Frame, MAX_FRAME, MAX_JSON, Request, Response, invalid};
use serde::de::DeserializeOwned;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};

pub const BRIDGE: &str = include_str!("transport.py");

pub fn read_line(reader: &mut impl BufRead) -> io::Result<Vec<u8>> {
    let mut line = Vec::new();
    reader.take(MAX_FRAME + 1).read_until(b'\n', &mut line)?;
    if line.len() as u64 > MAX_FRAME || !line.ends_with(b"\n") {
        return Err(invalid("frame incompleto ou longo demais"));
    }
    Ok(line)
}

pub fn exchange(
    url: &str,
    ca: &Path,
    request: Request,
    reader: &mut dyn Read,
    size: u64,
    output: &mut dyn Write,
    limit: u64,
) -> io::Result<()> {
    let mut child = Command::new("/usr/bin/python3")
        .args(["-I", "-c", BRIDGE, "request", url])
        .arg(ca)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let result = (|| {
        let mut line = serde_json::to_vec(&Frame {
            request,
            body_bytes: size,
        })
        .map_err(io::Error::other)?;
        line.push(b'\n');
        if line.len() as u64 > MAX_FRAME {
            return Err(invalid("requisição longa demais"));
        }
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| invalid("ponte indisponível"))?;
        stdin.write_all(&line)?;
        if io::copy(&mut reader.take(size), &mut stdin)? != size {
            return Err(invalid("arquivo mudou durante envio"));
        }
        drop(stdin);
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| invalid("ponte indisponível"))?;
        let mut stdout = BufReader::new(stdout);
        let response: Response = serde_json::from_slice(&read_line(&mut stdout)?)
            .map_err(|_| invalid("resposta inválida"))?;
        if response.status != 200 {
            let kind = match response.status {
                401 | 403 => io::ErrorKind::PermissionDenied,
                404 => io::ErrorKind::NotFound,
                409 => io::ErrorKind::AlreadyExists,
                400 => io::ErrorKind::InvalidInput,
                496 => io::ErrorKind::InvalidData,
                _ => io::ErrorKind::ConnectionAborted,
            };
            return Err(io::Error::new(
                kind,
                format!("servidor respondeu HTTP {}", response.status),
            ));
        }
        if response.length > limit {
            return Err(invalid("resposta excede limite"));
        }
        if io::copy(&mut stdout.take(response.length), output)? != response.length {
            return Err(invalid("download interrompido"));
        }
        if !child.wait()?.success() {
            return Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "falha HTTPS (confira certificado e conexão)",
            ));
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = child.kill();
        let _ = child.wait();
    }
    result
}

pub fn json<T: DeserializeOwned>(url: &str, ca: &Path, request: Request) -> io::Result<T> {
    let mut bytes = Vec::new();
    exchange(url, ca, request, &mut io::empty(), 0, &mut bytes, MAX_JSON)?;
    serde_json::from_slice(&bytes).map_err(|_| invalid("resposta JSON inválida"))
}
