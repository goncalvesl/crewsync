//! Servidor e dois clientes em processos separados sobre HTTPS real.
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::{PermissionsExt, symlink};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::mpsc;
use std::time::Duration;
use tempfile::TempDir;

fn server_bin() -> &'static str {
    env!("CARGO_BIN_EXE_crewsync-server")
}
fn client_bin() -> PathBuf {
    Path::new(server_bin()).with_file_name("crewsync-client")
}
fn run(command: &mut Command) -> Output {
    command.output().unwrap()
}
fn success(output: Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}
struct Server(Child);
impl Drop for Server {
    fn drop(&mut self) {
        let _ = Command::new("/bin/kill")
            .args(["-TERM", "--", &format!("-{}", self.0.id())])
            .status();
        let _ = self.0.wait();
    }
}
struct Pilot {
    _dir: TempDir,
    root: PathBuf,
    config: PathBuf,
    cert: PathBuf,
    key: PathBuf,
}
impl Pilot {
    fn new() -> Self {
        let dir = TempDir::new().unwrap();
        let root = dir.path().to_owned();
        let cert = root.join("cert.pem");
        let key = root.join("key.pem");
        success(run(Command::new("openssl")
            .args([
                "req",
                "-x509",
                "-newkey",
                "rsa:2048",
                "-sha256",
                "-nodes",
                "-days",
                "1",
                "-subj",
                "/CN=localhost",
                "-addext",
                "subjectAltName=DNS:localhost",
                "-addext",
                "basicConstraints=critical,CA:FALSE",
                "-keyout",
            ])
            .arg(&key)
            .arg("-out")
            .arg(&cert)));
        let config = root.join("server.toml");
        fs::write(&config, format!("version=1\n[server]\nstorage_root='{}'\nmax_upload_bytes=2097152\n[users.ana]\nenabled=true\n[users.reader]\nenabled=true\n[shares.Obras.permissions]\nana='write'\nreader='read'\n[shares.Privado.permissions]\nana='write'\n", root.join("store").display())).unwrap();
        Self {
            _dir: dir,
            root,
            config,
            cert,
            key,
        }
    }
    fn start(&self) -> (Server, String) {
        let mut child = Command::new(server_bin())
            .arg("serve")
            .arg(&self.config)
            .args(["127.0.0.1", "0"])
            .arg(&self.cert)
            .arg(&self.key)
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .process_group(0)
            .spawn()
            .unwrap();
        let stdout = child.stdout.take().unwrap();
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let mut line = String::new();
            let _ = BufReader::new(stdout).read_line(&mut line);
            let _ = sender.send(line);
        });
        let server = Server(child);
        let line = receiver
            .recv_timeout(Duration::from_secs(10))
            .expect("servidor precisa iniciar em até 10s");
        assert!(line.starts_with("CrewSync HTTPS ready"), "{line}");
        let port = line.split_whitespace().last().unwrap();
        (server, format!("https://localhost:{port}"))
    }
    fn activate(&self, url: &str, user: &str, name: &str) -> PathBuf {
        let code = success(run(Command::new(server_bin())
            .arg("invite")
            .arg(&self.config)
            .args([user, "600"])));
        let session = self.root.join(format!("{name}.json"));
        let mut child = Command::new(client_bin())
            .arg("activate")
            .arg(url)
            .arg(&self.cert)
            .arg(&session)
            .arg(name)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(code.as_bytes())
            .unwrap();
        success(child.wait_with_output().unwrap());
        assert_eq!(
            fs::metadata(&session).unwrap().permissions().mode() & 0o077,
            0
        );
        session
    }
    fn folder(&self, name: &str) -> PathBuf {
        let path = self.root.join(name);
        fs::create_dir(&path).unwrap();
        path
    }
}
fn sync(session: &Path, folder: &Path) -> Output {
    run(Command::new(client_bin())
        .arg("sync")
        .arg(session)
        .arg("Obras")
        .arg(folder))
}
fn stored_copy(dir: &Path, area: &str, expected: &[u8]) -> bool {
    fs::read_dir(dir.join(".crewsync").join(area))
        .unwrap()
        .any(|entry| fs::read(entry.unwrap().path()).ok().as_deref() == Some(expected))
}

