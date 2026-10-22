use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_crewsync-server"))
        .args(args)
        .output()
        .expect("executar o binário de teste")
}

#[test]
fn help_reports_current_limitations() {
    let output = run(&["--help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("saída UTF-8");
    assert!(stdout.contains("Uso: crewsync-server"));
    assert!(stdout.contains("ainda não implementadas"));
    assert!(output.stderr.is_empty());
}

#[test]
fn version_matches_package() {
    let output = run(&["--version"]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).expect("saída UTF-8"),
        format!("crewsync-server {}\n", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn unsupported_invocations_fail() {
    for args in [
        vec![],
        vec!["--unknown"],
        vec!["--help", "extra"],
        vec!["--version", "extra"],
    ] {
        let output = run(&args);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}

#[test]
fn check_config_validates_example_without_opening_storage() {
    let example = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../config/server.example.toml"
    );
    let output = run(&["check-config", example]);
    assert!(output.status.success());
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("3 usuários, 2 compartilhamentos")
    );
    assert!(output.stderr.is_empty());
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("not-created");
    let source = dir.path().join("server.toml");
    std::fs::write(
        &source,
        format!("version=1\n[server]\nstorage_root='{}'\n", root.display()),
    )
    .unwrap();
    assert!(
        run(&["check-config", source.to_str().unwrap()])
            .status
            .success()
    );
    assert!(!root.exists());
}

#[test]
fn check_config_errors_do_not_echo_configuration_values() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("server.toml");
    std::fs::write(&source, "version = 'private-marker'").unwrap();
    let output = run(&["check-config", source.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Configuração inválida"));
    assert!(!stderr.contains("private-marker"));
    assert_eq!(
        run(&["check-config", dir.path().join("missing").to_str().unwrap()])
            .status
            .code(),
        Some(1)
    );
    assert_eq!(run(&["check-config"]).status.code(), Some(2));
    assert_eq!(
        run(&["check-config", "file", "extra"]).status.code(),
        Some(2)
    );
}
