use crewsync_server::{
    auth::AuthStore,
    backup,
    storage::{Content, Store},
};
use std::fs;
#[test]
fn offline_backup_restore_validates_data_tokens_and_never_overwrites() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("source");
    let config = temp.path().join("server.toml");
    fs::write(
        &config,
        format!("version=1\n[server]\nstorage_root='{}'\n", root.display()),
    )
    .unwrap();
    let mut store = Store::open(&root).unwrap();
    let hash = crewsync_protocol::disk::hash_secret("abc").unwrap();
    store.put_object(&b"abc"[..], &hash, 3).unwrap();
    store
        .apply("s", "f", 0, Content::File { hash, size: 3 })
        .unwrap();
    let auth = AuthStore::open(&root).unwrap();
    let invite = auth.invite("u", 600).unwrap();
    let activation = auth.activate(&invite, "pc", |_| true).unwrap();
    let target = temp.path().join("backup");
    assert!(
        backup::create(&config, &target).is_err(),
        "serviço ativo bloqueia backup"
    );
    drop(store);
    backup::create(&config, &target).unwrap();
    assert!(backup::create(&config, &target).is_err());
    let restored = temp.path().join("restore");
    backup::restore(&target, &restored).unwrap();
    assert_eq!(
        Store::open(restored.join("store"))
            .unwrap()
            .manifest("s")
            .unwrap()
            .revision,
        1
    );
    assert_eq!(
        AuthStore::open(&restored.join("store"))
            .unwrap()
            .authenticate(&activation.token)
            .unwrap(),
        "u"
    );
    assert!(backup::restore(&target, &restored).is_err());
    fs::remove_file(target.join("backup.json")).unwrap();
    assert!(backup::restore(&target, &temp.path().join("incomplete")).is_err());
}

#[test]
fn backup_rejects_links_and_restoration_rejects_corrupt_objects() {
    use std::os::unix::fs::symlink;
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("source");
    let config = temp.path().join("server.toml");
    fs::write(
        &config,
        format!("version=1\n[server]\nstorage_root='{}'\n", root.display()),
    )
    .unwrap();
    let mut store = Store::open(&root).unwrap();
    let hash = crewsync_protocol::disk::hash_secret("abc").unwrap();
    store.put_object(&b"abc"[..], &hash, 3).unwrap();
    store
        .apply(
            "s",
            "f",
            0,
            Content::File {
                hash: hash.clone(),
                size: 3,
            },
        )
        .unwrap();
    drop(store);
    symlink(&config, root.join("unexpected-link")).unwrap();
    let bad = temp.path().join("bad-backup");
    assert!(backup::create(&config, &bad).is_err());
    assert!(!bad.join("backup.json").exists());
    fs::remove_file(root.join("unexpected-link")).unwrap();
    let good = temp.path().join("good-backup");
    backup::create(&config, &good).unwrap();
    fs::write(good.join("store/objects").join(hash), b"bad").unwrap();
    let restored = temp.path().join("restore");
    assert!(backup::restore(&good, &restored).is_err());
    assert!(!restored.join("restored.json").exists());
}
