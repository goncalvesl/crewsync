use std::fs;
use std::io::{self, Read};
use std::os::unix::fs::symlink;

use crewsync_server::storage::{Content, Store, validate_path};
use tempfile::TempDir;

const ABC: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
const EMPTY: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

fn fixture() -> (TempDir, Store) {
    let dir = TempDir::new().unwrap();
    let store = Store::open(dir.path().join("store")).unwrap();
    (dir, store)
}

fn file() -> Content {
    Content::File {
        hash: ABC.into(),
        size: 3,
    }
}

#[test]
fn objects_are_verified_deduplicated_and_streamed() {
    let (dir, mut store) = fixture();
    assert_eq!(store.put_object(&b"abc"[..], ABC, 3).unwrap(), 3);
    store.put_object(&b"abc"[..], ABC, 3).unwrap();
    store.put_object(io::empty(), EMPTY, 0).unwrap();
    assert_eq!(
        fs::read_dir(dir.path().join("store/objects"))
            .unwrap()
            .count(),
        2
    );
    let mut bytes = Vec::new();
    store
        .object(ABC, 3)
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    assert_eq!(bytes, b"abc");
    assert!(store.object(ABC, 4).is_err());
}

#[test]
fn failed_uploads_never_publish() {
    let (dir, mut store) = fixture();
    assert!(store.put_object(&b"bad"[..], ABC, 3).is_err());
    assert!(store.put_object(&b"abc"[..], ABC, 2).is_err());
    assert!(store.put_object(io::empty(), "../escape", 0).is_err());
    struct Broken(bool);
    impl Read for Broken {
        fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
            if self.0 {
                return Err(io::Error::other("transferência interrompida"));
            }
            self.0 = true;
            bytes[0] = b'a';
            Ok(1)
        }
    }
    assert!(store.put_object(Broken(false), ABC, 100).is_err());
    assert_eq!(
        fs::read_dir(dir.path().join("store/tmp")).unwrap().count(),
        0
    );
    assert_eq!(
        fs::read_dir(dir.path().join("store/objects"))
            .unwrap()
            .count(),
        0
    );
    store.put_object(&b"abc"[..], ABC, 3).unwrap();
}

#[test]
fn revisions_empty_directories_deletions_and_isolation_survive_restart() {
    let (dir, mut store) = fixture();
    store.put_object(&b"abc"[..], ABC, 3).unwrap();
    assert_eq!(
        store
            .apply("obras", "vazia", 0, Content::Directory)
            .unwrap(),
        1
    );
    assert_eq!(
        store.apply("obras", "vazia/planta.txt", 0, file()).unwrap(),
        2
    );
    assert!(store.apply("obras", "vazia", 1, Content::Deleted).is_err());
    assert_eq!(
        store
            .apply("obras", "vazia/planta.txt", 2, Content::Deleted)
            .unwrap(),
        3
    );
    assert_eq!(
        store.apply("obras", "vazia", 1, Content::Deleted).unwrap(),
        4
    );
    assert_eq!(store.manifest("outro").unwrap().revision, 0);
    drop(store);
    let mut store = Store::open(dir.path().join("store")).unwrap();
    let manifest = store.manifest("obras").unwrap();
    assert_eq!(manifest.revision, 4);
    assert_eq!(
        manifest.entries["vazia/planta.txt"].content,
        Content::Deleted
    );
    assert_eq!(
        store
            .apply("obras", "vazia/planta.txt", 0, file())
            .unwrap_err()
            .kind(),
        io::ErrorKind::AlreadyExists
    );
    store
        .apply("obras", "vazia", 4, Content::Directory)
        .unwrap();
    store.apply("obras", "vazia/planta.txt", 3, file()).unwrap();
}

#[test]
fn stale_writer_preserves_current_manifest_and_both_objects() {
    let (dir, mut store) = fixture();
    store.put_object(&b"abc"[..], ABC, 3).unwrap();
    store.put_object(io::empty(), EMPTY, 0).unwrap();
    store.apply("share", "file", 0, file()).unwrap();
    let before = store.manifest("share").unwrap();
    assert_eq!(
        store
            .apply(
                "share",
                "file",
                0,
                Content::File {
                    hash: EMPTY.into(),
                    size: 0
                }
            )
            .unwrap_err()
            .kind(),
        io::ErrorKind::AlreadyExists
    );
    assert_eq!(store.manifest("share").unwrap(), before);
    assert_eq!(
        fs::read_dir(dir.path().join("store/objects"))
            .unwrap()
            .count(),
        2
    );
}

#[test]
fn malicious_paths_and_invalid_hierarchy_are_rejected() {
    let (_dir, mut store) = fixture();
    for path in [
        "", "/a", "../a", "a/../b", "a//b", "a/", "a\\b", ".", "a\0b", "CON.txt", "LPT1", "a.",
        "a ", "a:b",
    ] {
        assert!(validate_path(path).is_err(), "{path:?}");
        assert!(store.apply("share", path, 0, Content::Directory).is_err());
    }
    assert!(
        store
            .apply("../escape", "a", 0, Content::Directory)
            .is_err()
    );
    assert!(
        store
            .apply("share", "missing/child", 0, Content::Directory)
            .is_err()
    );
    assert!(store.apply("share", "file", 0, file()).is_err());
    assert!(
        store
            .apply("share", "missing", 0, Content::Deleted)
            .is_err()
    );
    store.apply("share", "a", 0, Content::Directory).unwrap();
    store.apply("share", "a/b", 0, Content::Directory).unwrap();
    store.put_object(&b"abc"[..], ABC, 3).unwrap();
    assert!(store.apply("share", "a", 1, file()).is_err());
}

