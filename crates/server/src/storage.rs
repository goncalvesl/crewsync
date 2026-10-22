//! Objetos raw e manifestos duráveis. Consulte docs/decisions/0002-storage.md.
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use rustix::fs::{FlockOperation, flock};

const MAX_MANIFEST: u64 = 16 * 1024 * 1024;
const MAX_ENTRIES: usize = 100_000;

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

pub use crewsync_protocol::validate_path;

fn validate_share(share: &str) -> io::Result<()> {
    validate_path(share)?;
    if share.contains('/') {
        return Err(invalid("compartilhamento deve ter um único componente"));
    }
    Ok(())
}

fn validate_hash(hash: &str) -> io::Result<()> {
    if hash.len() != 64
        || !hash
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    {
        return Err(invalid("SHA-256 inválido"));
    }
    Ok(())
}

pub use crewsync_protocol::{Content, Entry, Manifest};

/// Instância exclusiva. O diretório deve pertencer somente ao serviço.
pub struct Store {
    root: PathBuf,
    _lock: File,
    #[cfg(test)]
    fail_at: Option<&'static str>,
}

impl Drop for Store {
    fn drop(&mut self) {
        // Libera explicitamente: um processo criado por outra thread pode manter
        // uma cópia transitória do descritor entre fork e exec.
        let _ = flock(&self._lock, FlockOperation::Unlock);
    }
}

fn regular(path: &Path) -> io::Result<()> {
    if !fs::symlink_metadata(path)?.file_type().is_file() {
        return Err(invalid("arquivo interno não é regular"));
    }
    Ok(())
}

fn sync_dir(path: &Path) -> io::Result<()> {
    File::open(path)?.sync_all()
}

fn private_dir(path: &Path) -> io::Result<()> {
    match fs::DirBuilder::new().mode(0o700).create(path) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
        Err(e) => return Err(e),
    }
    if !fs::symlink_metadata(path)?.file_type().is_dir() {
        return Err(invalid("diretório interno inválido"));
    }
    if fs::metadata(path)?.permissions().mode() & 0o077 != 0 {
        return Err(invalid(
            "diretório de armazenamento deve ter permissão 0700",
        ));
    }
    // Também cobre diretórios criados por uma tentativa anterior interrompida
    // antes de sincronizar o pai.
    if let Some(parent) = path.parent() {
        sync_dir(parent)?;
    }
    Ok(())
}

fn hash_file(path: &Path) -> io::Result<String> {
    regular(path)?;
    let output = Command::new("/usr/bin/sha256sum")
        .stdin(Stdio::from(File::open(path)?))
        .output()?;
    if !output.status.success() {
        return Err(invalid("falha ao calcular SHA-256"));
    }
    let hash = std::str::from_utf8(
        output
            .stdout
            .get(..64)
            .ok_or_else(|| invalid("saída SHA-256 inválida"))?,
    )
    .map_err(|_| invalid("saída SHA-256 inválida"))?
    .to_owned();
    validate_hash(&hash)?;
    Ok(hash)
}

impl Store {
    #[cfg(test)]
    fn checkpoint(&mut self, point: &str) -> io::Result<()> {
        if self.fail_at == Some(point) {
            self.fail_at = None;
            return Err(io::Error::new(
                io::ErrorKind::StorageFull,
                "falha de disco simulada",
            ));
        }
        Ok(())
    }

    pub fn open(root: impl AsRef<Path>) -> io::Result<Self> {
        // Recusa links inclusive nos ancestrais já existentes.
        let absolute = std::path::absolute(root)?;
        let mut component_path = PathBuf::new();
        for component in absolute.components() {
            if matches!(component, std::path::Component::ParentDir) {
                return Err(invalid("raiz contém .."));
            }
            component_path.push(component);
            if let Ok(metadata) = fs::symlink_metadata(&component_path)
                && metadata.file_type().is_symlink()
            {
                return Err(invalid("link na raiz do armazenamento"));
            }
        }
        private_dir(&absolute)?;
        let lock_path = absolute.join("lock");
        if lock_path.try_exists()? {
            regular(&lock_path)?;
        }
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
            .open(lock_path)?;
        flock(&lock, FlockOperation::NonBlockingLockExclusive)?;
        for name in ["objects", "manifests", "tmp"] {
            private_dir(&absolute.join(name))?;
        }
        let store = Self {
            root: absolute,
            _lock: lock,
            #[cfg(test)]
            fail_at: None,
        };
        store.recover()?;
        Ok(store)
    }

