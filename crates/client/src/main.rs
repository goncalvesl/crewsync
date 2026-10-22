use crewsync_client::{Session, activate, read_code, sync};
use std::env;
use std::io;
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.len() == 1 {
        if args[0] == "--help" {
            println!(
                "crewsync-client — Cliente de sincronização (em desenvolvimento)\n\nUso: crewsync-client --help | --version\n  activate URL CA SESSION NOME (convite por stdin)\n  shares SESSION\n  sync SESSION SHARE PASTA\n  watch SESSION SHARE PASTA [SEGUNDOS]\n\nCliente HTTPS para piloto Linux. Histórico e retenção ainda não implementadas."
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
            eprintln!("Ativando dispositivo no servidor HTTPS...");
            activate(
                &args[1].to_string_lossy(),
                Path::new(&args[2]),
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
        Some("sync" | "watch") if args.len() == 4 || args.len() == 5 => Some(run_sync(&args)),
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
    let watch = args[0] == "watch";
    if !watch && args.len() != 4 {
        return Err(io::Error::other("argumentos inválidos; use --help"));
    }
    let seconds: u64 = if args.len() == 5 {
        args[4]
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
    if std::fs::canonicalize(&args[1])?.starts_with(std::fs::canonicalize(&args[3])?) {
        return Err(io::Error::other(
            "credencial deve ficar fora da pasta sincronizada",
        ));
    }
    let mut delay = seconds;
    loop {
        let session = Session::load(Path::new(&args[1]))?;
        let result = sync::sync(&session, &args[2].to_string_lossy(), Path::new(&args[3]));
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
        std::thread::sleep(std::time::Duration::from_secs(delay));
    }
}
