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

#[test]
fn activation_confirms_on_enter_while_stdin_stays_open() {
    use std::io::Write;
    use std::process::Stdio;
    use std::sync::mpsc;
    use std::time::Duration;
    let directory = tempfile::tempdir().unwrap();
    // URL inválida é recusada antes de abrir socket: testa entrada e progresso.
    let mut child = Command::new(env!("CARGO_BIN_EXE_crewsync-client"))
        .args(["activate", "http://invalid", "unused-ca"])
        .arg(directory.path().join("session.json"))
        .arg("pc")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    let secret = "a".repeat(64);
    writeln!(input, "{secret}").unwrap();
    let (sender, receiver) = mpsc::channel();
    let thread = std::thread::spawn(move || {
        let _ = sender.send(child.wait_with_output());
    });
    let result = receiver.recv_timeout(Duration::from_secs(5));
    // Em regressão, EOF libera a leitura antiga e permite limpar o subprocesso.
    drop(input);
    let output = match result {
        Ok(result) => result.unwrap(),
        Err(_) => {
            let _ = receiver.recv_timeout(Duration::from_secs(5));
            thread.join().unwrap();
            panic!("Enter deve confirmar sem aguardar EOF");
        }
    };
    thread.join().unwrap();
    assert_eq!(output.status.code(), Some(1));
    let message = String::from_utf8(output.stderr).unwrap();
    assert!(message.contains("pressione Enter"));
    assert!(message.contains("Ativando dispositivo"));
    assert!(message.contains("HTTP 400"));
    assert!(!message.contains(&secret));
    assert!(!String::from_utf8(output.stdout).unwrap().contains(&secret));
    assert!(!directory.path().join("session.json").exists());
}
