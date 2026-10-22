//! Regras reais de cliente+armazenamento com transporte determinístico sem sockets.
use crewsync_client::{Remote, sync};
use crewsync_protocol::Operation;
use crewsync_server::access::AuthorizedStore;
use std::cell::{Cell, RefCell};
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

struct Backend {
    store: RefCell<AuthorizedStore>,
    lose_reply: Cell<bool>,
    corrupt_download: Cell<bool>,
}
impl Remote for Backend {
    fn identity(&self) -> &str {
        "https://pilot.invalid:8443"
    }
    fn transfer(
        &self,
        operation: Operation,
        input: &mut dyn Read,
        _size: u64,
        output: &mut dyn Write,
        limit: u64,
    ) -> io::Result<()> {
        let mut store = self.store.borrow_mut();
        let value = match operation {
            Operation::Manifest { share } => {
                serde_json::to_vec(&store.manifest("writer", &share)?).unwrap()
            }
            Operation::Mkdir { share, path, base } => {
                serde_json::to_vec(&store.create_directory("writer", &share, &path, base)?).unwrap()
            }
            Operation::Delete { share, path, base } => {
                serde_json::to_vec(&store.delete("writer", &share, &path, base)?).unwrap()
            }
            Operation::Put {
                share,
                path,
                base,
                hash,
                ..
            } => {
                let revision = store.put_file("writer", &share, &path, base, &hash, input)?;
                if self.lose_reply.replace(false) {
                    return Err(io::Error::new(
                        io::ErrorKind::ConnectionAborted,
                        "resposta perdida",
                    ));
                }
                serde_json::to_vec(&revision).unwrap()
            }
            Operation::Get { share, path } => {
                if self.corrupt_download.replace(false) {
                    output.write_all(b"bad")?;
                    return Ok(());
                }
                let file = store.file("writer", &share, &path)?;
                assert!(file.metadata()?.len() <= limit);
                io::copy(&mut file.take(limit), output)?;
                return Ok(());
            }
            _ => panic!("operação inesperada no teste"),
        };
        assert!(value.len() as u64 <= limit);
        output.write_all(&value)
    }
}
struct Fixture {
    _temp: TempDir,
    root: PathBuf,
    backend: Backend,
}
impl Fixture {
    fn new() -> Self {
        let temp = TempDir::new().unwrap();
        let root = temp.path().to_owned();
        let config = root.join("server.toml");
        fs::write(&config, format!("version=1\n[server]\nstorage_root='{}'\n[users.writer]\nenabled=true\n[shares.Obras.permissions]\nwriter='write'\n", root.join("store").display())).unwrap();
        let store = AuthorizedStore::open(&config).unwrap();
        Self {
            _temp: temp,
            root,
            backend: Backend {
                store: RefCell::new(store),
                lose_reply: Cell::new(false),
                corrupt_download: Cell::new(false),
            },
        }
    }
    fn folder(&self, name: &str) -> PathBuf {
        let path = self.root.join(name);
        fs::create_dir(&path).unwrap();
        path
    }
    fn sync(&self, folder: &Path) -> sync::Report {
        sync::sync(&self.backend, "Obras", folder).unwrap()
    }
}
fn preserved(folder: &Path, area: &str, bytes: &[u8]) -> bool {
    fs::read_dir(folder.join(".crewsync").join(area))
        .unwrap()
        .any(|entry| fs::read(entry.unwrap().path()).ok().as_deref() == Some(bytes))
}
#[test]
fn creations_modifications_conflicts_deletions_and_cycles() {
    let fixture = Fixture::new();
    let a = fixture.folder("a");
    let b = fixture.folder("b");
    fs::create_dir(a.join("dir")).unwrap();
    fs::write(a.join("dir/file"), b"initial").unwrap();
    fixture.sync(&a);
    fixture.sync(&b);
    assert_eq!(fs::read(b.join("dir/file")).unwrap(), b"initial");
    fs::write(a.join("dir/file"), b"a").unwrap();
    fs::write(b.join("dir/file"), b"b").unwrap();
    fixture.sync(&a);
    assert_eq!(fixture.sync(&b).conflicts, 1);
    assert_eq!(fs::read(b.join("dir/file")).unwrap(), b"a");
    assert!(preserved(&b, "conflicts", b"b"));
    let report = fixture.sync(&b);
    assert_eq!((report.uploaded, report.downloaded), (0, 0));
    fs::remove_file(a.join("dir/file")).unwrap();
    fs::remove_dir(a.join("dir")).unwrap();
    fixture.sync(&a);
    fixture.sync(&b);
    assert!(!b.join("dir").exists());
    let report = fixture.sync(&a);
    assert_eq!((report.uploaded, report.downloaded), (0, 0));
}
#[test]
fn response_loss_after_publication_is_idempotent() {
    let fixture = Fixture::new();
    let a = fixture.folder("a");
    fs::write(a.join("file"), b"content").unwrap();
    fixture.backend.lose_reply.set(true);
    assert!(sync::sync(&fixture.backend, "Obras", &a).is_err());
    let revision = fixture
        .backend
        .store
        .borrow()
        .manifest("writer", "Obras")
        .unwrap()
        .revision;
    fixture.sync(&a);
    assert_eq!(
        fixture
            .backend
            .store
            .borrow()
            .manifest("writer", "Obras")
            .unwrap()
            .revision,
        revision
    );
    assert_eq!(fs::read(a.join("file")).unwrap(), b"content");
}
#[test]
fn changed_offline_file_survives_remote_deletion() {
    let fixture = Fixture::new();
    let a = fixture.folder("a");
    let b = fixture.folder("b");
    fs::write(a.join("file"), b"initial").unwrap();
    fixture.sync(&a);
    fixture.sync(&b);
    fs::write(b.join("file"), b"offline").unwrap();
    fs::remove_file(a.join("file")).unwrap();
    fixture.sync(&a);
    assert_eq!(fixture.sync(&b).conflicts, 1);
    assert!(!b.join("file").exists());
    assert!(preserved(&b, "conflicts", b"offline"));
}
#[test]
fn deleted_remote_directory_preserves_locally_edited_children() {
    let fixture = Fixture::new();
    let a = fixture.folder("a");
    let b = fixture.folder("b");
    fs::create_dir(a.join("dir")).unwrap();
    fs::write(a.join("dir/file"), b"initial").unwrap();
    fixture.sync(&a);
    fixture.sync(&b);
    fs::write(b.join("dir/file"), b"offline").unwrap();
    fs::remove_dir_all(a.join("dir")).unwrap();
    fixture.sync(&a);
    assert!(fixture.sync(&b).conflicts >= 1);
    assert!(!b.join("dir").exists());
    assert!(
        fs::read_dir(b.join(".crewsync/conflicts"))
            .unwrap()
            .any(|e| fs::read(e.unwrap().path().join("file")).ok().as_deref() == Some(b"offline"))
    );
}
#[test]
fn file_directory_type_changes_and_renames_converge() {
    let fixture = Fixture::new();
    let a = fixture.folder("a");
    let b = fixture.folder("b");
    fs::create_dir(a.join("dir")).unwrap();
    fs::write(a.join("dir/child"), b"child").unwrap();
    fixture.sync(&a);
    fixture.sync(&b);
    fs::remove_dir_all(a.join("dir")).unwrap();
    fs::write(a.join("dir"), b"file now").unwrap();
    fixture.sync(&a);
    fixture.sync(&b);
    assert_eq!(fs::read(b.join("dir")).unwrap(), b"file now");
    fs::remove_file(a.join("dir")).unwrap();
    fs::create_dir(a.join("dir")).unwrap();
    fs::create_dir(a.join("dir/empty")).unwrap();
    fixture.sync(&a);
    fixture.sync(&b);
    assert!(b.join("dir/empty").is_dir());
    fs::rename(a.join("dir"), a.join("renamed")).unwrap();
    fixture.sync(&a);
    fixture.sync(&b);
    assert!(!b.join("dir").exists());
    assert!(b.join("renamed/empty").is_dir());
}
#[test]
fn corrupt_download_never_replaces_local_file() {
    let fixture = Fixture::new();
    let a = fixture.folder("a");
    let b = fixture.folder("b");
    fs::write(a.join("file"), b"old").unwrap();
    fixture.sync(&a);
    fixture.sync(&b);
    fs::write(a.join("file"), b"new").unwrap();
    fixture.sync(&a);
    fixture.backend.corrupt_download.set(true);
    assert!(sync::sync(&fixture.backend, "Obras", &b).is_err());
    assert_eq!(fs::read(b.join("file")).unwrap(), b"old");
    fixture.sync(&b);
    assert_eq!(fs::read(b.join("file")).unwrap(), b"new");
}
#[test]
fn symlinks_and_reserved_paths_abort_before_uploads() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new();
    let a = fixture.folder("a");
    let outside = fixture.root.join("outside");
    fs::write(&outside, b"secret").unwrap();
    symlink(&outside, a.join("link")).unwrap();
    fs::write(a.join("normal"), b"normal").unwrap();
    assert!(sync::sync(&fixture.backend, "Obras", &a).is_err());
    assert!(
        fixture
            .backend
            .store
            .borrow()
            .manifest("writer", "Obras")
            .unwrap()
            .entries
            .is_empty()
    );
    assert!(crewsync_protocol::validate_path(".crewsync/state.json").is_err());
}
#[test]
fn stale_initial_files_do_not_resurrect_tombstones() {
    let fixture = Fixture::new();
    let a = fixture.folder("a");
    let b = fixture.folder("b");
    fs::write(a.join("file"), b"initial").unwrap();
    fixture.sync(&a);
    fs::remove_file(a.join("file")).unwrap();
    fixture.sync(&a);
    fs::write(b.join("file"), b"stale").unwrap();
    assert_eq!(fixture.sync(&b).conflicts, 1);
    assert!(!b.join("file").exists());
    assert!(preserved(&b, "conflicts", b"stale"));
}

