use crewsync_server::storage::{Content, Store};
use std::{fs, io};

const ABC: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
fn file() -> Content {
    Content::File {
        hash: ABC.into(),
        size: 3,
    }
}

#[test]
fn legacy_current_version_is_preserved_and_restore_creates_a_new_revision() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("store");
    let mut store = Store::open(&root).unwrap();
    store.put_object(&b"abc"[..], ABC, 3).unwrap();
    let first = store.apply("Obras", "file", 0, file()).unwrap();
    drop(store);
    // Simula armazenamento do piloto anterior, sem diretório de histórico.
    fs::remove_dir(root.join("history")).unwrap();
    let mut store = Store::open(&root).unwrap();
    assert_eq!(store.history("Obras", "file").unwrap()[0].revision, first);
    let deleted = store
        .apply("Obras", "file", first, Content::Deleted)
        .unwrap();
    drop(store);
    let mut store = Store::open(&root).unwrap();
    let restored = store.restore("Obras", "file", first, deleted).unwrap();
    assert!(restored > deleted);
    let history = store.history("Obras", "file").unwrap();
    assert_eq!(
        history.iter().map(|e| e.revision).collect::<Vec<_>>(),
        [first, deleted, restored]
    );
    assert_eq!(
        store.manifest("Obras").unwrap().entries["file"].content,
        file()
    );
    assert_eq!(fs::read_dir(root.join("objects")).unwrap().count(), 1);
    assert_eq!(
        store
            .restore("Obras", "file", first, deleted)
            .unwrap_err()
            .kind(),
        io::ErrorKind::AlreadyExists
    );
    assert!(store.restore("Obras", "file", deleted, restored).is_err());
    assert_eq!(
        store.restore("Other", "file", first, 0).unwrap_err().kind(),
        io::ErrorKind::NotFound
    );
    assert_eq!(
        store
            .restore("Obras", "other", first, 0)
            .unwrap_err()
            .kind(),
        io::ErrorKind::NotFound
    );
    assert!(store.history("Obras", "../file").is_err());
    drop(store);
    assert_eq!(
        Store::open(&root)
            .unwrap()
            .history("Obras", "file")
            .unwrap(),
        history
    );
}

#[test]
fn restoring_nested_files_requires_live_parents_and_directory_restore_is_not_recursive() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path().join("store")).unwrap();
    store.put_object(&b"abc"[..], ABC, 3).unwrap();
    let parent = store.apply("Obras", "dir", 0, Content::Directory).unwrap();
    let file_revision = store.apply("Obras", "dir/file", 0, file()).unwrap();
    let deleted_file = store
        .apply("Obras", "dir/file", file_revision, Content::Deleted)
        .unwrap();
    let deleted_parent = store
        .apply("Obras", "dir", parent, Content::Deleted)
        .unwrap();
    assert!(
        store
            .restore("Obras", "dir/file", file_revision, deleted_file)
            .is_err()
    );
    store
        .restore("Obras", "dir", parent, deleted_parent)
        .unwrap();
    assert_eq!(
        store.manifest("Obras").unwrap().entries["dir/file"].content,
        Content::Deleted
    );
    store
        .restore("Obras", "dir/file", file_revision, deleted_file)
        .unwrap();
}

#[test]
fn corrupt_history_and_missing_historical_objects_fail_closed() {
    for corrupt_metadata in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("store");
        let mut store = Store::open(&root).unwrap();
        store.put_object(&b"abc"[..], ABC, 3).unwrap();
        let rev = store.apply("Obras", "file", 0, file()).unwrap();
        store.apply("Obras", "file", rev, Content::Deleted).unwrap();
        drop(store);
        if corrupt_metadata {
            let path = fs::read_dir(root.join("history/Obras"))
                .unwrap()
                .next()
                .unwrap()
                .unwrap()
                .path();
            fs::write(path, b"{}").unwrap();
        } else {
            fs::remove_file(root.join("objects").join(ABC)).unwrap();
        }
        assert!(Store::open(root).is_err());
    }
}
