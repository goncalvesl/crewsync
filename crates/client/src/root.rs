//! Uma raiz local contém um diretório por compartilhamento autorizado.
use crate::{Remote, sync, tree::Tree};
use crewsync_protocol::{Operation, disk, invalid, validate_path};
use std::{collections::BTreeSet, io, path::Path};

pub fn sync(session: &impl Remote, root: &Path) -> io::Result<sync::Report> {
    let tree = Tree::open(root)?;
    if tree.meta_path().join("state.json").try_exists()? {
        return Err(invalid(
            "raiz já usada como compartilhamento; escolha uma nova raiz ou migre a pasta antes",
        ));
    }
    let binding = tree.meta_path().join("root.json");
    match disk::read_json::<String>(&binding, 64 * 1024) {
        Ok(url) if url != session.identity() => {
            return Err(invalid("raiz vinculada a outro servidor"));
        }
        Ok(_) => {}
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            disk::atomic_json(&binding, &session.identity())?;
        }
        Err(e) => return Err(e),
    }
    let shares: Vec<String> = session.call(Operation::Shares)?;
    let mut unique = BTreeSet::new();
    // Valida a lista inteira antes de criar qualquer subpasta.
    for share in &shares {
        validate_path(share)?;
        if share.contains('/') || !unique.insert(share) {
            return Err(invalid("lista de compartilhamentos inválida"));
        }
    }
    let mut report = sync::Report::default();
    let mut first_error = None;
    for share in shares {
        match tree
            .child(&share)
            .and_then(|child| sync::sync_tree(session, &share, child))
        {
            Ok(result) => {
                report.uploaded += result.uploaded;
                report.downloaded += result.downloaded;
                report.conflicts += result.conflicts;
            }
            Err(error) => {
                eprintln!("Falha ao sincronizar {share}: {error}");
                first_error.get_or_insert(error);
            }
        }
    }
    match first_error {
        Some(error) => Err(error),
        None => Ok(report),
    }
}