#[test]
fn modification_during_upload_is_sent_in_next_pass() {
    struct During<'a> {
        backend: &'a Backend,
        path: PathBuf,
        once: Cell<bool>,
    }
    impl Remote for During<'_> {
        fn identity(&self) -> &str {
            self.backend.identity()
        }
        fn transfer(
            &self,
            operation: Operation,
            input: &mut dyn Read,
            size: u64,
            output: &mut dyn Write,
            limit: u64,
        ) -> io::Result<()> {
            if matches!(operation, Operation::Put { .. }) && self.once.replace(false) {
                fs::write(&self.path, b"new edit")?;
            }
            self.backend.transfer(operation, input, size, output, limit)
        }
    }
    let fixture = Fixture::new();
    let a = fixture.folder("a");
    let b = fixture.folder("b");
    let path = a.join("file");
    fs::write(&path, b"snapshot").unwrap();
    let remote = During {
        backend: &fixture.backend,
        path,
        once: Cell::new(true),
    };
    sync::sync(&remote, "Obras", &a).unwrap();
    fixture.sync(&b);
    assert_eq!(fs::read(b.join("file")).unwrap(), b"snapshot");
    fixture.sync(&a);
    fixture.sync(&b);
    assert_eq!(fs::read(b.join("file")).unwrap(), b"new edit");
}

#[test]
fn local_directory_deletion_cannot_remove_new_remote_children() {
    let fixture = Fixture::new();
    let a = fixture.folder("a");
    let b = fixture.folder("b");
    fs::create_dir(a.join("dir")).unwrap();
    fs::write(a.join("dir/old"), b"initial").unwrap();
    fixture.sync(&a);
    fixture.sync(&b);
    fs::remove_dir_all(b.join("dir")).unwrap();
    fs::write(a.join("dir/new"), b"new remotely").unwrap();
    fs::write(a.join("dir/old"), b"edited remotely").unwrap();
    fixture.sync(&a);
    assert!(fixture.sync(&b).conflicts > 0);
    assert_eq!(fs::read(b.join("dir/new")).unwrap(), b"new remotely");
    assert_eq!(fs::read(b.join("dir/old")).unwrap(), b"edited remotely");
    assert_eq!(fixture.sync(&b).uploaded, 0);
}
