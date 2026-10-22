use std::fs;
use std::io;
use std::os::unix::fs::symlink;

use crewsync_server::config::{Config, Operation};
use tempfile::TempDir;

const BASE: &str = "version = 1\n[server]\nstorage_root = '/var/lib/crewsync'\n";

#[test]
fn example_has_explicit_access_and_denies_everything_else() {
    let config = Config::parse(include_bytes!("../../../config/server.example.toml")).unwrap();
    assert_eq!(config.user_count(), 3);
    assert_eq!(config.share_count(), 2);
    assert!(config.allows("ana", "Obras", Operation::Read));
    assert!(config.allows("ana", "Obras", Operation::Write));
    assert!(config.allows("bruno", "Obras", Operation::Read));
    assert!(!config.allows("bruno", "Obras", Operation::Write));
    for user in ["bruno", "carla", "desconhecido", ""] {
        assert!(!config.allows(user, "Administrativo", Operation::Read));
    }
    for share in ["Obras/subpasta", "../Obras", "obras", "inexistente", ""] {
        assert!(!config.allows("ana", share, Operation::Read));
    }
    assert!(!config.allows("carla", "Obras", Operation::Read));
}

#[test]
fn omitted_enabled_or_permissions_deny_access() {
    let config = Config::parse(format!("{BASE}\n[users.ana]\n[users.bruno]\nenabled=true\n[shares.Obras.permissions]\nana='write'\n[shares.Vazio]\n").as_bytes()).unwrap();
    assert!(!config.allows("ana", "Obras", Operation::Write));
    assert!(!config.allows("bruno", "Obras", Operation::Read));
    assert!(!config.allows("bruno", "Vazio", Operation::Read));
    let empty = Config::parse(BASE.as_bytes()).unwrap();
    assert_eq!(empty.user_count(), 0);
    assert!(!empty.allows("ana", "Obras", Operation::Read));
}

#[test]
fn parser_supports_toml_quoting_comments_unicode_and_inline_tables() {
    let source = format!(
        "{BASE}\n[users]\nana = {{ enabled = true }} # comentário\n[shares.'Construção'.permissions]\nana = \"read\"\n"
    );
    let config = Config::parse(source.as_bytes()).unwrap();
    assert!(config.allows("ana", "Construção", Operation::Read));
    assert_eq!(config.max_upload_bytes(), 1_073_741_824);
}

#[test]
fn invalid_syntax_schema_types_and_references_are_rejected_without_echoing_input() {
    let documents = [
        "version=1\nversion=1\n".to_owned(),
        BASE.replace("version = 1", "version = 2"),
        BASE.replace("version = 1", "version = 'private-marker'"),
        BASE.replace("version = 1", "version = true"),
        BASE.replace("version = 1", ""),
        BASE.replace("/var/lib/crewsync", "relative/path"),
        BASE.replace("/var/lib/crewsync", "/"),
        BASE.replace("'/var/lib/crewsync'", "\"/var/\\u0000\""),
        BASE.replace("/var/lib/crewsync", "/var/../private-marker"),
        format!("{BASE}unknown = 'private-marker'\n"),
        format!("{BASE}max_upload_bytes=0\n"),
        format!("{BASE}max_upload_bytes=-1\n"),
        format!("{BASE}max_upload_bytes=1099511627777\n"),
        format!("{BASE}max_upload_bytes='private-marker'\n"),
        format!("{BASE}\n[users.ana]\nenabled='private-marker'\n"),
        format!("{BASE}\n[users.ana]\nenabeld=true\n"),
        format!("{BASE}\n[users.'../ana']\nenabled=true\n"),
        format!("{BASE}\n[shares.Obras.permissions]\nana='read'\n"),
        format!("{BASE}\n[users.ana]\nenabled=true\n[shares.Obras.permissions]\nana='admin'\n"),
        format!("{BASE}\n[users.ana]\nenabled=true\n[shares.Obras.permissions]\nana=true\n"),
        format!("{BASE}\n[shares.'Obras/subpasta']\n"),
        format!("{BASE}\n[shares.'..']\n"),
        format!("{BASE}\n[shares.Obras]\npath='private-marker'\n"),
        format!("private-marker=\"unterminated\n{BASE}"),
    ];
    for source in documents {
        let error = Config::parse(source.as_bytes()).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData, "{source}");
        assert!(!error.to_string().contains("private-marker"));
    }
}

#[test]
fn invalid_utf8_and_oversized_input_are_rejected() {
    assert!(Config::parse(&[0xff]).is_err());
    assert!(Config::parse(&vec![b' '; 1024 * 1024 + 1]).is_err());
}

#[test]
fn loading_rejects_missing_directory_and_symlink_sources() {
    let dir = TempDir::new().unwrap();
    assert!(Config::load(dir.path().join("missing")).is_err());
    assert!(Config::load(dir.path()).is_err());
    let file = dir.path().join("server.toml");
    fs::write(&file, BASE).unwrap();
    Config::load(&file).unwrap();
    let link = dir.path().join("link.toml");
    symlink(file, &link).unwrap();
    assert!(Config::load(link).is_err());
}

#[test]
fn documented_user_and_share_limits_are_enforced() {
    let mut users = BASE.to_owned();
    for number in 0..1000 {
        users.push_str(&format!("\n[users.u{number}]\n"));
    }
    assert_eq!(Config::parse(users.as_bytes()).unwrap().user_count(), 1000);
    users.push_str("\n[users.extra]\n");
    assert!(Config::parse(users.as_bytes()).is_err());
    let mut shares = BASE.to_owned();
    for number in 0..1000 {
        shares.push_str(&format!("\n[shares.s{number}]\n"));
    }
    assert_eq!(
        Config::parse(shares.as_bytes()).unwrap().share_count(),
        1000
    );
    shares.push_str("\n[shares.extra]\n");
    assert!(Config::parse(shares.as_bytes()).is_err());
    for name in ["", "..", "a.b", "ação", &"a".repeat(65)] {
        let source = format!("{BASE}\n[users.'{name}']\n");
        assert!(Config::parse(source.as_bytes()).is_err());
    }
}
