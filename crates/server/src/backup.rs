//! Snapshots offline em diretório novo, validados antes do marcador de conclusão.
use crate::{auth::AuthStore, config::Config, storage::Store};
use crewsync_protocol::{disk, invalid};
use rustix::fs::{FlockOperation, flock};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Component, Path, PathBuf};

fn destination(path: &Path, source: &Path) -> io::Result<PathBuf> {
    let absolute = std::path::absolute(path)?;
    if absolute
        .components()
        .any(|c| matches!(c, Component::ParentDir))
        || absolute.starts_with(source)
    {
        return Err(invalid("destino deve ser novo, fora da origem e sem .."));
    }
    let parent = absolute
        .parent()
        .ok_or_else(|| invalid("destino inválido"))?;
    if fs::canonicalize(parent)? != parent {
        return Err(invalid("pai do destino deve existir e não conter links"));
    }
    fs::DirBuilder::new().mode(0o700).create(&absolute)?;
    File::open(parent)?.sync_all()?;
    Ok(absolute)
}
fn copy_tree(from: &Path, to: &Path) -> io::Result<()> {
    fs::DirBuilder::new().mode(0o700).create(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else if entry.file_type()?.is_file() {
            let mut input = disk::regular_file(&entry.path())?;
            let mut output = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(target)?;
            io::copy(&mut input, &mut output)?;
            output.sync_all()?;
        } else {
            return Err(invalid("backup recusa links e arquivos especiais"));
        }
    }
    File::open(to)?.sync_all()
}
fn auth_lock(root: &Path) -> io::Result<File> {
    AuthStore::open(root)?;
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
        .open(root.join("auth/lock"))?;
    if !file.metadata()?.is_file() {
        return Err(invalid("lock inválido"));
    }
    flock(&file, FlockOperation::NonBlockingLockExclusive)?;
    Ok(file)
}
fn validate(root: &Path) -> io::Result<()> {
    let _store = Store::open(root)?;
    AuthStore::open(root)?.devices()?;
    Ok(())
}
pub fn create(config: &Path, target: &Path) -> io::Result<()> {
    let configuration = Config::load(config)?;
    let root = configuration.storage_root();
    let _store = Store::open(root)?;
    let _auth = auth_lock(root)?;
    let target = destination(target, root)?;
    copy_tree(root, &target.join("store"))?;
    validate(&target.join("store"))?;
    let mut input = disk::regular_file(config)?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(target.join("server.toml"))?;
    io::copy(&mut input, &mut output)?;
    output.flush()?;
    output.sync_all()?;
    disk::atomic_json(
        &target.join("backup.json"),
        &serde_json::json!({"format":1,"version":env!("CARGO_PKG_VERSION")}),
    )
}
pub fn restore(source: &Path, target: &Path) -> io::Result<()> {
    let source = fs::canonicalize(source)?;
    let record: serde_json::Value = disk::read_json(&source.join("backup.json"), 4096)?;
    if record["format"] != 1 {
        return Err(invalid("backup incompleto ou não suportado"));
    }
    let root = source.join("store");
    let _store = Store::open(&root)?;
    let _auth = auth_lock(&root)?;
    let target = destination(target, &source)?;
    // target já existe; copiar para subdiretório evita qualquer sobrescrita.
    copy_tree(&root, &target.join("store"))?;
    validate(&target.join("store"))?;
    disk::atomic_json(
        &target.join("restored.json"),
        &serde_json::json!({"format":1}),
    )?;
    Ok(())
}
