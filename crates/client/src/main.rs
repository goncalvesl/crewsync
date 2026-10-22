use crewsync_client::{Session, activate_with, read_code, sync};
use std::env;
use std::io;
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.len() == 1 {
        if args[0] == "--help" {
            println!(
                "crewsync-client — Cliente de sincronização (em desenvolvimento)\n\nUso: crewsync-client --help | --version\n  activate URL CA|system|insecure-http SESSION NOME (convite por stdin)\n  shares SESSION\n  history SESSION SHARE CAMINHO\n  restore SESSION SHARE CAMINHO REVISAO BASE\n  sync-root SESSION RAIZ\n  watch-root SESSION RAIZ [SEGUNDOS]\n  sync SESSION SHARE PASTA\n  watch SESSION SHARE PASTA [SEGUNDOS]\n\nCliente HTTP/HTTPS para piloto Linux. Retenção automática ainda não implementada."
            );
            return ExitCode::SUCCESS;
        }
        if args[0] == "--version" {
            println!("crewsync-client {}", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
    }
    let result = match args.first().and_then(|a| a.to_str()) {
        Some("activate") if args.len() == 5 => Some((|| {
            eprintln!("Cole o código de convite e pressione Enter:");
            let code = read_code(io::stdin().lock())?;
            eprintln!("Ativando dispositivo no servidor...");
            let trust = if args[2] == "system" {
                crewsync_protocol::transport::Trust::System
            } else if args[2] == "insecure-http" {
                eprintln!(
                    "HTTP sem criptografia: convites, credenciais e arquivos trafegam em texto claro."
                );
                crewsync_protocol::transport::Trust::PlainHttp
            } else {
                crewsync_protocol::transport::Trust::CustomCa(Path::new(&args[2]))
            };
            activate_with(
                &args[1].to_string_lossy(),
                trust,
                Path::new(&args[3]),
                &args[4].to_string_lossy(),
                &code,
            )?;
            println!("Dispositivo ativado; credencial salva com permissão 0600.");
            Ok(())
        })()),
        Some("shares") if args.len() == 2 => Some((|| {
            let session = Session::load(Path::new(&args[1]))?;
            let shares: Vec<String> = session.call(crewsync_protocol::Operation::Shares)?;
            for share in shares {
                println!("{share}");
            }
            Ok(())
        })()),
        Some("history") if args.len() == 4 => Some((|| {
            let session = Session::load(Path::new(&args[1]))?;
            let entries: Vec<crewsync_protocol::Entry> =
                session.call(crewsync_protocol::Operation::History {
                    share: args[2].to_string_lossy().into_owned(),
                    path: args[3].to_string_lossy().into_owned(),
                })?;
            for entry in entries {
                let description = match entry.content {
                    crewsync_protocol::Content::File { size, .. } => {
                        format!("arquivo ({size} bytes)")
                    }
                    crewsync_protocol::Content::Directory => "diretório".into(),
                    crewsync_protocol::Content::Deleted => "excluído".into(),
                };
                println!("{}\t{description}", entry.revision);
            }
            Ok(())
        })()),
        Some("restore") if args.len() == 6 => Some((|| {
            let parse = |index: usize| {
                args[index]
                    .to_string_lossy()
                    .parse::<u64>()
                    .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "revisão inválida"))
            };
            let revision = parse(4)?;
            let base = parse(5)?;
            let session = Session::load(Path::new(&args[1]))?;
            let result: u64 = session.call(crewsync_protocol::Operation::Restore {
                share: args[2].to_string_lossy().into_owned(),
                path: args[3].to_string_lossy().into_owned(),
                revision,
                base,
            })?;
            println!(
                "Restaurado no servidor como revisão {result}. Execute sync ou aguarde watch."
            );
            Ok(())
        })()),
        Some("sync" | "watch") if args.len() == 4 || args.len() == 5 => Some(run_sync(&args)),
        Some("sync-root" | "watch-root") if args.len() == 3 || args.len() == 4 => {
            Some(run_sync(&args))
        }
        _ => None,
    };
    if let Some(result) = result {
        return match result {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("Falha: {error}");
                ExitCode::FAILURE
            }
        };
    }
    eprintln!("Uso inválido. Consulte crewsync-client --help.");
    ExitCode::from(2)
}

fn run_sync(args: &[std::ffi::OsString]) -> io::Result<()> {
    let all = args[0] == "sync-root" || args[0] == "watch-root";
    let watch = args[0] == "watch" || args[0] == "watch-root";
    let folder_index = if all { 2 } else { 3 };
    let interval_index = folder_index + 1;
    if !watch && args.len() != interval_index {
        return Err(io::Error::other("argumentos inválidos; use --help"));
    }
    let seconds: u64 = if args.len() > interval_index {
        args[interval_index]
            .to_string_lossy()
            .parse()
            .map_err(|_| io::Error::other("intervalo inválido"))?
    } else {
        5
    };
    if !(1..=3600).contains(&seconds) {
        return Err(io::Error::other(
            "intervalo deve estar entre 1 e 3600 segundos",
        ));
    }
    if std::fs::canonicalize(&args[1])?.starts_with(std::fs::canonicalize(&args[folder_index])?) {
        return Err(io::Error::other(
            "credencial deve ficar fora da pasta sincronizada",
        ));
    }
    let changes = if watch {
        match crewsync_client::watch::Changes::new(Path::new(&args[folder_index])) {
            Ok(changes) => Some(changes),
            Err(_) => {
                eprintln!("Observador indisponível; usando reconciliação periódica.");
                None
            }
        }
    } else {
        None
    };
    let mut delay = seconds;
    loop {
        let session = Session::load(Path::new(&args[1]))?;
        let result = if all {
            crewsync_client::root::sync(&session, Path::new(&args[folder_index]))
        } else {
            sync::sync(
                &session,
                &args[2].to_string_lossy(),
                Path::new(&args[folder_index]),
            )
        };
        let failed = result.is_err();
        match result {
            Ok(report) => {
                println!(
                    "Sincronizado: {} envios, {} recebimentos, {} conflitos preservados em .crewsync/conflicts.",
                    report.uploaded, report.downloaded, report.conflicts
                );
                delay = seconds;
            }
            Err(error)
                if watch
                    && matches!(
                        error.kind(),
                        io::ErrorKind::ConnectionAborted
                            | io::ErrorKind::ConnectionRefused
                            | io::ErrorKind::BrokenPipe
                            | io::ErrorKind::TimedOut
                            | io::ErrorKind::Interrupted
                            | io::ErrorKind::AlreadyExists
                    ) =>
            {
                eprintln!("Reconciliação interrompida; nova tentativa em {delay}s.");
                delay = (delay * 2).min(60);
            }
            Err(error) => return Err(error),
        }
        if !watch {
            return Ok(());
        }
        let interval = std::time::Duration::from_secs(delay);
        if !failed && let Some(changes) = &changes {
            changes.wait(interval);
        } else {
            std::thread::sleep(interval);
        }
    }
}
