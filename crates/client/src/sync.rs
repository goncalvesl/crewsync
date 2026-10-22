//! Comparação entre baseline, disco e manifesto remoto; cópias locais preservadas.
use crate::{Remote, tree::Tree};
use crewsync_protocol::{Content, Entry, Manifest, Operation, disk, invalid};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, Seek};
use std::path::Path;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct State {
    format: u32,
    url: String,
    share: String,
    entries: BTreeMap<String, Entry>,
}
#[derive(Default)]
pub struct Report {
    pub uploaded: usize,
    pub downloaded: usize,
    pub conflicts: usize,
}
fn live(content: &Content) -> Option<&Content> {
    if *content == Content::Deleted {
        None
    } else {
        Some(content)
    }
}

pub fn sync(session: &impl Remote, share: &str, root: &Path) -> io::Result<Report> {
    crewsync_protocol::validate_path(share)?;
    if share.contains('/') {
        return Err(invalid("compartilhamento deve ser de primeiro nível"));
    }
    let tree = Tree::open(root)?;
    sync_tree(session, share, tree)
}

pub(crate) fn sync_tree(session: &impl Remote, share: &str, tree: Tree) -> io::Result<Report> {
    if tree.meta_path().join("root.json").try_exists()? {
        return Err(invalid(
            "pasta usada como raiz de compartilhamentos; use sync-root/watch-root",
        ));
    }
    let state_path = tree.meta_path().join("state.json");
    let mut state: State = match disk::read_json(&state_path, crewsync_protocol::MAX_JSON) {
        Ok(state) => state,
        Err(e) if e.kind() == io::ErrorKind::NotFound => State {
            format: 1,
            url: session.identity().to_owned(),
            share: share.to_owned(),
            entries: BTreeMap::new(),
        },
        Err(e) => return Err(e),
    };
    if state.format != 1 || state.url != session.identity() || state.share != share {
        return Err(invalid(
            "pasta vinculada a outro servidor/compartilhamento ou estado inválido",
        ));
    }
    let remote: Manifest = session.call(Operation::Manifest {
        share: share.to_owned(),
    })?;
    validate_manifest(&remote)?;
    if state
        .entries
        .values()
        .map(|entry| entry.revision)
        .max()
        .unwrap_or(0)
        > remote.revision
        || state
            .entries
            .keys()
            .any(|path| !remote.entries.contains_key(path))
    {
        return Err(invalid(
            "servidor recuou ou perdeu revisões; reconciliação recusada",
        ));
    }
    if state.entries.len() > 100_000 {
        return Err(invalid("baseline excede limite"));
    }
    for (path, entry) in &state.entries {
        crewsync_protocol::validate_path(path)?;
        if entry.revision == 0 {
            return Err(invalid("baseline inválido"));
        }
        if let Content::File { hash, size } = &entry.content {
            disk::validate_hash(hash)?;
            if *size > (1u64 << 40) {
                return Err(invalid("baseline inválido"));
            }
        }
    }
    let local = tree.scan()?; // Nenhuma alteração se árvore contém links/especiais.
    // Excluir/substituir diretório local não pode apagar filhos alterados remotamente.
    let structural_conflicts: Vec<String> = state
        .entries
        .iter()
        .filter_map(|(path, baseline)| {
            if baseline.content != Content::Directory
                || local.get(path) == Some(&Content::Directory)
                || remote.entries.get(path).map(|entry| &entry.content) != Some(&Content::Directory)
            {
                return None;
            }
            let prefix = format!("{path}/");
            let changed = remote
                .entries
                .range(prefix.clone()..)
                .take_while(|(child, _)| child.starts_with(&prefix))
                .any(|(child, entry)| {
                    state.entries.get(child).map(|base| base.revision) != Some(entry.revision)
                });
            changed.then(|| path.clone())
        })
        .collect();
    let protected = |path: &str| {
        structural_conflicts
            .iter()
            .any(|parent| path == parent || path.starts_with(&format!("{parent}/")))
    };
    let paths: BTreeSet<_> = local
        .keys()
        .chain(remote.entries.keys())
        .chain(state.entries.keys())
        .cloned()
        .collect();
    let mut paths: Vec<_> = paths.into_iter().collect();
    // Pais vivos primeiro, exclusões de baixo para cima.
    paths.sort_by_key(|path| {
        let deleted = !protected(path)
            && (remote
                .entries
                .get(path)
                .is_some_and(|e| e.content == Content::Deleted)
                && !local.contains_key(path)
                || !local.contains_key(path)
                    && state.entries.contains_key(path)
                    && remote.entries.get(path) == state.entries.get(path));
        let depth = path.split('/').count();
        (
            !deleted,
            if deleted { usize::MAX - depth } else { depth },
            path.clone(),
        )
    });
    if !state_path.try_exists()? {
        save_state(&state_path, &state)?;
    }
    let mut report = Report::default();
    for path in paths {
        let baseline = state.entries.get(&path).cloned();
        let latest = remote.entries.get(&path).cloned();
        let baseline_content = baseline.as_ref().and_then(|e| live(&e.content));
        let remote_content = latest.as_ref().and_then(|e| live(&e.content));
        let disk_content = tree.current(&path)?;
        let local_content = disk_content.as_ref();
        if local_content == remote_content {
            if latest != baseline {
                if let Some(entry) = latest {
                    state.entries.insert(path, entry);
                }
                save_state(&state_path, &state)?;
            }
            continue;
        }
        let local_changed = local_content != baseline_content;
        let remote_changed = protected(&path)
            || latest.as_ref().map_or(0, |e| e.revision)
                != baseline.as_ref().map_or(0, |e| e.revision);
        if local_changed && !remote_changed {
            let base = baseline.as_ref().map_or(0, |e| e.revision);
            let revision: u64 = match local_content {
                Some(Content::Directory) => session.call(Operation::Mkdir {
                    share: share.to_owned(),
                    path: path.clone(),
                    base,
                })?,
                Some(Content::File { hash, size }) => {
                    let (temp_path, mut temp) = tree.temp()?;
                    let result = (|| {
                        let source = tree.file(&path)?;
                        let copied = io::copy(&mut source.take(size + 1), &mut temp)?;
                        if copied != *size || disk::hash_file(&temp)? != *hash {
                            return Err(io::Error::new(
                                io::ErrorKind::Interrupted,
                                "arquivo mudou durante snapshot",
                            ));
                        }
                        tree.check(&path, local_content)?;
                        temp.rewind()?;
                        let mut response = Vec::new();
                        session.transfer(
                            Operation::Put {
                                share: share.to_owned(),
                                path: path.clone(),
                                base,
                                hash: hash.clone(),
                                size: *size,
                            },
                            &mut temp,
                            *size,
                            &mut response,
                            1024,
                        )?;
                        serde_json::from_slice(&response).map_err(|_| invalid("revisão inválida"))
                    })();
                    let _ = fs::remove_file(temp_path);
                    result?
                }
                _ => session.call(Operation::Delete {
                    share: share.to_owned(),
                    path: path.clone(),
                    base,
                })?,
            };
            state.entries.insert(
                path.clone(),
                Entry {
                    revision,
                    content: local_content.cloned().unwrap_or(Content::Deleted),
                },
            );
            report.uploaded += 1;
        } else {
            // Remoto mudou; conflito local é preservado antes de instalar a versão remota.
            let prefix = format!("{path}/");
            let changed_children = local.iter().any(|(child, content)| {
                child.starts_with(&prefix)
                    && state
                        .entries
                        .get(child)
                        .and_then(|entry| live(&entry.content))
                        != Some(content)
            }) || state.entries.iter().any(|(child, entry)| {
                child.starts_with(&prefix) && live(&entry.content) != local.get(child)
            });
            let conflict =
                local_changed || (local_content == Some(&Content::Directory) && changed_children);
            let temp = if let Some(Content::File { hash, size }) = remote_content {
                let (temp_path, mut file) = tree.temp()?;
                let result = (|| {
                    session.transfer(
                        Operation::Get {
                            share: share.to_owned(),
                            path: path.clone(),
                        },
                        &mut io::empty(),
                        0,
                        &mut file,
                        *size,
                    )?;
                    file.sync_all()?;
                    if file.metadata()?.len() != *size || disk::hash_file(&file)? != *hash {
                        return Err(invalid(
                            "download mudou ou hash incorreto; execute novamente",
                        ));
                    }
                    Ok(())
                })();
                if let Err(error) = result {
                    let _ = fs::remove_file(&temp_path);
                    return Err(error);
                }
                Some(temp_path)
            } else {
                None
            };
            tree.replace(
                &path,
                remote_content,
                temp.as_deref(),
                local_content,
                conflict,
            )?;
            if let Some(entry) = latest {
                state.entries.insert(path.clone(), entry);
            }
            if conflict {
                report.conflicts += 1;
            }
            report.downloaded += 1;
        }
        save_state(&state_path, &state)?;
    }
    Ok(report)
}

fn validate_manifest(manifest: &Manifest) -> io::Result<()> {
    if manifest.format != 1 || manifest.entries.len() > 100_000 {
        return Err(invalid("manifesto remoto inválido"));
    }
    for (path, entry) in &manifest.entries {
        crewsync_protocol::validate_path(path)?;
        if entry.revision == 0 || entry.revision > manifest.revision {
            return Err(invalid("revisão remota inválida"));
        }
        if let Content::File { hash, size } = &entry.content {
            if *size > (1u64 << 40) {
                return Err(invalid("objeto remoto excede limite"));
            }
            disk::validate_hash(hash)?;
        }
    }
    Ok(())
}
use std::io::Read;

fn save_state(path: &Path, state: &State) -> io::Result<()> {
    let size = serde_json::to_vec(state).map_err(io::Error::other)?.len() as u64;
    if size > crewsync_protocol::MAX_JSON {
        return Err(invalid("baseline excede limite de 16 MiB"));
    }
    disk::atomic_json(path, state)
}
