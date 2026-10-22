use std::fs;
use std::io::{self, Read};
use std::path::Path;

use crewsync_server::access::AuthorizedStore;
use crewsync_server::storage::Content;
use tempfile::TempDir;

const ABC: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

fn configuration(root: &Path) -> String {
    format!(
        r#"
version = 1
[server]
storage_root = '{}'
max_upload_bytes = 10
[users.writer]
enabled = true
[users.reader]
enabled = true
[users.disabled]
enabled = false
[users.no_access]
enabled = true
[shares.Obras.permissions]
writer = 'write'
reader = 'read'
disabled = 'write'
[shares.Privado.permissions]
writer = 'write'
"#,
        root.display()
    )
}

fn fixture() -> (TempDir, AuthorizedStore) {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("server.toml");
    fs::write(&path, configuration(&dir.path().join("store"))).unwrap();
    let server = AuthorizedStore::open(path).unwrap();
    (dir, server)
}

fn denied<T>(result: io::Result<T>) {
    assert!(matches!(result, Err(e) if e.kind() == io::ErrorKind::PermissionDenied));
}

struct Unreadable;
impl Read for Unreadable {
    fn read(&mut self, _bytes: &mut [u8]) -> io::Result<usize> {
        panic!("upload não autorizado ou conflitante não deve ser lido");
    }
}

#[test]
fn writer_creates_reader_reads_and_only_writer_can_change_or_delete() {
    let (_dir, mut server) = fixture();
    let parent = server
        .create_directory("writer", "Obras", "Projetos", 0)
        .unwrap();
    let rev = server
        .put_file("writer", "Obras", "Projetos/planta", 0, ABC, &b"abc"[..])
        .unwrap();
    let mut bytes = Vec::new();
    server
        .file("reader", "Obras", "Projetos/planta")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    assert_eq!(bytes, b"abc");
    assert_eq!(server.manifest("reader", "Obras").unwrap().revision, rev);
    denied(server.put_file("reader", "Obras", "Projetos/planta", rev, ABC, Unreadable));
    denied(server.create_directory("reader", "Obras", "outro", 0));
    denied(server.delete("reader", "Obras", "Projetos/planta", rev));
    assert_eq!(server.manifest("writer", "Obras").unwrap().revision, rev);
    server
        .delete("writer", "Obras", "Projetos/planta", rev)
        .unwrap();
    server
        .delete("writer", "Obras", "Projetos", parent)
        .unwrap();
    assert_eq!(
        server.manifest("reader", "Obras").unwrap().entries["Projetos/planta"].content,
        Content::Deleted
    );
    assert!(server.file("reader", "Obras", "Projetos/planta").is_err());
}

#[test]
fn all_operations_deny_unknown_disabled_or_ungranted_identities() {
    let (_dir, mut server) = fixture();
    for user in ["unknown", "disabled", "no_access", ""] {
        assert!(server.shares(user).is_empty());
        denied(server.manifest(user, "Obras"));
        denied(server.file(user, "Obras", "file"));
        denied(server.put_file(user, "Obras", "file", 0, ABC, Unreadable));
        denied(server.create_directory(user, "Obras", "dir", 0));
        denied(server.delete(user, "Obras", "file", 0));
    }
    for share in ["unknown", "Obras/subpasta", "../Obras"] {
        denied(server.manifest("writer", share));
        denied(server.put_file("writer", share, "file", 0, ABC, Unreadable));
        denied(server.create_directory("writer", share, "dir", 0));
        denied(server.delete("writer", share, "file", 0));
        denied(server.file("writer", share, "file"));
    }
}

#[test]
fn global_objects_cannot_be_read_through_another_share_or_hash() {
    let (_dir, mut server) = fixture();
    server
        .put_file("writer", "Privado", "secret", 0, ABC, &b"abc"[..])
        .unwrap();
    assert_eq!(server.shares("reader"), ["Obras"]);
    denied(server.file("reader", "Privado", "secret"));
    assert_eq!(
        server.file("reader", "Obras", ABC).unwrap_err().kind(),
        io::ErrorKind::NotFound
    );
    assert_eq!(
        server.file("reader", "Obras", "secret").unwrap_err().kind(),
        io::ErrorKind::NotFound
    );
    denied(server.file("reader", "Privado", "../Obras/secret"));
    assert!(server.file("reader", "Obras", "../Privado/secret").is_err());
    assert!(
        server
            .manifest("reader", "Obras")
            .unwrap()
            .entries
            .is_empty()
    );
}

#[test]
fn upload_limits_paths_hashes_and_conflicts_keep_manifest_unchanged() {
    let (dir, mut server) = fixture();
    assert!(
        server
            .put_file("writer", "Obras", "../file", 0, ABC, Unreadable)
            .is_err()
    );
    assert!(
        server
            .put_file("writer", "Obras", "file", 0, ABC, &b"12345678901"[..])
            .is_err()
    );
    assert!(
        server
            .put_file("writer", "Obras", "file", 0, ABC, &b"bad"[..])
            .is_err()
    );
    assert!(
        server
            .manifest("writer", "Obras")
            .unwrap()
            .entries
            .is_empty()
    );
    let rev = server
        .put_file("writer", "Obras", "file", 0, ABC, &b"abc"[..])
        .unwrap();
    assert_eq!(
        server
            .put_file("writer", "Obras", "file", 0, ABC, Unreadable)
            .unwrap_err()
            .kind(),
        io::ErrorKind::AlreadyExists
    );
    server.delete("writer", "Obras", "file", rev).unwrap();
    drop(server);
    let mut server = AuthorizedStore::open(dir.path().join("server.toml")).unwrap();
    assert_eq!(
        server
            .put_file("writer", "Obras", "file", 0, ABC, Unreadable)
            .unwrap_err()
            .kind(),
        io::ErrorKind::AlreadyExists
    );
}

