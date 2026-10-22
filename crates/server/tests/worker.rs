//! Worker real em processo separado e framing de streaming, sem sockets.
use crewsync_protocol::{Frame, Operation, Request, Response};
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use tempfile::TempDir;
const ABC: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
struct Worker {
    child: Child,
    input: Option<ChildStdin>,
    output: BufReader<ChildStdout>,
}
impl Drop for Worker {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
impl Worker {
    fn open(config: &Path) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_crewsync-server"))
            .arg("worker")
            .arg(config)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let input = child.stdin.take();
        let mut output = BufReader::new(child.stdout.take().unwrap());
        let mut ready = String::new();
        output.read_line(&mut ready).unwrap();
        assert_eq!(ready, "READY\n");
        Self {
            child,
            input,
            output,
        }
    }
    fn request(
        &mut self,
        token: Option<&str>,
        operation: Operation,
        body: &[u8],
    ) -> (u16, Vec<u8>) {
        let frame = Frame {
            request: Request {
                token: token.map(str::to_owned),
                operation,
            },
            body_bytes: body.len() as u64,
        };
        let input = self.input.as_mut().unwrap();
        serde_json::to_writer(&mut *input, &frame).unwrap();
        input.write_all(b"\n").unwrap();
        input.write_all(body).unwrap();
        input.flush().unwrap();
        let mut line = String::new();
        self.output.read_line(&mut line).unwrap();
        let response: Response = serde_json::from_str(&line).unwrap();
        let mut bytes = vec![0; response.length as usize];
        self.output.read_exact(&mut bytes).unwrap();
        (response.status, bytes)
    }
}
fn setup() -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let config = dir.path().join("server.toml");
    fs::write(&config, format!("version=1\n[server]\nstorage_root='{}'\n[users.ana]\nenabled=true\n[users.reader]\nenabled=true\n[shares.Obras.permissions]\nana='write'\nreader='read'\n[shares.Private.permissions]\nana='write'\n",dir.path().join("store").display())).unwrap();
    (dir, config)
}
fn invite(config: &Path, user: &str) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_crewsync-server"))
        .arg("invite")
        .arg(config)
        .args([user, "600"])
        .output()
        .unwrap();
    assert!(out.status.success());
    String::from_utf8(out.stdout).unwrap().trim().to_owned()
}
fn activate(worker: &mut Worker, code: &str) -> serde_json::Value {
    let (status, body) = worker.request(
        None,
        Operation::Activate {
            code: code.to_owned(),
            name: "test-pc".into(),
        },
        &[],
    );
    assert_eq!(status, 200);
    serde_json::from_slice(&body).unwrap()
}
#[test]
fn authentication_streaming_paths_acl_reloading_revocation_and_restart() {
    let (_dir, config) = setup();
    let code = invite(&config, "ana");
    let reader_code = invite(&config, "reader");
    let mut worker = Worker::open(&config);
    let auth = activate(&mut worker, &code);
    let token = auth["token"].as_str().unwrap();
    assert_eq!(
        worker
            .request(
                None,
                Operation::Activate {
                    code,
                    name: "duplicate".into()
                },
                &[]
            )
            .0,
        401
    );
    let reader = activate(&mut worker, &reader_code);
    let reader_token = reader["token"].as_str().unwrap();
    assert_eq!(worker.request(None, Operation::Shares, &[]).0, 401);
    assert_eq!(
        worker
            .request(Some(&"0".repeat(64)), Operation::Shares, &[])
            .0,
        401
    );
    assert_eq!(
        worker
            .request(
                Some(token),
                Operation::Put {
                    share: "Obras".into(),
                    path: "file".into(),
                    base: 0,
                    hash: ABC.into(),
                    size: 3
                },
                b"abc"
            )
            .0,
        200
    );
    let (status, bytes) = worker.request(
        Some(reader_token),
        Operation::Get {
            share: "Obras".into(),
            path: "file".into(),
        },
        &[],
    );
    assert_eq!(status, 200);
    assert_eq!(bytes, b"abc");
    assert_eq!(
        worker
            .request(
                Some(reader_token),
                Operation::Put {
                    share: "Obras".into(),
                    path: "blocked".into(),
                    base: 0,
                    hash: ABC.into(),
                    size: 3
                },
                b"abc"
            )
            .0,
        403
    );
    // Próxima requisição continua alinhada mesmo depois de recusar corpo anterior.
    assert_eq!(
        worker
            .request(
                Some(reader_token),
                Operation::Manifest {
                    share: "Obras".into()
                },
                &[]
            )
            .0,
        200
    );
    assert_eq!(
        worker
            .request(
                Some(reader_token),
                Operation::Manifest {
                    share: "Private".into()
                },
                &[]
            )
            .0,
        403
    );
    assert_eq!(
        worker
            .request(
                Some(token),
                Operation::Mkdir {
                    share: "Obras".into(),
                    path: "../escape".into(),
                    base: 0
                },
                &[]
            )
            .0,
        400
    );
    assert_eq!(
        worker
            .request(
                Some(token),
                Operation::Put {
                    share: "Obras".into(),
                    path: "bad".into(),
                    base: 0,
                    hash: ABC.into(),
                    size: 3
                },
                b"bad"
            )
            .0,
        400
    );
    drop(worker);
    let mut worker = Worker::open(&config);
    assert_eq!(
        worker
            .request(
                Some(token),
                Operation::Get {
                    share: "Obras".into(),
                    path: "file".into()
                },
                &[]
            )
            .1,
        b"abc"
    );
    let original = fs::read_to_string(&config).unwrap();
    fs::write(
        &config,
        original.replace(
            "[users.reader]\nenabled=true",
            "[users.reader]\nenabled=false",
        ),
    )
    .unwrap();
    assert_eq!(
        worker.request(Some(reader_token), Operation::Shares, &[]).0,
        401
    );
    fs::write(&config, "malformed").unwrap();
    assert_eq!(
        worker.request(Some(reader_token), Operation::Shares, &[]).0,
        401
    );
    fs::write(&config, original).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_crewsync-server"))
        .arg("revoke")
        .arg(&config)
        .arg(auth["device_id"].as_str().unwrap())
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(worker.request(Some(token), Operation::Shares, &[]).0, 401);
    assert_eq!(
        worker.request(Some(reader_token), Operation::Shares, &[]).0,
        200
    );
}
#[test]
fn interrupted_upload_is_not_published_and_recovers_after_restart() {
    let (dir, config) = setup();
    let code = invite(&config, "ana");
    let mut worker = Worker::open(&config);
    let auth = activate(&mut worker, &code);
    let token = auth["token"].as_str().unwrap();
    let frame = Frame {
        request: Request {
            token: Some(token.into()),
            operation: Operation::Put {
                share: "Obras".into(),
                path: "file".into(),
                base: 0,
                hash: ABC.into(),
                size: 100,
            },
        },
        body_bytes: 100,
    };
    let mut input = worker.input.take().unwrap();
    serde_json::to_writer(&mut input, &frame).unwrap();
    input.write_all(b"\nabc").unwrap();
    drop(input);
    assert!(!worker.child.wait().unwrap().success());
    drop(worker);
    let mut worker = Worker::open(&config);
    let (status, body) = worker.request(
        Some(token),
        Operation::Manifest {
            share: "Obras".into(),
        },
        &[],
    );
    assert_eq!(status, 200);
    let manifest: crewsync_protocol::Manifest = serde_json::from_slice(&body).unwrap();
    assert!(manifest.entries.is_empty());
    assert_eq!(
        fs::read_dir(dir.path().join("store/tmp")).unwrap().count(),
        0
    );
}
