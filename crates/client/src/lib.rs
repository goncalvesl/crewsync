// SPDX-License-Identifier: GPL-3.0-only
//! Cliente Linux de piloto: ativação e reconciliação por revisão-base.
pub mod root;
pub mod sync;
pub mod tree;
use crewsync_protocol::{Operation, Request, disk, invalid, transport};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Read};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub url: String,
    #[serde(default)]
    pub ca: Option<PathBuf>,
    #[serde(default)]
    pub allow_http: bool,
    token: String,
    pub device_id: String,
}
#[derive(Deserialize)]
struct Activation {
    token: String,
    device_id: String,
}
impl Session {
    pub fn load(path: &Path) -> io::Result<Self> {
        let metadata = fs::symlink_metadata(path)?;
        if !metadata.is_file() || metadata.permissions().mode() & 0o077 != 0 {
            return Err(invalid(
                "credencial deve ser arquivo privado 0600, sem link",
            ));
        }
        let session: Self = disk::read_json(path, 64 * 1024)?;
        disk::validate_hash(&session.token)?;
        transport::validate_url(&session.url, session.trust()?)?;
        Ok(session)
    }
    fn trust(&self) -> io::Result<transport::Trust<'_>> {
        match (&self.ca, self.allow_http) {
            (Some(ca), false) => Ok(transport::Trust::CustomCa(ca)),
            (None, false) => Ok(transport::Trust::System),
            (None, true) => Ok(transport::Trust::PlainHttp),
            (Some(_), true) => Err(invalid("sessão combina HTTP com CA; configuração inválida")),
        }
    }
    pub fn request(&self, operation: Operation) -> Request {
        Request {
            token: Some(self.token.clone()),
            operation,
        }
    }
    pub fn call<T: serde::de::DeserializeOwned>(&self, operation: Operation) -> io::Result<T> {
        transport::json_with(&self.url, self.trust()?, self.request(operation))
    }
}

pub fn activate(url: &str, ca: &Path, output: &Path, name: &str, code: &str) -> io::Result<()> {
    activate_with(url, transport::Trust::CustomCa(ca), output, name, code)
}

pub fn activate_with(
    url: &str,
    trust: transport::Trust<'_>,
    output: &Path,
    name: &str,
    code: &str,
) -> io::Result<()> {
    transport::validate_url(url, trust)?;
    let absolute_output = std::path::absolute(output)?;
    let output = absolute_output.as_path();
    if output.try_exists()? {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "arquivo de credencial já existe",
        ));
    }
    // Reservar antes de consumir convite; falha de rede deixa arquivo vazio removível.
    let reserved = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(output)?;
    let result = (|| {
        let ca = match trust {
            transport::Trust::CustomCa(ca) => Some(std::path::absolute(ca)?),
            _ => None,
        };
        let response: Activation = transport::json_with(
            url,
            trust,
            Request {
                token: None,
                operation: Operation::Activate {
                    code: code.to_owned(),
                    name: name.to_owned(),
                },
            },
        )?;
        disk::validate_hash(&response.token)?;
        disk::atomic_json(
            output,
            &Session {
                url: url.to_owned(),
                ca,
                allow_http: matches!(trust, transport::Trust::PlainHttp),
                token: response.token,
                device_id: response.device_id,
            },
        )
    })();
    drop(reserved);
    if result.is_err() {
        let _ = fs::remove_file(output);
    }
    result
}

/// Lê uma única linha limitada: Enter confirma, EOF também funciona em pipes.
pub fn read_code(input: impl Read) -> io::Result<String> {
    let mut data = String::new();
    BufReader::new(input).take(129).read_line(&mut data)?;
    if data.len() > 128 {
        return Err(invalid("código de convite longo demais"));
    }
    let code = data.trim();
    disk::validate_hash(code)
        .map_err(|_| invalid("código de convite inválido; use o código gerado por invite"))?;
    Ok(code.to_owned())
}

/// Transporte do reconciliador; separa regras de disco do HTTPS para testar falhas.
pub trait Remote {
    fn identity(&self) -> &str;
    fn transfer(
        &self,
        operation: Operation,
        input: &mut dyn Read,
        size: u64,
        output: &mut dyn std::io::Write,
        limit: u64,
    ) -> io::Result<()>;
    fn call<T: serde::de::DeserializeOwned>(&self, operation: Operation) -> io::Result<T> {
        let mut bytes = Vec::new();
        self.transfer(
            operation,
            &mut io::empty(),
            0,
            &mut bytes,
            crewsync_protocol::MAX_JSON,
        )?;
        serde_json::from_slice(&bytes).map_err(|_| invalid("resposta JSON inválida"))
    }
}
impl Remote for Session {
    fn identity(&self) -> &str {
        &self.url
    }
    fn transfer(
        &self,
        operation: Operation,
        input: &mut dyn Read,
        size: u64,
        output: &mut dyn std::io::Write,
        limit: u64,
    ) -> io::Result<()> {
        transport::exchange_with(
            &self.url,
            self.trust()?,
            self.request(operation),
            input,
            size,
            output,
            limit,
        )
    }
}

#[cfg(test)]
mod activation_input_tests {
    use super::*;
    #[test]
    fn legacy_sessions_keep_private_ca_and_http_requires_saved_opt_in() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("session.json");
        let token = "a".repeat(64);
        disk::atomic_json(&path, &serde_json::json!({"url":"https://localhost:8443", "ca":"/private/ca.crt", "token":token, "device_id":"device"})).unwrap();
        let legacy = Session::load(&path).unwrap();
        assert!(!legacy.allow_http);
        assert_eq!(legacy.ca, Some(PathBuf::from("/private/ca.crt")));
        for allow_http in [false, true] {
            disk::atomic_json(&path, &serde_json::json!({"url":"http://localhost:8080", "ca":null, "allow_http":allow_http, "token":token, "device_id":"device"})).unwrap();
            assert_eq!(Session::load(&path).is_ok(), allow_http);
        }
        disk::atomic_json(&path, &serde_json::json!({"url":"http://localhost:8080", "ca":"/private/ca.crt", "allow_http":true, "token":token, "device_id":"device"})).unwrap();
        assert!(Session::load(&path).is_err());
    }

    #[test]
    fn enter_confirms_without_waiting_for_eof() {
        struct LiveInput {
            line: Option<Vec<u8>>,
        }
        impl Read for LiveInput {
            fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
                match self.line.take() {
                    Some(line) => {
                        bytes[..line.len()].copy_from_slice(&line);
                        Ok(line.len())
                    }
                    None => Err(io::Error::other("não deveria esperar EOF depois de Enter")),
                }
            }
        }
        let code = "a".repeat(64);
        let input = LiveInput {
            line: Some(format!("{code}\n").into_bytes()),
        };
        assert_eq!(read_code(input).unwrap(), code);
    }
    #[test]
    fn pipe_eof_crlf_limits_and_invalid_input_are_handled_without_echo() {
        let code = "a".repeat(64);
        for text in [
            code.clone(),
            format!("{code}\r\n"),
            format!("{code}\nignored"),
        ] {
            assert_eq!(read_code(text.as_bytes()).unwrap(), code);
        }
        for text in [
            String::new(),
            "\n".into(),
            "private-marker\n".into(),
            "a".repeat(129),
        ] {
            let error = read_code(text.as_bytes()).unwrap_err();
            assert!(!error.to_string().contains("private-marker"));
        }
    }
}

pub mod watch;