#[test]
#[ignore = "requer sockets TCP: executar com --ignored fora do sandbox; CI executa"]
fn two_clients_converge_preserve_conflicts_and_offline_deletions() {
    let pilot = Pilot::new();
    let (server, url) = pilot.start();
    let first = pilot.activate(&url, "ana", "pc1");
    let second = pilot.activate(&url, "ana", "pc2");
    let a = pilot.folder("a");
    let b = pilot.folder("b");
    fs::create_dir(a.join("vazia")).unwrap();
    fs::write(a.join("planta.txt"), b"initial").unwrap();
    fs::write(a.join("grande.bin"), vec![b'x'; 1024 * 1024]).unwrap();
    success(sync(&first, &a));
    success(sync(&second, &b));
    assert!(b.join("vazia").is_dir());
    assert_eq!(
        fs::read(b.join("grande.bin")).unwrap(),
        fs::read(a.join("grande.bin")).unwrap()
    );
    fs::write(a.join("planta.txt"), b"version a").unwrap();
    fs::write(b.join("planta.txt"), b"version b").unwrap();
    success(sync(&first, &a));
    let report = success(sync(&second, &b));
    assert!(report.contains("1 conflitos"), "{report}");
    assert_eq!(fs::read(b.join("planta.txt")).unwrap(), b"version a");
    assert!(stored_copy(&b, "conflicts", b"version b"));
    let stable = success(sync(&second, &b));
    assert!(stable.contains("0 envios, 0 recebimentos"), "{stable}");
    // Exclusão e reconexão de cliente com baseline antigo.
    fs::remove_file(a.join("planta.txt")).unwrap();
    fs::remove_dir(a.join("vazia")).unwrap();
    success(sync(&first, &a));
    success(sync(&second, &b));
    assert!(!b.join("planta.txt").exists());
    assert!(!b.join("vazia").exists());
    assert!(stored_copy(&b, "trash", b"version a"));
    // Rename como exclusão/criação e deduplicação.
    fs::rename(a.join("grande.bin"), a.join("movido.bin")).unwrap();
    success(sync(&first, &a));
    success(sync(&second, &b));
    assert!(!b.join("grande.bin").exists());
    assert_eq!(
        fs::metadata(b.join("movido.bin")).unwrap().len(),
        1024 * 1024
    );
    assert_eq!(
        fs::read_dir(pilot.root.join("store/objects"))
            .unwrap()
            .count(),
        3
    );
    drop(server);
    let (server, new_url) = pilot.start();
    // Mesmo servidor reiniciado em outra porta: adaptar identidade só neste teste.
    for session in [&first, &second] {
        let mut data: serde_json::Value =
            serde_json::from_slice(&fs::read(session).unwrap()).unwrap();
        data["url"] = new_url.clone().into();
        fs::write(session, serde_json::to_vec(&data).unwrap()).unwrap();
    }
    let c = pilot.folder("after-restart");
    success(sync(&first, &c));
    assert_eq!(
        fs::metadata(c.join("movido.bin")).unwrap().len(),
        1024 * 1024
    );
    drop(server);
}

