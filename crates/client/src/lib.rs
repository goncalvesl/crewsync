//! Cliente Linux de piloto: ativação e reconciliação por revisão-base.
pub mod sync;
pub mod tree;
use crewsync_protocol::{Operation, Request, disk, invalid, transport};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{self, Read};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub url: String,
    pub ca: PathBuf,
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
        Ok(session)
    }
    pub fn request(&self, operation: Operation) -> Request {
        Request {
            token: Some(self.token.clone()),
            operation,
        }
    }
    pub fn call<T: serde::de::DeserializeOwned>(&self, operation: Operation) -> io::Result<T> {
        transport::json(&self.url, &self.ca, self.request(operation))
    }
}

pub fn activate(url: &str, ca: &Path, output: &Path, name: &str, code: &str) -> io::Result<()> {
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
        let ca = std::path::absolute(ca)?;
        let response: Activation = transport::json(
            url,
            &ca,
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

pub fn read_code(mut input: impl Read) -> io::Result<String> {
    let mut data = String::new();
    input.by_ref().take(128).read_to_string(&mut data)?;
    let code = data.trim();
    disk::validate_hash(code)?;
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
        transport::exchange(
            &self.url,
            &self.ca,
            self.request(operation),
            input,
            size,
            output,
            limit,
        )
    }
}
