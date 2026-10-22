//! Acesso local ancorado em descritores Linux, sem seguir links.
use crewsync_protocol::{Content, disk, invalid, validate_path};
use rustix::fs::{self as rfs, FlockOperation, Mode, OFlags, RenameFlags, ResolveFlags};
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io;
use std::os::fd::AsRawFd;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

pub struct Tree {
    root: File,
    meta: File,
    lock: File,
}
impl Drop for Tree {
    fn drop(&mut self) {
        let _ = rfs::flock(&self.lock, FlockOperation::Unlock);
    }
}
fn proc_path(file: &File) -> PathBuf {
    PathBuf::from(format!("/proc/self/fd/{}", file.as_raw_fd()))
}
fn stamp(meta: &fs::Metadata) -> (u64, i64, i64, i64, i64) {
    (
        meta.len(),
        meta.mtime(),
        meta.mtime_nsec(),
        meta.ctime(),
        meta.ctime_nsec(),
    )
}
fn flags() -> OFlags {
    OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOFOLLOW | OFlags::NONBLOCK
}

impl Tree {
    pub fn open(path: &Path) -> io::Result<Self> {
        let absolute = std::path::absolute(path)?;
        let root = File::from(rfs::openat2(
            rfs::CWD,
            &absolute,
            flags() | OFlags::DIRECTORY,
            Mode::empty(),
            ResolveFlags::NO_SYMLINKS,
        )?);
        Self::from_root(root)
    }
    fn from_root(root: File) -> io::Result<Self> {
        match rfs::mkdirat(&root, ".crewsync", Mode::from_bits_truncate(0o700)) {
            Ok(()) => root.sync_all()?,
            Err(e) if e == rustix::io::Errno::EXIST => {}
            Err(e) => return Err(e.into()),
        }
        let meta = File::from(rfs::openat2(
            &root,
            ".crewsync",
            flags() | OFlags::DIRECTORY,
            Mode::empty(),
            ResolveFlags::BENEATH | ResolveFlags::NO_SYMLINKS,
        )?);
        disk::private_dir(&proc_path(&root).join(".crewsync"))?;
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(OFlags::NOFOLLOW.bits() as i32)
            .open(proc_path(&meta).join("lock"))?;
        if !lock.metadata()?.is_file() {
            return Err(invalid("lock local inválido"));
        }
        rfs::flock(&lock, FlockOperation::NonBlockingLockExclusive)?;
        let tree = Self { root, meta, lock };
        for area in ["tmp", "trash", "conflicts"] {
            disk::private_dir(&tree.meta_path().join(area))?;
        }
        for item in fs::read_dir(tree.meta_path().join("tmp"))? {
            let path = item?.path();
            if !fs::symlink_metadata(&path)?.is_file() {
                return Err(invalid("temporário local inválido"));
            }
            fs::remove_file(path)?;
        }
        Ok(tree)
    }
    /// Cria e abre um compartilhamento sem seguir links, ancorado na raiz.
    pub(crate) fn child(&self, name: &str) -> io::Result<Self> {
        validate_path(name)?;
        if name.contains('/') {
            return Err(invalid("compartilhamento deve ser de primeiro nível"));
        }
        match rfs::mkdirat(&self.root, name, Mode::from_bits_truncate(0o700)) {
            Ok(()) => self.root.sync_all()?,
            Err(e) if e == rustix::io::Errno::EXIST => {}
            Err(e) => return Err(e.into()),
        }
        Self::from_root(self.directory(name)?)
    }
    pub fn meta_path(&self) -> PathBuf {
        proc_path(&self.meta)
    }
    pub fn file(&self, path: &str) -> io::Result<File> {
        validate_path(path)?;
        let file = File::from(rfs::openat2(
            &self.root,
            path,
            flags(),
            Mode::empty(),
            ResolveFlags::BENEATH | ResolveFlags::NO_SYMLINKS,
        )?);
        if !file.metadata()?.is_file() {
            return Err(invalid("entrada local não é arquivo regular"));
        }
        Ok(file)
    }
    fn directory(&self, path: &str) -> io::Result<File> {
        if path.is_empty() {
            return self.root.try_clone();
        }
        validate_path(path)?;
        Ok(File::from(rfs::openat2(
            &self.root,
            path,
            flags() | OFlags::DIRECTORY,
            Mode::empty(),
            ResolveFlags::BENEATH | ResolveFlags::NO_SYMLINKS,
        )?))
    }
    fn parent(&self, path: &str) -> io::Result<(File, String)> {
        validate_path(path)?;
        let (parent, name) = path.rsplit_once('/').unwrap_or(("", path));
        Ok((self.directory(parent)?, name.to_owned()))
    }
    pub fn current(&self, path: &str) -> io::Result<Option<Content>> {
        let (parent, name) = match self.parent(path) {
            Err(e)
                if matches!(
                    e.kind(),
                    io::ErrorKind::NotFound | io::ErrorKind::NotADirectory
                ) =>
            {
                return Ok(None);
            }
            other => other?,
        };
        let stat = match rfs::statat(&parent, &name, rfs::AtFlags::SYMLINK_NOFOLLOW) {
            Ok(stat) => stat,
            Err(e) if e == rustix::io::Errno::NOENT => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        match rfs::FileType::from_raw_mode(stat.st_mode) {
            rfs::FileType::Directory => {
                self.directory(path)?;
                Ok(Some(Content::Directory))
            }
            rfs::FileType::RegularFile => {
                let file = self.file(path)?;
                let before = file.metadata()?;
                if before.len() > (1u64 << 40) {
                    return Err(invalid("arquivo excede limite de 1 TiB"));
                }
                let hash = disk::hash_file(&file)?;
                if stamp(&before) != stamp(&file.metadata()?) {
                    return Err(io::Error::new(
                        io::ErrorKind::Interrupted,
                        "arquivo mudou durante leitura",
                    ));
                }
                Ok(Some(Content::File {
                    hash,
                    size: before.len(),
                }))
            }
            _ => Err(invalid("links e arquivos especiais não são sincronizados")),
        }
    }
    pub fn scan(&self) -> io::Result<BTreeMap<String, Content>> {
        let mut entries = BTreeMap::new();
        self.walk("", &mut entries)?;
        Ok(entries)
    }
    fn walk(&self, prefix: &str, entries: &mut BTreeMap<String, Content>) -> io::Result<()> {
        for item in fs::read_dir(proc_path(&self.directory(prefix)?))? {
            let item = item?;
            let name = item
                .file_name()
                .to_str()
                .ok_or_else(|| invalid("nome local não UTF-8"))?
                .to_owned();
            if prefix.is_empty() && name == ".crewsync" {
                continue;
            }
            let path = if prefix.is_empty() {
                name
            } else {
                format!("{prefix}/{name}")
            };
            validate_path(&path)?;
            let content = self.current(&path)?.ok_or_else(|| {
                io::Error::new(io::ErrorKind::Interrupted, "árvore mudou durante varredura")
            })?;
            let directory = content == Content::Directory;
            entries.insert(path.clone(), content);
            if entries.len() > 100_000 {
                return Err(invalid("árvore excede limite de entradas"));
            }
            if directory {
                self.walk(&path, entries)?;
            }
        }
        Ok(())
    }
    pub fn temp(&self) -> io::Result<(PathBuf, File)> {
        let path = self.meta_path().join("tmp").join(disk::random_hex(16)?);
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)?;
        Ok((path, file))
    }
    pub fn check(&self, path: &str, expected: Option<&Content>) -> io::Result<()> {
        if self.current(path)?.as_ref() != expected {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "arquivo local mudou; execute novamente",
            ));
        }
        Ok(())
    }
    pub fn preserve(&self, path: &str, area: &str) -> io::Result<()> {
        let (parent, name) = self.parent(path)?;
        let target = self.meta_path().join(area);
        let saved = format!(
            "{}-{}",
            disk::random_hex(8)?,
            name.chars().take(50).collect::<String>()
        );
        disk::atomic_json(
            &target.join(format!("{saved}.json")),
            &serde_json::json!({"original_path":path,"saved":saved}),
        )?;
        rfs::renameat_with(
            &parent,
            &name,
            rfs::CWD,
            target.join(&saved),
            RenameFlags::NOREPLACE,
        )?;
        parent.sync_all()?;
        File::open(target)?.sync_all()
    }
    pub fn replace(
        &self,
        path: &str,
        content: Option<&Content>,
        temp: Option<&Path>,
        expected: Option<&Content>,
        conflict: bool,
    ) -> io::Result<()> {
        self.check(path, expected)?;
        if expected.is_some() {
            self.preserve(path, if conflict { "conflicts" } else { "trash" })?;
        }
        let (parent, name) = self.parent(path)?;
        match content {
            Some(Content::Directory) => {
                rfs::mkdirat(&parent, &name, Mode::from_bits_truncate(0o700))?
            }
            Some(Content::File { .. }) => {
                let temp = temp.ok_or_else(|| invalid("download ausente"))?;
                rfs::renameat_with(rfs::CWD, temp, &parent, &name, RenameFlags::NOREPLACE)?;
                File::open(self.meta_path().join("tmp"))?.sync_all()?;
            }
            _ => {}
        }
        parent.sync_all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn replacing_parent_with_symlink_cannot_escape_open_tree() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("root");
        fs::create_dir(&root).unwrap();
        fs::create_dir(root.join("dir")).unwrap();
        let outside = directory.path().join("outside");
        fs::create_dir(&outside).unwrap();
        fs::write(outside.join("file"), b"untouched").unwrap();
        let tree = Tree::open(&root).unwrap();
        fs::rename(root.join("dir"), root.join("old")).unwrap();
        std::os::unix::fs::symlink(&outside, root.join("dir")).unwrap();
        assert!(tree.file("dir/file").is_err());
        assert!(tree.replace("dir/file", None, None, None, false).is_err());
        assert_eq!(fs::read(outside.join("file")).unwrap(), b"untouched");
    }
    #[test]
    fn lock_excludes_other_instance_and_releases_on_drop() {
        let dir = tempfile::tempdir().unwrap();
        let tree = Tree::open(dir.path()).unwrap();
        assert!(Tree::open(dir.path()).is_err());
        drop(tree);
        Tree::open(dir.path()).unwrap();
    }
    #[test]
    fn replacement_keeps_previous_bytes_and_refuses_stale_snapshot() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("file"), b"old").unwrap();
        let tree = Tree::open(dir.path()).unwrap();
        let old = tree.current("file").unwrap();
        fs::write(dir.path().join("file"), b"new").unwrap();
        assert!(
            tree.replace("file", None, None, old.as_ref(), false)
                .is_err()
        );
        assert_eq!(fs::read(dir.path().join("file")).unwrap(), b"new");
    }
}
