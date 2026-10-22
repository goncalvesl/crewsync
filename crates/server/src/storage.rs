//! Objetos raw e manifestos duráveis. Consulte docs/decisions/0002-storage.md.
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

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

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct History {
    format: u32,
    path: String,
    entries: Vec<Entry>,
    #[serde(default)]
    archived_at: BTreeMap<u64, u64>,
}

/// Política aplicada somente pela manutenção administrativa com lock exclusivo.
#[derive(Clone, Copy, Debug, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Retention {
    pub history_days: u64,
    pub previous_versions: usize,
    pub trash_days: u64,
}
impl Default for Retention {
    fn default() -> Self {
        Self {
            history_days: 30,
            previous_versions: 10,
            trash_days: 30,
        }
    }
}
impl Retention {
    pub fn validate(&self) -> io::Result<()> {
        if !(1..=36500).contains(&self.history_days)
            || !(1..=1000).contains(&self.previous_versions)
            || !(1..=36500).contains(&self.trash_days)
        {
            return Err(invalid(
                "retenção inválida: dias 1..36500 e versões 1..1000",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Default, serde::Serialize)]
pub struct Maintenance {
    pub objects: u64,
    pub object_bytes: u64,
    pub reclaimable_objects: u64,
    pub reclaimable_bytes: u64,
    pub pruned_versions: u64,
    pub applied: bool,
}
fn now() -> io::Result<u64> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(io::Error::other)
}

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
    crewsync_protocol::disk::hash_file(&File::open(path)?)
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
        for name in ["objects", "manifests", "tmp", "history"] {
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
        for share in fs::read_dir(self.root.join("history"))? {
            let share = share?;
            let name = share
                .file_name()
                .into_string()
                .map_err(|_| invalid("histórico inválido"))?;
            validate_share(&name)?;
            private_dir(&share.path())?;
            let manifest = self.manifest(&name)?;
            for item in fs::read_dir(share.path())? {
                let item = item?;
                let history = self.read_history(&item.path())?;
                let expected = crewsync_protocol::disk::hash_secret(&history.path)?;
                Self::validate_history_current(&history, &manifest)?;
                if item.file_name() != expected.as_str() {
                    return Err(invalid("histórico inconsistente"));
                }
            }
        }
        Ok(())
    }

    fn validate_history_current(history: &History, manifest: &Manifest) -> io::Result<()> {
        let current = manifest
            .entries
            .get(&history.path)
            .ok_or_else(|| invalid("caminho histórico ausente do manifesto"))?;
        if history.entries.last().is_none_or(|entry| {
            entry.revision > current.revision
                || (entry.revision == current.revision && entry != current)
        }) {
            return Err(invalid("histórico inconsistente com versão atual"));
        }
        Ok(())
    }

    fn read_history(&self, path: &Path) -> io::Result<History> {
        let history: History = crewsync_protocol::disk::read_json(path, MAX_MANIFEST)?;
        validate_path(&history.path)?;
        if !matches!(history.format, 1 | 2) || history.entries.is_empty() {
            return Err(invalid("formato de histórico inválido"));
        }
        if history.format == 2
            && (history.archived_at.len() != history.entries.len()
                || history
                    .entries
                    .iter()
                    .any(|e| history.archived_at.get(&e.revision).is_none_or(|t| *t == 0)))
        {
            return Err(invalid("datas de histórico inconsistentes"));
        }
        let mut previous = 0;
        for entry in &history.entries {
            if entry.revision <= previous {
                return Err(invalid("revisões do histórico inválidas"));
            }
            previous = entry.revision;
            if let Content::File { hash, size } = &entry.content {
                self.verify_object(hash, *size)?;
            }
        }
        Ok(history)
    }

    fn history_path(&self, share: &str, path: &str) -> io::Result<PathBuf> {
        validate_share(share)?;
        validate_path(path)?;
        Ok(self
            .root
            .join("history")
            .join(share)
            .join(crewsync_protocol::disk::hash_secret(path)?))
    }

    /// Versões disponíveis, incluindo a atual, em ordem de revisão.
    pub fn history(&self, share: &str, path: &str) -> io::Result<Vec<Entry>> {
        let file = self.history_path(share, path)?;
        let manifest = self.manifest(share)?;
        let mut entries = match fs::symlink_metadata(&file) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => Vec::new(),
            Err(e) => return Err(e),
            Ok(_) => {
                let history = self.read_history(&file)?;
                Self::validate_history_current(&history, &manifest)?;
                if history.path != path {
                    return Err(invalid("caminho do histórico inconsistente"));
                }
                history.entries
            }
        };
        if let Some(current) = manifest.entries.get(path) {
            entries.retain(|e| e.revision < current.revision);
            entries.push(current.clone());
        }
        if entries.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "histórico não encontrado",
            ));
        }
        Ok(entries)
    }

    /// Restauração individual; pais devem existir e a revisão-base deve ser atual.
    pub fn restore(
        &mut self,
        share: &str,
        path: &str,
        revision: u64,
        base: u64,
    ) -> io::Result<u64> {
        let entry = self
            .history(share, path)?
            .into_iter()
            .find(|e| e.revision == revision)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "versão não encontrada"))?;
        if entry.content == Content::Deleted {
            return Err(invalid("escolha uma versão anterior à exclusão"));
        }
        self.apply(share, path, base, entry.content)
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

    /// Sem transferências em curso: Store mantém o lock exclusivo por toda a execução.
    pub fn maintain(&mut self, policy: Retention, apply: bool) -> io::Result<Maintenance> {
        self.maintain_at(policy, apply, now()?)
    }

    fn maintain_at(
        &mut self,
        policy: Retention,
        apply: bool,
        timestamp: u64,
    ) -> io::Result<Maintenance> {
        policy.validate()?;
        let mut report = Maintenance {
            applied: apply,
            ..Maintenance::default()
        };
        let mut references = BTreeSet::new();
        for item in fs::read_dir(self.root.join("manifests"))? {
            let item = item?;
            let share = item
                .file_name()
                .into_string()
                .map_err(|_| invalid("compartilhamento inválido"))?;
            for entry in self.manifest(&share)?.entries.values() {
                if let Content::File { hash, .. } = &entry.content {
                    references.insert(hash.clone());
                }
            }
        }
        for share in fs::read_dir(self.root.join("history"))? {
            let share = share?;
            let name = share
                .file_name()
                .into_string()
                .map_err(|_| invalid("compartilhamento inválido"))?;
            let manifest = self.manifest(&name)?;
            for item in fs::read_dir(share.path())? {
                let path = item?.path();
                let mut history = self.read_history(&path)?;
                Self::validate_history_current(&history, &manifest)?;
                let current = &manifest.entries[&history.path];
                let mut kept = Vec::new();
                let mut count = 0;
                let last_file = history
                    .entries
                    .iter()
                    .rev()
                    .find(|e| matches!(e.content, Content::File { .. }))
                    .map(|e| e.revision);
                for entry in history.entries.iter().rev() {
                    let archived = *history
                        .archived_at
                        .entry(entry.revision)
                        .or_insert(timestamp);
                    let age = timestamp.saturating_sub(archived);
                    let trash = current.content == Content::Deleted
                        && Some(entry.revision) == last_file
                        && age < policy.trash_days * 86400;
                    let normal =
                        count < policy.previous_versions && age < policy.history_days * 86400;
                    if entry.revision < current.revision && (normal || trash) {
                        kept.push(entry.clone());
                        count += 1;
                    }
                }
                kept.reverse();
                report.pruned_versions += (history.entries.len() - kept.len()) as u64;
                history.entries = kept;
                history
                    .archived_at
                    .retain(|revision, _| history.entries.iter().any(|e| e.revision == *revision));
                history.format = 2;
                for entry in &history.entries {
                    if let Content::File { hash, .. } = &entry.content {
                        references.insert(hash.clone());
                    }
                }
                if apply {
                    if history.entries.is_empty() {
                        fs::remove_file(&path)?;
                        sync_dir(&share.path())?;
                    } else {
                        let temp = self.root.join("tmp/maintenance");
                        let result = (|| {
                            let mut file = OpenOptions::new()
                                .write(true)
                                .create_new(true)
                                .mode(0o600)
                                .open(&temp)?;
                            serde_json::to_writer(&mut file, &history).map_err(io::Error::other)?;
                            #[cfg(test)]
                            self.checkpoint("maintenance_written")?;
                            file.sync_all()?;
                            fs::rename(&temp, &path)?;
                            sync_dir(&share.path())
                        })();
                        let _ = fs::remove_file(&temp);
                        result?;
                    }
                }
            }
        }
        // Só iniciar remoção depois de todos os metadados podados estarem duráveis.
        #[cfg(test)]
        if apply {
            self.checkpoint("maintenance_metadata_durable")?;
        }
        for item in fs::read_dir(self.root.join("objects"))? {
            let item = item?;
            let name = item
                .file_name()
                .into_string()
                .map_err(|_| invalid("objeto inválido"))?;
            validate_hash(&name)?;
            regular(&item.path())?;
            let size = item.metadata()?.len();
            report.objects += 1;
            report.object_bytes += size;
            if !references.contains(&name) {
                report.reclaimable_objects += 1;
                report.reclaimable_bytes += size;
                if apply {
                    fs::remove_file(item.path())?;
                    #[cfg(test)]
                    self.checkpoint("maintenance_object_removed")?;
                }
            }
        }
        if apply {
            sync_dir(&self.root.join("objects"))?;
        }
        Ok(report)
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
        if base != 0 {
            let entries = self.history(share, path)?;
            let target = self.history_path(share, path)?;
            let mut archived_at = if target.try_exists()? {
                self.read_history(&target)?.archived_at
            } else {
                BTreeMap::new()
            };
            let timestamp = now()?;
            for entry in &entries {
                archived_at.entry(entry.revision).or_insert(timestamp);
            }
            let history = History {
                format: 2,
                path: path.to_owned(),
                entries,
                archived_at,
            };
            let bytes = serde_json::to_vec(&history).map_err(io::Error::other)?;
            if bytes.len() as u64 > MAX_MANIFEST - 1024 {
                return Err(invalid("histórico excede limite; alteração recusada"));
            }
            let target = self.history_path(share, path)?;
            private_dir(&self.root.join("history").join(share))?;
            let temp = self.root.join("tmp/history");
            let result = (|| {
                let mut file = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .mode(0o600)
                    .open(&temp)?;
                file.write_all(&bytes)?;
                #[cfg(test)]
                self.checkpoint("history_written")?;
                file.sync_all()?;
                fs::rename(&temp, target)?;
                #[cfg(test)]
                self.checkpoint("history_published")?;
                sync_dir(&self.root.join("history").join(share))?;
                Ok::<(), io::Error>(())
            })();
            let _ = fs::remove_file(temp);
            result?;
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
    fn retention_counts_age_trash_and_deduplication_without_removing_current_data() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("store");
        let mut store = Store::open(&root).unwrap();
        let mut base = 0;
        for bytes in ["one", "two", "three"] {
            let hash = crewsync_protocol::disk::hash_secret(bytes).unwrap();
            store.put_object(bytes.as_bytes(), &hash, 100).unwrap();
            base = store
                .apply(
                    "s",
                    "file",
                    base,
                    Content::File {
                        hash,
                        size: bytes.len() as u64,
                    },
                )
                .unwrap();
        }
        let policy = Retention {
            previous_versions: 1,
            history_days: 1,
            trash_days: 2,
        };
        let time = now().unwrap();
        let dry = store.maintain_at(policy, false, time).unwrap();
        assert_eq!(dry.pruned_versions, 1);
        assert_eq!(dry.reclaimable_objects, 1);
        assert_eq!(store.history("s", "file").unwrap().len(), 3);
        store.maintain_at(policy, true, time).unwrap();
        assert_eq!(store.history("s", "file").unwrap().len(), 2);
        assert_eq!(fs::read_dir(root.join("objects")).unwrap().count(), 2);
        let deleted = store.apply("s", "file", base, Content::Deleted).unwrap();
        store.maintain_at(policy, true, time + 86400 + 1).unwrap();
        assert_eq!(
            store.history("s", "file").unwrap().len(),
            2,
            "lixeira preserva última versão"
        );
        store.maintain_at(policy, true, time + 3 * 86400).unwrap();
        assert_eq!(store.history("s", "file").unwrap().len(), 1);
        assert_eq!(fs::read_dir(root.join("objects")).unwrap().count(), 0);
        assert_eq!(
            store.manifest("s").unwrap().entries["file"].revision,
            deleted
        );
        assert!(store.apply("s", "file", 0, Content::Directory).is_err());
        // Objeto compartilhado por outro caminho nunca é removido por idade.
        store.put_object(&b"abc"[..], ABC, 3).unwrap();
        store
            .apply(
                "s",
                "keep",
                0,
                Content::File {
                    hash: ABC.into(),
                    size: 3,
                },
            )
            .unwrap();
        store.maintain_at(policy, true, time + 100 * 86400).unwrap();
        store.object(ABC, 3).unwrap();
        drop(store);
        Store::open(root).unwrap();
    }

    #[test]
    fn maintenance_failures_preserve_all_remaining_references() {
        for stage in [
            "maintenance_written",
            "maintenance_metadata_durable",
            "maintenance_object_removed",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let root = dir.path().join("store");
            let mut store = Store::open(&root).unwrap();
            store.put_object(&b"abc"[..], ABC, 3).unwrap();
            let first = store
                .apply(
                    "s",
                    "f",
                    0,
                    Content::File {
                        hash: ABC.into(),
                        size: 3,
                    },
                )
                .unwrap();
            store.apply("s", "f", first, Content::Directory).unwrap();
            let orphan = crewsync_protocol::disk::hash_secret("orphan").unwrap();
            store.put_object(&b"orphan"[..], &orphan, 6).unwrap();
            store.fail_at = Some(stage);
            let time = now().unwrap()
                + if stage == "maintenance_written" {
                    0
                } else {
                    31 * 86400
                };
            assert!(store.maintain_at(Retention::default(), true, time).is_err());
            drop(store);
            let store = Store::open(&root).unwrap();
            assert_eq!(
                store.manifest("s").unwrap().entries["f"].content,
                Content::Directory
            );
            for entry in store.history("s", "f").unwrap() {
                if let Content::File { hash, size } = entry.content {
                    store.object(&hash, size).unwrap();
                }
            }
        }
    }

    #[test]
    fn legacy_history_gets_a_full_retention_grace_period() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = Store::open(dir.path().join("store")).unwrap();
        let first = store.apply("s", "f", 0, Content::Directory).unwrap();
        store.apply("s", "f", first, Content::Deleted).unwrap();
        let path = store.history_path("s", "f").unwrap();
        let mut data: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        data["format"] = 1.into();
        data.as_object_mut().unwrap().remove("archived_at");
        fs::write(path, serde_json::to_vec(&data).unwrap()).unwrap();
        let timestamp = now().unwrap() + 100 * 86400;
        assert_eq!(
            store
                .maintain_at(Retention::default(), true, timestamp)
                .unwrap()
                .pruned_versions,
            0
        );
        assert_eq!(store.history("s", "f").unwrap().len(), 2);
    }

    #[test]
    fn interrupted_replacement_keeps_previous_version_recoverable() {
        for stage in [
            "history_written",
            "history_published",
            "manifest_written",
            "manifest_synced",
            "manifest_published",
            "manifest_durable",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let root = dir.path().join("store");
            let mut store = Store::open(&root).unwrap();
            store.put_object(&b"abc"[..], ABC, 3).unwrap();
            let first = store
                .apply(
                    "share",
                    "file",
                    0,
                    Content::File {
                        hash: ABC.into(),
                        size: 3,
                    },
                )
                .unwrap();
            store.fail_at = Some(stage);
            assert_eq!(
                store
                    .apply("share", "file", first, Content::Deleted)
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::StorageFull
            );
            drop(store);
            let mut store = Store::open(&root).unwrap();
            let history = store.history("share", "file").unwrap();
            assert_eq!(history[0].revision, first);
            assert!(matches!(history[0].content, Content::File { .. }));
            let base = store.manifest("share").unwrap().entries["file"].revision;
            store.restore("share", "file", first, base).unwrap();
            drop(store);
            Store::open(&root).unwrap();
        }
    }

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
