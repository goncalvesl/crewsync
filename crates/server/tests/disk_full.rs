//! Ensaio real de ENOSPC, executado isoladamente em tmpfs limitado pelo Docker.
use crewsync_server::storage::{Content, Store};
use std::{fs, io};

#[test]
#[ignore = "exige CREWSYNC_FULL_DISK_DIR em tmpfs de 1 MiB; scripts/check-disk-full.sh"]
fn real_full_filesystem_keeps_confirmed_data_and_cleans_incomplete_upload() {
    let parent = std::env::var_os("CREWSYNC_FULL_DISK_DIR").expect("defina o tmpfs isolado");
    let temp = tempfile::tempdir_in(parent).unwrap();
    let root = temp.path().join("store");
    let mut store = Store::open(&root).unwrap();
    let hash = crewsync_protocol::disk::hash_secret("abc").unwrap();
    store.put_object(&b"abc"[..], &hash, 3).unwrap();
    store
        .apply(
            "Obras",
            "confirmed",
            0,
            Content::File {
                hash: hash.clone(),
                size: 3,
            },
        )
        .unwrap();
    let result = store.put_object(
        io::Read::take(io::repeat(b'x'), 8 * 1024 * 1024),
        &hash,
        8 * 1024 * 1024,
    );
    assert_eq!(result.unwrap_err().kind(), io::ErrorKind::StorageFull);
    drop(store);
    let store = Store::open(&root).unwrap();
    assert_eq!(store.manifest("Obras").unwrap().revision, 1);
    assert_eq!(fs::read_dir(root.join("tmp")).unwrap().count(), 0);
    let mut content = Vec::new();
    io::Read::read_to_end(&mut store.object(&hash, 3).unwrap(), &mut content).unwrap();
    assert_eq!(content, b"abc");
}