#[test]
#[ignore = "requer sockets TCP: executar com --ignored fora do sandbox; CI executa"]
fn permissions_revocation_https_and_symlinks_fail_closed() {
    let pilot = Pilot::new();
    let (_server, url) = pilot.start();
    let writer = pilot.activate(&url, "ana", "writer");
    let reader = pilot.activate(&url, "reader", "reader");
    let a = pilot.folder("a");
    let b = pilot.folder("b");
    fs::write(a.join("file"), b"content").unwrap();
    success(sync(&writer, &a));
    success(sync(&reader, &b));
    fs::write(b.join("file"), b"blocked").unwrap();
    assert!(!sync(&reader, &b).status.success());
    assert_eq!(fs::read(b.join("file")).unwrap(), b"blocked");
    assert!(
        !run(Command::new(client_bin())
            .arg("sync")
            .arg(&reader)
            .arg("Privado")
            .arg(&b))
        .status
        .success()
    );
    let outside = pilot.root.join("outside");
    fs::write(&outside, b"untouched").unwrap();
    symlink(&outside, a.join("escape")).unwrap();
    assert!(!sync(&writer, &a).status.success());
    assert_eq!(fs::read(outside).unwrap(), b"untouched");
    fs::remove_file(a.join("escape")).unwrap();
    let original = fs::read(&writer).unwrap();
    let mut data: serde_json::Value = serde_json::from_slice(&original).unwrap();
    data["url"] = url.replace("localhost", "127.0.0.1").into();
    fs::write(&writer, serde_json::to_vec(&data).unwrap()).unwrap();
    assert!(
        !run(Command::new(client_bin()).arg("shares").arg(&writer))
            .status
            .success()
    );
    data["url"] = url.replace("https:", "http:").into();
    fs::write(&writer, serde_json::to_vec(&data).unwrap()).unwrap();
    assert!(
        !run(Command::new(client_bin()).arg("shares").arg(&writer))
            .status
            .success()
    );
    fs::write(&writer, &original).unwrap();
    let data: serde_json::Value = serde_json::from_slice(&original).unwrap();
    success(run(Command::new(server_bin())
        .arg("revoke")
        .arg(&pilot.config)
        .arg(data["device_id"].as_str().unwrap())));
    assert!(!sync(&writer, &a).status.success());
    let raw = fs::read_to_string(pilot.root.join("store/auth/state.json")).unwrap();
    assert!(!raw.contains(data["token"].as_str().unwrap()));
}

#[test]
#[ignore = "requer sockets TCP: executar com --ignored fora do sandbox; CI executa"]
fn startup_errors_report_tls_port_configuration_and_private_storage() {
    use std::net::TcpListener;
    fn failure(pilot: &Pilot, cert: &Path, key: &Path, port: u16) -> String {
        let child = Command::new(server_bin())
            .arg("serve")
            .arg(&pilot.config)
            .arg("127.0.0.1")
            .arg(port.to_string())
            .arg(cert)
            .arg(key)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0)
            .spawn()
            .unwrap();
        let id = child.id();
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = sender.send(child.wait_with_output());
        });
        let result = match receiver.recv_timeout(Duration::from_secs(10)) {
            Ok(result) => result.unwrap(),
            Err(_) => {
                let _ = Command::new("/bin/kill")
                    .args(["-KILL", "--", &format!("-{id}")])
                    .status();
                panic!("falha de inicialização deve encerrar o processo em até 10s");
            }
        };
        assert!(!result.status.success());
        assert!(result.stdout.is_empty());
        let diagnostic = String::from_utf8(result.stderr).unwrap();
        assert!(
            diagnostic.contains("Falha ao iniciar HTTPS:"),
            "{diagnostic}"
        );
        assert!(
            !diagnostic.contains("private-marker"),
            "não expor valores da configuração"
        );
        diagnostic
    }
    let pilot = Pilot::new();
    let absent = pilot.root.join("private-marker-missing.pem");
    assert!(
        failure(&pilot, &absent, &pilot.key, 0).contains("certificado ou chave TLS não encontrado")
    );
    let bad_key = pilot.root.join("bad-key.pem");
    fs::write(&bad_key, b"private-marker-not-a-key").unwrap();
    assert!(failure(&pilot, &pilot.cert, &bad_key, 0).contains("inválidos ou incompatíveis"));
    let storage = pilot.root.join("store");
    fs::create_dir(&storage).unwrap();
    fs::set_permissions(&storage, fs::Permissions::from_mode(0o755)).unwrap();
    let message = failure(&pilot, &pilot.cert, &pilot.key, 0);
    assert!(
        message.contains("0700") && message.contains("raiz de armazenamento"),
        "{message}"
    );
    fs::set_permissions(&storage, fs::Permissions::from_mode(0o700)).unwrap();
    let held_store = crewsync_server::access::AuthorizedStore::open(&pilot.config).unwrap();
    assert!(
        failure(&pilot, &pilot.cert, &pilot.key, 0)
            .contains("armazenamento já está em uso por outro processo")
    );
    drop(held_store);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    assert!(
        failure(
            &pilot,
            &pilot.cert,
            &pilot.key,
            listener.local_addr().unwrap().port()
        )
        .contains("já está em uso")
    );
    fs::write(&pilot.config, "version = 'private-marker'").unwrap();
    let message = failure(&pilot, &pilot.cert, &pilot.key, 0);
    assert!(
        message.contains("configuração") && message.contains("esquema"),
        "{message}"
    );
}