    fn recover(&self) -> io::Result<()> {
        for item in fs::read_dir(self.root.join("tmp"))? {
            let path = item?.path();
            regular(&path)?;
            fs::remove_file(path)?;
        }
        sync_dir(&self.root.join("tmp"))?;
        for item in fs::read_dir(self.root.join("objects"))? {
            let path = item?.path();
            regular(&path)?;
            validate_hash(
                path.file_name()
                    .and_then(|s| s.to_str())
                    .ok_or_else(|| invalid("nome de objeto inválido"))?,
            )?;
        }
        for item in fs::read_dir(self.root.join("manifests"))? {
            let path = item?.path();
            regular(&path)?;
            let share = path
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(|| invalid("nome de manifesto inválido"))?;
            self.manifest(share)?;
        }
        Ok(())
    }

    /// Ausência de manifesto representa compartilhamento vazio (não concede acesso).
    pub fn manifest(&self, share: &str) -> io::Result<Manifest> {
        validate_share(share)?;
        let path = self.root.join("manifests").join(share);
        match fs::symlink_metadata(&path) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Manifest::default()),
            Err(e) => return Err(e),
            Ok(_) => regular(&path)?,
        }
        let mut data = Vec::new();
        File::open(path)?
            .take(MAX_MANIFEST + 1)
            .read_to_end(&mut data)?;
        if data.len() as u64 > MAX_MANIFEST {
            return Err(invalid("manifesto excede limite"));
        }
        let manifest: Manifest =
            serde_json::from_slice(&data).map_err(|_| invalid("manifesto JSON inválido"))?;
        self.validate_manifest(&manifest)?;
        Ok(manifest)
    }

    fn validate_manifest(&self, manifest: &Manifest) -> io::Result<()> {
        if manifest.format != 1 || manifest.entries.len() > MAX_ENTRIES {
            return Err(invalid("formato ou tamanho de manifesto inválido"));
        }
        let max_revision = manifest
            .entries
            .values()
            .map(|e| e.revision)
            .max()
            .unwrap_or(0);
        if max_revision != manifest.revision {
            return Err(invalid("revisão global inconsistente"));
        }
        for (path, entry) in &manifest.entries {
            validate_path(path)?;
            if entry.revision == 0 || entry.revision > manifest.revision {
                return Err(invalid("revisão de entrada inválida"));
            }
            if entry.content != Content::Deleted {
                let mut parent = path.as_str();
                while let Some((ancestor, _)) = parent.rsplit_once('/') {
                    if !matches!(
                        manifest.entries.get(ancestor).map(|e| &e.content),
                        Some(Content::Directory)
                    ) {
                        return Err(invalid("diretório pai ausente"));
                    }
                    parent = ancestor;
                }
            }
            if let Content::File { hash, size } = &entry.content {
                self.verify_object(hash, *size)?;
            }
        }
        Ok(())
    }

    fn verify_object(&self, hash: &str, size: u64) -> io::Result<()> {
        validate_hash(hash)?;
        let path = self.root.join("objects").join(hash);
        regular(&path)?;
        if fs::metadata(&path)?.len() != size || hash_file(&path)? != hash {
            return Err(invalid("objeto corrompido"));
        }
        Ok(())
    }

    /// Recebe em streaming, exige SHA-256 esperado e limita bytes antes de publicar.
    pub fn put_object(
        &mut self,
        mut reader: impl Read,
        expected: &str,
        limit: u64,
    ) -> io::Result<u64> {
        validate_hash(expected)?;
        let temp = self.root.join("tmp/object");
        let result = (|| {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&temp)?;
            let mut buffer = [0u8; 64 * 1024];
            let mut size = 0u64;
            loop {
                let count = match reader.read(&mut buffer) {
                    Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    other => other?,
                };
                if count == 0 {
                    break;
                }
                size = size
                    .checked_add(count as u64)
                    .ok_or_else(|| invalid("objeto longo demais"))?;
                if size > limit {
                    return Err(invalid("objeto excede limite"));
                }
                file.write_all(&buffer[..count])?;
            }
            #[cfg(test)]
            self.checkpoint("object_written")?;
            file.sync_all()?;
            #[cfg(test)]
            self.checkpoint("object_synced")?;
            if hash_file(&temp)? != expected {
                return Err(invalid("SHA-256 recebido não corresponde ao conteúdo"));
            }
            let target = self.root.join("objects").join(expected);
            match fs::symlink_metadata(&target) {
                Ok(_) => {
                    self.verify_object(expected, size)?;
                    fs::remove_file(&temp)?;
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound => fs::rename(&temp, target)?,
                Err(e) => return Err(e),
            }
            #[cfg(test)]
            self.checkpoint("object_published")?;
            sync_dir(&self.root.join("objects"))?;
            #[cfg(test)]
            self.checkpoint("object_durable")?;
            Ok(size)
        })();
        let _ = fs::remove_file(temp);
        result
    }

    /// Leitura validada do objeto; não carrega o conteúdo inteiro em memória.
    pub fn object(&self, hash: &str, size: u64) -> io::Result<File> {
        self.verify_object(hash, size)?;
        File::open(self.root.join("objects").join(hash))
    }

    /// Publica uma alteração. Revisão-base obsoleta retorna AlreadyExists.
    pub fn apply(
        &mut self,
        share: &str,
        path: &str,
        base: u64,
        content: Content,
    ) -> io::Result<u64> {
        validate_path(path)?;
        let mut manifest = self.manifest(share)?;
        let current = manifest.entries.get(path);
        if current.map_or(0, |e| e.revision) != base {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "conflito de revisão; preserve a versão concorrente",
            ));
        }
        if content == Content::Deleted && current.is_none_or(|e| e.content == Content::Deleted) {
            return Err(invalid("exclusão exige entrada existente"));
        }
        if content != Content::Directory {
            let prefix = format!("{path}/");
            if manifest
                .entries
                .iter()
                .any(|(p, e)| p.starts_with(&prefix) && e.content != Content::Deleted)
            {
                return Err(invalid("diretório ainda tem descendentes"));
            }
        }
        manifest.revision = manifest
            .revision
            .checked_add(1)
            .ok_or_else(|| invalid("revisões esgotadas"))?;
        manifest.entries.insert(
            path.to_owned(),
            Entry {
                revision: manifest.revision,
                content,
            },
        );
        self.validate_manifest(&manifest)?;
        let bytes =
            serde_json::to_vec(&manifest).map_err(|_| invalid("falha ao serializar manifesto"))?;
        if bytes.len() as u64 > MAX_MANIFEST {
            return Err(invalid("manifesto excede limite"));
        }
        let temp = self.root.join("tmp/manifest");
        let result = (|| {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&temp)?;
            file.write_all(&bytes)?;
            #[cfg(test)]
            self.checkpoint("manifest_written")?;
            file.sync_all()?;
            #[cfg(test)]
            self.checkpoint("manifest_synced")?;
            fs::rename(&temp, self.root.join("manifests").join(share))?;
            #[cfg(test)]
            self.checkpoint("manifest_published")?;
            sync_dir(&self.root.join("manifests"))?;
            #[cfg(test)]
            self.checkpoint("manifest_durable")?;
            Ok(manifest.revision)
        })();
        let _ = fs::remove_file(temp);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const ABC: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

    #[test]
    fn publication_failures_recover_old_or_complete_new_state() {
        for stage in [
            "object_written",
            "object_synced",
            "object_published",
            "object_durable",
            "manifest_written",
            "manifest_synced",
            "manifest_published",
            "manifest_durable",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let root = dir.path().join("store");
            let mut store = Store::open(&root).unwrap();
            store.apply("share", "old", 0, Content::Directory).unwrap();
            store.fail_at = Some(stage);
            let result = if stage.starts_with("object") {
                store.put_object(&b"abc"[..], ABC, 3)
            } else {
                store.put_object(&b"abc"[..], ABC, 3).unwrap();
                store.apply(
                    "share",
                    "new",
                    0,
                    Content::File {
                        hash: ABC.into(),
                        size: 3,
                    },
                )
            };
            assert_eq!(
                result.unwrap_err().kind(),
                io::ErrorKind::StorageFull,
                "{stage}"
            );
            drop(store);
            let store = Store::open(&root).unwrap();
            let manifest = store.manifest("share").unwrap();
            assert!(manifest.entries.contains_key("old"));
            let published = matches!(stage, "manifest_published" | "manifest_durable");
            assert_eq!(manifest.entries.contains_key("new"), published, "{stage}");
            assert_eq!(manifest.revision, if published { 2 } else { 1 });
            assert_eq!(fs::read_dir(root.join("tmp")).unwrap().count(), 0);
        }
    }
}