#[test]
fn invalid_reload_keeps_previous_configuration_and_storage() {
    let (dir, mut server) = fixture();
    server
        .create_directory("writer", "Obras", "dir", 0)
        .unwrap();
    let before = server.manifest("reader", "Obras").unwrap();
    let original = configuration(&dir.path().join("store"));
    let source = dir.path().join("server.toml");
    for document in [
        "not valid TOML".to_owned(),
        original.replace("'read'", "'owner'"),
        original
            .replace(
                "[users.reader]\nenabled = true",
                "[users.reader]\nenabled = false",
            )
            .replace("'read'", "'owner'"),
        original.replace(
            dir.path().join("store").to_str().unwrap(),
            dir.path().join("elsewhere").to_str().unwrap(),
        ),
    ] {
        fs::write(&source, document).unwrap();
        assert!(server.reload_config().is_err());
        assert_eq!(server.manifest("reader", "Obras").unwrap(), before);
        assert_eq!(server.shares("reader"), ["Obras"]);
    }
    fs::remove_file(&source).unwrap();
    let other = dir.path().join("other.toml");
    fs::write(&other, original).unwrap();
    std::os::unix::fs::symlink(&other, &source).unwrap();
    assert!(server.reload_config().is_err());
    assert_eq!(server.manifest("reader", "Obras").unwrap(), before);
    fs::remove_file(&source).unwrap();
    assert!(server.reload_config().is_err());
    assert_eq!(server.manifest("reader", "Obras").unwrap(), before);
    assert!(!dir.path().join("elsewhere").exists());
}

#[test]
fn valid_reload_revokes_downgrades_and_disables_without_restart() {
    let (dir, mut server) = fixture();
    let original = configuration(&dir.path().join("store"));
    let source = dir.path().join("server.toml");
    let rev = server
        .put_file("writer", "Obras", "file", 0, ABC, &b"abc"[..])
        .unwrap();
    fs::write(
        &source,
        original.replace("writer = 'write'", "writer = 'read'"),
    )
    .unwrap();
    server.reload_config().unwrap();
    server.file("writer", "Obras", "file").unwrap();
    denied(server.delete("writer", "Obras", "file", rev));
    denied(server.put_file("writer", "Obras", "file", rev, ABC, Unreadable));
    fs::write(
        &source,
        original.replace(
            "[users.reader]\nenabled = true",
            "[users.reader]\nenabled = false",
        ),
    )
    .unwrap();
    server.reload_config().unwrap();
    denied(server.manifest("reader", "Obras"));
    denied(server.file("reader", "Obras", "file"));
    fs::write(&source, original.replace("reader = 'read'", "")).unwrap();
    server.reload_config().unwrap();
    denied(server.manifest("reader", "Obras"));
    fs::write(&source, original).unwrap();
    server.reload_config().unwrap();
    server.manifest("reader", "Obras").unwrap();
    server.delete("writer", "Obras", "file", rev).unwrap();
}

#[test]
fn reload_can_remove_user_share_and_reduce_upload_limit() {
    let (dir, mut server) = fixture();
    let original = configuration(&dir.path().join("store"));
    let source = dir.path().join("server.toml");
    server
        .create_directory("writer", "Privado", "preservado", 0)
        .unwrap();
    fs::write(
        &source,
        original
            .replace("[users.reader]\nenabled = true", "")
            .replace("reader = 'read'", ""),
    )
    .unwrap();
    server.reload_config().unwrap();
    denied(server.manifest("reader", "Obras"));
    fs::write(
        &source,
        original.replace("[shares.Privado.permissions]\nwriter = 'write'", ""),
    )
    .unwrap();
    server.reload_config().unwrap();
    denied(server.manifest("writer", "Privado"));
    fs::write(
        &source,
        original.replace("max_upload_bytes = 10", "max_upload_bytes = 2"),
    )
    .unwrap();
    server.reload_config().unwrap();
    assert!(
        server
            .manifest("writer", "Privado")
            .unwrap()
            .entries
            .contains_key("preservado")
    );
    assert!(
        server
            .put_file("writer", "Obras", "file", 0, ABC, &b"abc"[..])
            .is_err()
    );
    assert!(
        server
            .manifest("writer", "Obras")
            .unwrap()
            .entries
            .is_empty()
    );
}

#[test]
fn invalid_initial_configuration_does_not_create_storage() {
    let dir = TempDir::new().unwrap();
    let source = dir.path().join("server.toml");
    fs::write(
        &source,
        configuration(&dir.path().join("store")).replace("'read'", "'admin'"),
    )
    .unwrap();
    assert!(AuthorizedStore::open(source).is_err());
    assert!(!dir.path().join("store").exists());
}

#[test]
fn history_and_restore_enforce_current_permissions_and_share_isolation() {
    let (_dir, mut server) = fixture();
    let rev = server
        .put_file("writer", "Obras", "file", 0, ABC, &b"abc"[..])
        .unwrap();
    let deleted = server.delete("writer", "Obras", "file", rev).unwrap();
    assert_eq!(server.history("reader", "Obras", "file").unwrap().len(), 2);
    denied(server.restore("reader", "Obras", "file", rev, deleted));
    for user in ["unknown", "disabled", "no_access"] {
        denied(server.history(user, "Obras", "file"));
        denied(server.restore(user, "Obras", "file", rev, deleted));
    }
    denied(server.history("reader", "Privado", "file"));
    denied(server.restore("reader", "Privado", "file", rev, 0));
    server
        .restore("writer", "Obras", "file", rev, deleted)
        .unwrap();
}
