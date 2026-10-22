//! Edição validada, com revisão-base e publicação atômica do TOML.
use crate::config::Config;
use crewsync_protocol::{disk, invalid};
use rustix::fs::{FlockOperation, flock};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use toml_edit::{DocumentMut, Item, Table, TableLike, value};

pub struct Snapshot {
    pub config: Config,
    pub revision: String,
    source: String,
}
impl Snapshot {
    pub fn load(path: &Path) -> io::Result<Self> {
        let mut source = String::new();
        disk::regular_file(path)?
            .take(1024 * 1024 + 1)
            .read_to_string(&mut source)?;
        let config = Config::parse(source.as_bytes())?;
        Ok(Self {
            config,
            revision: disk::hash_secret(&source)?,
            source,
        })
    }
}

pub enum Change<'a> {
    User {
        name: &'a str,
        enabled: bool,
    },
    Share {
        name: &'a str,
    },
    Permission {
        share: &'a str,
        user: &'a str,
        access: &'a str,
    },
}
fn table<'a>(parent: &'a mut dyn TableLike, name: &str) -> io::Result<&'a mut dyn TableLike> {
    let item = parent.entry(name).or_insert(Item::Table(Table::new()));
    if item.is_inline_table() {
        *item = Item::Table(
            item.clone()
                .into_table()
                .map_err(|_| invalid("tabela inválida"))?,
        );
    }
    item.as_table_like_mut()
        .ok_or_else(|| invalid("tabela inválida"))
}

pub fn update(path: &Path, revision: &str, root: &Path, change: Change<'_>) -> io::Result<()> {
    let mut lock_name = path.as_os_str().to_os_string();
    lock_name.push(".admin.lock");
    let lock_path = PathBuf::from(lock_name);
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
        .open(lock_path)?;
    if !lock.metadata()?.is_file() {
        return Err(invalid("lock inválido"));
    }
    flock(&lock, FlockOperation::NonBlockingLockExclusive)?;
    let current = Snapshot::load(path)?;
    if current.revision != revision || current.config.storage_root() != root {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "configuração mudou; recarregue a página",
        ));
    }
    let mut document: DocumentMut = current
        .source
        .parse()
        .map_err(|_| invalid("TOML inválido"))?;
    match change {
        Change::User { name, enabled } => {
            table(table(document.as_table_mut(), "users")?, name)?
                .insert("enabled", value(enabled));
        }
        Change::Share { name } => {
            let shares = table(document.as_table_mut(), "shares")?;
            if shares.contains_key(name) {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "pasta já cadastrada",
                ));
            }
            table(table(shares, name)?, "permissions")?;
        }
        Change::Permission {
            share,
            user,
            access,
        } => {
            if !current.config.users().any(|(name, _)| name == user)
                || !current.config.shares().any(|(name, _)| name == share)
            {
                return Err(invalid("usuário ou pasta inexistente"));
            }
            let permissions = table(
                table(table(document.as_table_mut(), "shares")?, share)?,
                "permissions",
            )?;
            match access {
                "none" => {
                    permissions.remove(user);
                }
                "read" | "write" => {
                    permissions.insert(user, value(access));
                }
                _ => return Err(invalid("permissão inválida")),
            }
        }
    }
    let updated = document.to_string();
    Config::parse(updated.as_bytes())?;
    if Snapshot::load(path)?.revision != revision {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "configuração mudou; recarregue a página",
        ));
    }
    let parent = path
        .parent()
        .ok_or_else(|| invalid("configuração sem diretório"))?;
    let temporary = parent.join(format!(".crewsync-config-{}", disk::random_hex(16)?));
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temporary)?;
        file.write_all(updated.as_bytes())?;
        file.sync_all()?;
        fs::rename(&temporary, path)?;
        File::open(parent)?.sync_all()
    })();
    let _ = fs::remove_file(temporary);
    result
}