#[test]
fn only_one_instance_can_write_and_lock_releases_on_drop() {
    let (dir, store) = fixture();
    assert!(Store::open(dir.path().join("store")).is_err());
    drop(store);
    Store::open(dir.path().join("store")).unwrap();
}

#[test]
fn recovery_discards_unpublished_temps_and_keeps_orphan_objects() {
    let (dir, mut store) = fixture();
    store.put_object(&b"abc"[..], ABC, 3).unwrap();
    store.apply("share", "file", 0, file()).unwrap();
    // Queda depois de publicar outro objeto, antes de publicar o manifesto.
    store.put_object(io::empty(), EMPTY, 0).unwrap();
    let before = store.manifest("share").unwrap();
    drop(store);
    let root = dir.path().join("store");
    // Queda durante upload ou durante gravação do próximo manifesto.
    fs::write(root.join("tmp/object"), b"partial").unwrap();
    fs::write(root.join("tmp/manifest"), b"{partial").unwrap();
    let store = Store::open(&root).unwrap();
    assert_eq!(store.manifest("share").unwrap(), before);
    assert_eq!(fs::read_dir(root.join("tmp")).unwrap().count(), 0);
    store.object(EMPTY, 0).unwrap();
}

#[test]
fn recovery_rejects_corruption_and_missing_objects_without_deleting_manifest() {
    for missing in [false, true] {
        let (dir, mut store) = fixture();
        store.put_object(&b"abc"[..], ABC, 3).unwrap();
        store.apply("share", "file", 0, file()).unwrap();
        let root = dir.path().join("store");
        let before = fs::read(root.join("manifests/share")).unwrap();
        drop(store);
        if missing {
            fs::remove_file(root.join("objects").join(ABC)).unwrap();
        } else {
            fs::write(root.join("objects").join(ABC), b"bad").unwrap();
        }
        assert!(Store::open(&root).is_err());
        assert_eq!(fs::read(root.join("manifests/share")).unwrap(), before);
    }
}

#[test]
fn recovery_rejects_invalid_manifest_format_revision_and_paths() {
    for json in [
        r#"{"format":2,"revision":0,"entries":{}}"#,
        r#"{"format":1,"revision":1,"entries":{}}"#,
        r#"{"format":1,"revision":1,"entries":{"../escape":{"revision":1,"content":{"kind":"Directory"}}}}"#,
        r#"{"format":1,"revision":1,"entries":{"a/b":{"revision":1,"content":{"kind":"Directory"}}}}"#,
        r#"{"format":1,"revision":0,"entries":{"a":{"revision":0,"content":{"kind":"Directory"}}}}"#,
        "{broken",
    ] {
        let (dir, store) = fixture();
        drop(store);
        fs::write(dir.path().join("store/manifests/share"), json).unwrap();
        assert!(Store::open(dir.path().join("store")).is_err(), "{json}");
    }
}

#[test]
fn symlinks_and_special_files_are_refused() {
    for area in ["objects", "manifests", "tmp"] {
        let (dir, store) = fixture();
        drop(store);
        let outside = dir.path().join("outside");
        fs::write(&outside, b"untouched").unwrap();
        symlink(&outside, dir.path().join("store").join(area).join(ABC)).unwrap();
        assert!(Store::open(dir.path().join("store")).is_err());
        assert_eq!(fs::read(&outside).unwrap(), b"untouched");
    }
    let (dir, store) = fixture();
    drop(store);
    symlink(dir.path().join("store"), dir.path().join("link")).unwrap();
    assert!(Store::open(dir.path().join("link")).is_err());
    fs::create_dir(dir.path().join("store/objects").join(ABC)).unwrap();
    assert!(Store::open(dir.path().join("store")).is_err());
}

#[test]
fn corrupt_existing_object_is_never_replaced_by_deduplication() {
    let (dir, mut store) = fixture();
    store.put_object(&b"abc"[..], ABC, 3).unwrap();
    fs::write(dir.path().join("store/objects").join(ABC), b"bad").unwrap();
    assert!(store.put_object(&b"abc"[..], ABC, 3).is_err());
    assert!(store.object(ABC, 3).is_err());
}

#[test]
fn large_upload_uses_multiple_buffers() {
    let (_dir, mut store) = fixture();
    let hash = "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0";
    let size = store
        .put_object(io::repeat(b'a').take(1_000_000), hash, 1_000_000)
        .unwrap();
    assert_eq!(size, 1_000_000);
    assert_eq!(
        io::copy(&mut store.object(hash, size).unwrap(), &mut io::sink()).unwrap(),
        size
    );
}

#[test]
fn non_private_storage_directory_is_rejected() {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("store");
    fs::create_dir(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(Store::open(&root).is_err());
}

#[test]
fn utf8_names_survive_reopening() {
    let (dir, mut store) = fixture();
    store
        .apply("Construção", "Medições (2026)", 0, Content::Directory)
        .unwrap();
    drop(store);
    let store = Store::open(dir.path().join("store")).unwrap();
    assert_eq!(
        store.manifest("Construção").unwrap().entries["Medições (2026)"].content,
        Content::Directory
    );
}
