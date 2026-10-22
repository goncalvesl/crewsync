use std::env;
use std::process::ExitCode;

use crewsync_server::{auth::AuthStore, config::Config, network};
use std::path::Path;

fn main() -> ExitCode {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.len() == 1 {
        if args[0] == "--help" {
            println!(
                "crewsync-server — Servidor central (em desenvolvimento)\n\nUso: crewsync-server --help | --version | check-config ARQUIVO\n  invite CONFIG USUARIO SEGUNDOS\n  devices CONFIG\n  revoke CONFIG DEVICE_ID\n  serve CONFIG HOST PORT CERT KEY\n  maintain CONFIG [--apply] (servidor parado)\n  backup CONFIG NOVO_DIRETORIO\n  restore BACKUP NOVO_DIRETORIO\n\nServidor HTTPS para piloto Linux. Histórico e retenção disponíveis; use maintain para simular a coleta."
            );
            return ExitCode::SUCCESS;
        }
        if args[0] == "--version" {
            println!("crewsync-server {}", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
    }
    if args.len() == 2 && args[0] == "check-config" {
        return match Config::load(&args[1]) {
            Ok(config) => {
                println!(
                    "Configuração válida: {} usuários, {} compartilhamentos.",
                    config.user_count(),
                    config.share_count()
                );
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("Configuração inválida: {error}");
                ExitCode::FAILURE
            }
        };
    }
    let result = match args.first().and_then(|a| a.to_str()) {
        Some("maintain") if args.len() == 2 || (args.len() == 3 && args[2] == "--apply") => {
            Some((|| {
                let config = Config::load(&args[1])?;
                let mut store = crewsync_server::storage::Store::open(config.storage_root())?;
                let report = store.maintain(config.retention(), args.len() == 3)?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(std::io::Error::other)?
                );
                Ok(())
            })())
        }
        Some("backup") if args.len() == 3 => Some(crewsync_server::backup::create(
            Path::new(&args[1]),
            Path::new(&args[2]),
        )),
        Some("restore") if args.len() == 3 => Some(crewsync_server::backup::restore(
            Path::new(&args[1]),
            Path::new(&args[2]),
        )),
        Some("worker") if args.len() == 2 => Some(network::worker(Path::new(&args[1]))),
        Some("serve") if args.len() == 6 => Some(network::serve(
            Path::new(&args[1]),
            &args[2].to_string_lossy(),
            &args[3].to_string_lossy(),
            Path::new(&args[4]),
            Path::new(&args[5]),
        )),
        Some("invite" | "devices" | "revoke") => Some(admin(&args)),
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
    eprintln!("Uso inválido. Consulte crewsync-server --help.");
    ExitCode::from(2)
}

fn admin(args: &[std::ffi::OsString]) -> std::io::Result<()> {
    let command = args[0].to_string_lossy();
    let expected = match command.as_ref() {
        "invite" => 4,
        "devices" => 2,
        _ => 3,
    };
    if args.len() != expected {
        return Err(std::io::Error::other(
            "argumentos administrativos inválidos; use --help",
        ));
    }
    let config = Config::load(&args[1])?;
    let auth = AuthStore::open(config.storage_root())?;
    match command.as_ref() {
        "invite" => {
            let user = args[2]
                .to_str()
                .ok_or_else(|| std::io::Error::other("usuário inválido"))?;
            if !config.user_enabled(user) {
                return Err(std::io::Error::other("usuário inexistente ou desativado"));
            }
            let ttl = args[3]
                .to_string_lossy()
                .parse()
                .map_err(|_| std::io::Error::other("validade inválida"))?;
            println!("{}", auth.invite(user, ttl)?);
        }
        "devices" => {
            for device in auth.devices()? {
                println!(
                    "{} {} {} {}",
                    device.id,
                    device.user,
                    if device.revoked { "revogado" } else { "ativo" },
                    device.name
                );
            }
        }
        _ => auth.revoke(&args[2].to_string_lossy())?,
    }
    Ok(())
}
