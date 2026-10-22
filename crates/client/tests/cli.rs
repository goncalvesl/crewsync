use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_crewsync-client"))
        .args(args)
        .output()
        .expect("executar o binário de teste")
}

#[test]
fn help_reports_current_limitations() {
    let output = run(&["--help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("saída UTF-8");
    assert!(stdout.contains("Uso: crewsync-client"));
    assert!(stdout.contains("ainda não implementadas"));
    assert!(output.stderr.is_empty());
}

#[test]
fn version_matches_package() {
    let output = run(&["--version"]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).expect("saída UTF-8"),
        format!("crewsync-client {}\n", env!("CARGO_PKG_VERSION"))
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
