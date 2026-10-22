//! Configuração TOML e regras locais de autorização. Veja decisão 0003.
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};

use serde::Deserialize;

use crate::storage::validate_path;

const MAX_CONFIG: u64 = 1024 * 1024;
const MAX_USERS: usize = 1_000;
const MAX_SHARES: usize = 1_000;
const DEFAULT_UPLOAD_LIMIT: u64 = 1024 * 1024 * 1024;
// Parser padrão, isolado do diretório atual e de módulos/configurações do usuário.
const TOML_TO_JSON: &str = r#"
import sys, json, tomllib
try:
    config = tomllib.loads(sys.stdin.buffer.read().decode('utf-8'))
    output = json.dumps(config, ensure_ascii=True, allow_nan=False)
except Exception:
    sys.exit(1)
sys.stdout.write(output)
"#;

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

/// `Write` inclui leitura, criação, alteração e exclusão.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Permission {
    Read,
    Write,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    Read,
    Write,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct User {
    #[serde(default)]
    enabled: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Share {
    #[serde(default)]
    permissions: BTreeMap<String, Permission>,
}

fn default_upload_limit() -> u64 {
    DEFAULT_UPLOAD_LIMIT
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ServerConfig {
    storage_root: PathBuf,
    #[serde(default = "default_upload_limit")]
    max_upload_bytes: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    version: u32,
    server: ServerConfig,
    #[serde(default)]
    users: BTreeMap<String, User>,
    #[serde(default)]
    shares: BTreeMap<String, Share>,
}

/// Configuração validada; só pode ser construída pelo carregador.
#[derive(Debug)]
pub struct Config {
    document: Document,
}

impl Config {
    /// Carrega um arquivo regular de até 1 MiB. Não abre o armazenamento.
    pub fn load(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        if !fs::symlink_metadata(path)?.file_type().is_file() {
            return Err(invalid("configuração deve ser arquivo regular, sem link"));
        }
        let mut bytes = Vec::new();
        OpenOptions::new()
            .read(true)
            .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
            .open(path)?
            .take(MAX_CONFIG + 1)
            .read_to_end(&mut bytes)?;
        Self::parse(&bytes)
    }

    /// Converte TOML e valida integralmente o esquema antes de retornar.
    pub fn parse(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() as u64 > MAX_CONFIG {
            return Err(invalid("configuração excede limite de 1 MiB"));
        }
        let mut child = Command::new("/usr/bin/python3")
            .args(["-I", "-c", TOML_TO_JSON])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| {
                invalid("parser TOML indisponível; requer /usr/bin/python3 com tomllib")
            })?;
        let input_result = match child.stdin.take() {
            Some(mut stdin) => stdin.write_all(bytes),
            None => Err(io::Error::other("entrada do parser TOML indisponível")),
        };
        // Aguarda inclusive em erro de escrita, evitando deixar processo filho.
        let output = child.wait_with_output()?;
        if !output.status.success() || input_result.is_err() {
            return Err(invalid("TOML inválido ou parser TOML indisponível"));
        }
        let document: Document = serde_json::from_slice(&output.stdout)
            .map_err(|_| invalid("esquema de configuração inválido; confira campos e tipos"))?;
        let config = Self { document };
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> io::Result<()> {
        if self.document.version != 1 {
            return Err(invalid("versão de configuração não suportada"));
        }
        if !self.document.server.storage_root.is_absolute()
            || self
                .document
                .server
                .storage_root
                .components()
                .any(|c| matches!(c, Component::ParentDir))
            || self.document.server.storage_root == Path::new("/")
        {
            return Err(invalid(
                "storage_root deve ser absoluto, sem .. e diferente de /",
            ));
        }
        if self
            .document
            .server
            .storage_root
            .as_os_str()
            .as_encoded_bytes()
            .contains(&0)
        {
            return Err(invalid("storage_root contém caractere inválido"));
        }
        if self.document.server.max_upload_bytes == 0
            || self.document.server.max_upload_bytes > (1u64 << 40)
        {
            return Err(invalid("max_upload_bytes deve estar entre 1 byte e 1 TiB"));
        }
        if self.document.users.len() > MAX_USERS || self.document.shares.len() > MAX_SHARES {
            return Err(invalid(
                "configuração excede limite de usuários ou compartilhamentos",
            ));
        }
        for name in self.document.users.keys() {
            if name.is_empty()
                || name.len() > 64
                || !name
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"_-".contains(&c))
            {
                return Err(invalid("identificador de usuário inválido"));
            }
        }
        for (name, share) in &self.document.shares {
            validate_path(name)?;
            if name.contains('/') {
                return Err(invalid("compartilhamento deve ser de primeiro nível"));
            }
            if share
                .permissions
                .keys()
                .any(|user| !self.document.users.contains_key(user))
            {
                return Err(invalid("permissão referencia usuário inexistente"));
            }
        }
        Ok(())
    }

    pub fn storage_root(&self) -> &Path {
        &self.document.server.storage_root
    }

    pub fn max_upload_bytes(&self) -> u64 {
        self.document.server.max_upload_bytes
    }

    pub fn user_count(&self) -> usize {
        self.document.users.len()
    }

    pub fn share_count(&self) -> usize {
        self.document.shares.len()
    }

    pub fn user_enabled(&self, user: &str) -> bool {
        self.document.users.get(user).is_some_and(|u| u.enabled)
    }

    pub fn allows(&self, user: &str, share: &str, operation: Operation) -> bool {
        if !self.document.users.get(user).is_some_and(|u| u.enabled) {
            return false;
        }
        match self
            .document
            .shares
            .get(share)
            .and_then(|s| s.permissions.get(user))
        {
            Some(Permission::Write) => true,
            Some(Permission::Read) => operation == Operation::Read,
            None => false,
        }
    }

    pub(crate) fn readable_shares<'a>(&'a self, user: &'a str) -> impl Iterator<Item = &'a str> {
        self.document
            .shares
            .keys()
            .filter(move |share| self.allows(user, share, Operation::Read))
            .map(String::as_str)
    }
}
