//! Configuração TOML e regras locais de autorização. Veja decisão 0003.
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Read};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Component, Path, PathBuf};

use serde::Deserialize;

use crate::storage::validate_path;

const MAX_CONFIG: u64 = 1024 * 1024;
const MAX_USERS: usize = 1_000;
const MAX_SHARES: usize = 1_000;
const DEFAULT_UPLOAD_LIMIT: u64 = 1024 * 1024 * 1024;
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
    #[serde(default)]
    retention: crate::storage::Retention,
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
        let source = std::str::from_utf8(bytes).map_err(|_| invalid("TOML deve ser UTF-8"))?;
        let document: Document = toml::from_str(source)
            .map_err(|_| invalid("TOML ou esquema inválido; confira campos e tipos"))?;
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
        self.document.server.retention.validate()?;
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

    pub fn retention(&self) -> crate::storage::Retention {
        self.document.server.retention
    }

    pub fn users(&self) -> impl Iterator<Item = (&str, bool)> {
        self.document
            .users
            .iter()
            .map(|(name, user)| (name.as_str(), user.enabled))
    }

    pub fn shares(&self) -> impl Iterator<Item = (&str, &BTreeMap<String, Permission>)> {
        self.document
            .shares
            .iter()
            .map(|(name, share)| (name.as_str(), &share.permissions))
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