#[test]
#[ignore = "requer sockets TCP: executar com --ignored; CI executa"]
fn history_restores_deleted_and_changed_files_to_both_clients() {
    let pilot = Pilot::new();
    let (_server, url) = pilot.start();
    let first = pilot.activate(&url, "ana", "history-pc1");
    let second = pilot.activate(&url, "ana", "history-pc2");
    let reader = pilot.activate(&url, "reader", "history-reader");
    let a = pilot.folder("a");
    let b = pilot.folder("b");
    fs::write(a.join("planta.txt"), b"original").unwrap();
    success(sync(&first, &a));
    success(sync(&second, &b));
    fs::write(a.join("planta.txt"), b"alterada").unwrap();
    success(sync(&first, &a));
    success(sync(&second, &b));
    fs::remove_file(a.join("planta.txt")).unwrap();
    success(sync(&first, &a));
    success(sync(&second, &b));
    let history = success(run(Command::new(client_bin())
        .arg("history")
        .arg(&reader)
        .args(["Obras", "planta.txt"])));
    assert_eq!(history.lines().count(), 3);
    assert!(history.starts_with("1\tarquivo"), "{history}");
    assert!(history.contains("3\texcluído"), "{history}");
    let restore = |session: &Path, revision: &str, base: &str| {
        run(Command::new(client_bin())
            .arg("restore")
            .arg(session)
            .args(["Obras", "planta.txt", revision, base]))
    };
    assert!(!restore(&reader, "1", "3").status.success());
    assert!(!restore(&first, "1", "2").status.success());
    assert!(!restore(&first, "3", "3").status.success());
    assert!(success(restore(&first, "1", "3")).contains("revisão 4"));
    success(sync(&first, &a));
    success(sync(&second, &b));
    assert_eq!(fs::read(a.join("planta.txt")).unwrap(), b"original");
    assert_eq!(fs::read(b.join("planta.txt")).unwrap(), b"original");
    // Restaurar uma versão alterada também cria uma revisão, sem reenviar bytes.
    assert!(success(restore(&second, "2", "4")).contains("revisão 5"));
    success(sync(&first, &a));
    success(sync(&second, &b));
    assert_eq!(fs::read(a.join("planta.txt")).unwrap(), b"alterada");
    assert_eq!(fs::read(b.join("planta.txt")).unwrap(), b"alterada");
    assert_eq!(
        fs::read_dir(pilot.root.join("store/objects"))
            .unwrap()
            .count(),
        2
    );
}

#[test]
#[ignore = "requer sockets TCP: executar com --ignored; CI executa"]
fn truncated_https_upload_restarts_worker_without_publishing_partial_content() {
    use crewsync_protocol::{Manifest, Operation, Request, transport};
    let pilot = Pilot::new();
    let (_server, url) = pilot.start();
    let session = pilot.activate(&url, "ana", "interrupted");
    let data: serde_json::Value = serde_json::from_slice(&fs::read(&session).unwrap()).unwrap();
    let token = data["token"].as_str().unwrap().to_owned();
    let error = transport::exchange(
        &url,
        &pilot.cert,
        Request {
            token: Some(token.clone()),
            operation: Operation::Put {
                share: "Obras".into(),
                path: "partial".into(),
                base: 0,
                hash: crewsync_protocol::disk::hash_secret("abc").unwrap(),
                size: 100,
            },
        },
        &mut &b"abc"[..],
        100,
        &mut Vec::new(),
        4096,
    );
    assert!(error.is_err());
    let manifest: Manifest = transport::json(
        &url,
        &pilot.cert,
        Request {
            token: Some(token),
            operation: Operation::Manifest {
                share: "Obras".into(),
            },
        },
    )
    .unwrap();
    assert!(manifest.entries.is_empty());
    let folder = pilot.folder("healthy");
    fs::write(folder.join("ok"), b"complete").unwrap();
    success(sync(&session, &folder));
    assert_eq!(
        fs::read_dir(pilot.root.join("store/tmp")).unwrap().count(),
        0
    );
}
