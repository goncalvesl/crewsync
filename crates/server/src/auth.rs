//! Convites de uso único e tokens por dispositivo, persistidos somente por hash.
use crewsync_protocol::{disk, invalid};
use rustix::fs::{FlockOperation, flock};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Invite {
    hash: String,
    user: String,
    expires: u64,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Device {
    pub id: String,
    pub user: String,
    pub name: String,
    pub revoked: bool,
    token_hash: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct State {
    format: u32,
    invites: Vec<Invite>,
    devices: Vec<Device>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Activation {
    pub token: String,
    pub device_id: String,
}
pub struct AuthStore {
    dir: PathBuf,
    #[cfg(test)]
    fail_at: std::cell::Cell<Option<&'static str>>,
}
fn denied() -> io::Error {
    io::Error::new(
        io::ErrorKind::PermissionDenied,
        "credencial inválida ou expirada",
    )
}
fn now() -> io::Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_secs())
}
fn label(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && !value.chars().any(char::is_control)
}

impl AuthStore {
    pub fn open(root: &Path) -> io::Result<Self> {
        // Inicialização usa o mesmo requisito de raiz privada do armazenamento.
        let root = std::path::absolute(root)?;
        let mut parent = PathBuf::new();
        for component in root.components() {
            if matches!(component, std::path::Component::ParentDir) {
                return Err(invalid("raiz inválida"));
            }
            parent.push(component);
            if let Ok(metadata) = fs::symlink_metadata(&parent)
                && metadata.file_type().is_symlink()
            {
                return Err(invalid("link na raiz"));
            }
        }
        disk::private_dir(&root)?;
        let dir = root.join("auth");
        disk::private_dir(&dir)?;
        Ok(Self {
            dir,
            #[cfg(test)]
            fail_at: std::cell::Cell::new(None),
        })
    }

    fn transaction<T>(
        &self,
        change: impl FnOnce(&mut State) -> io::Result<(T, bool)>,
    ) -> io::Result<T> {
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
            .open(self.dir.join("lock"))?;
        if !lock.metadata()?.is_file() {
            return Err(invalid("lock não regular"));
        }
        flock(&lock, FlockOperation::LockExclusive)?;
        let result = (|| {
            let path = self.dir.join("state.json");
            let mut state: State = match disk::read_json(&path, 8 * 1024 * 1024) {
                Ok(state) => state,
                Err(e) if e.kind() == io::ErrorKind::NotFound => State {
                    format: 1,
                    invites: vec![],
                    devices: vec![],
                },
                Err(e) => return Err(e),
            };
            if state.format != 1 || state.invites.len() > 10_000 || state.devices.len() > 10_000 {
                return Err(invalid("estado de autenticação inválido"));
            }
            let mut seen = std::collections::BTreeSet::new();
            for invitation in &state.invites {
                disk::validate_hash(&invitation.hash)?;
                if !label(&invitation.user) || !seen.insert(&invitation.hash) {
                    return Err(invalid("convite inválido"));
                }
            }
            let mut ids = std::collections::BTreeSet::new();
            for device in &state.devices {
                disk::validate_hash(&device.token_hash)?;
                if !label(&device.user)
                    || !label(&device.name)
                    || device.id.len() != 32
                    || !device.id.bytes().all(|c| c.is_ascii_hexdigit())
                    || !seen.insert(&device.token_hash)
                    || !ids.insert(&device.id)
                {
                    return Err(invalid("dispositivo inválido"));
                }
            }
            let (value, dirty) = change(&mut state)?;
            if dirty {
                #[cfg(test)]
                if self.fail_at.get() == Some("before_publish") {
                    self.fail_at.set(None);
                    return Err(io::Error::new(
                        io::ErrorKind::StorageFull,
                        "falha simulada antes de publicar",
                    ));
                }
                disk::atomic_json(&path, &state)?;
                #[cfg(test)]
                if self.fail_at.get() == Some("after_publish") {
                    self.fail_at.set(None);
                    return Err(io::Error::new(
                        io::ErrorKind::StorageFull,
                        "falha simulada após publicar",
                    ));
                }
            }
            Ok(value)
        })();
        let _ = flock(&lock, FlockOperation::Unlock);
        result
    }

    pub fn invite(&self, user: &str, ttl: u64) -> io::Result<String> {
        self.invite_at(user, ttl, now()?)
    }
    fn invite_at(&self, user: &str, ttl: u64, time: u64) -> io::Result<String> {
        if !label(user) || !(1..=86400).contains(&ttl) {
            return Err(invalid("usuário ou validade inválidos (1..86400 segundos)"));
        }
        let code = disk::random_hex(32)?;
        let hash = disk::hash_secret(&code)?;
        self.transaction(|state| {
            state.invites.retain(|entry| entry.expires > time);
            if state.invites.len() >= 10_000 {
                return Err(invalid("limite de convites atingido"));
            }
            state.invites.push(Invite {
                hash,
                user: user.to_owned(),
                expires: time
                    .checked_add(ttl)
                    .ok_or_else(|| invalid("validade inválida"))?,
            });
            Ok((code, true))
        })
    }

    pub fn activate(
        &self,
        code: &str,
        name: &str,
        enabled: impl Fn(&str) -> bool,
    ) -> io::Result<Activation> {
        self.activate_at(code, name, enabled, now()?)
    }
    fn activate_at(
        &self,
        code: &str,
        name: &str,
        enabled: impl Fn(&str) -> bool,
        time: u64,
    ) -> io::Result<Activation> {
        disk::validate_hash(code).map_err(|_| denied())?;
        if !label(name) {
            return Err(invalid("nome de dispositivo inválido"));
        }
        let hash = disk::hash_secret(code)?;
        self.transaction(|state| {
            let index = state
                .invites
                .iter()
                .position(|entry| {
                    entry.hash == hash && entry.expires > time && enabled(&entry.user)
                })
                .ok_or_else(denied)?;
            if state.devices.len() >= 10_000 {
                return Err(invalid("limite de dispositivos atingido"));
            }
            let token = disk::random_hex(32)?;
            let device_id = disk::random_hex(16)?;
            state.devices.push(Device {
                id: device_id.clone(),
                user: state.invites[index].user.clone(),
                name: name.to_owned(),
                revoked: false,
                token_hash: disk::hash_secret(&token)?,
            });
            state.invites.remove(index);
            Ok((Activation { token, device_id }, true))
        })
    }

    pub fn authenticate(&self, token: &str) -> io::Result<String> {
        disk::validate_hash(token).map_err(|_| denied())?;
        let hash = disk::hash_secret(token)?;
        self.transaction(|state| {
            let user = state
                .devices
                .iter()
                .find(|entry| !entry.revoked && entry.token_hash == hash)
                .map(|entry| entry.user.clone())
                .ok_or_else(denied)?;
            Ok((user, false))
        })
    }
    pub fn devices(&self) -> io::Result<Vec<Device>> {
        self.transaction(|s| Ok((s.devices.clone(), false)))
    }
    pub fn revoke(&self, id: &str) -> io::Result<()> {
        self.transaction(|s| {
            let entry = s
                .devices
                .iter_mut()
                .find(|entry| entry.id == id)
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::NotFound, "dispositivo não encontrado")
                })?;
            entry.revoked = true;
            Ok(((), true))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn activation_expiration_single_use_revocation_and_restart() {
        let dir = tempfile::tempdir().unwrap();
        let store = AuthStore::open(&dir.path().join("store")).unwrap();
        let expired = store.invite_at("ana", 10, 100).unwrap();
        assert!(store.activate_at(&expired, "pc", |_| true, 110).is_err());
        let code = store.invite_at("ana", 10, 200).unwrap();
        assert!(store.activate_at(&code, "pc", |_| false, 201).is_err());
        let activation = store.activate_at(&code, "pc", |_| true, 201).unwrap();
        assert!(store.activate_at(&code, "pc", |_| true, 201).is_err());
        assert_eq!(store.authenticate(&activation.token).unwrap(), "ana");
        let raw = fs::read_to_string(store.dir.join("state.json")).unwrap();
        assert!(!raw.contains(&code));
        assert!(!raw.contains(&activation.token));
        drop(store);
        let store = AuthStore::open(&dir.path().join("store")).unwrap();
        assert_eq!(store.devices().unwrap().len(), 1);
        store.revoke(&activation.device_id).unwrap();
        assert!(store.authenticate(&activation.token).is_err());
        assert!(store.authenticate(&"0".repeat(64)).is_err());
        fs::write(store.dir.join("state.json"), "corrupt").unwrap();
        assert!(store.devices().is_err());
    }
    #[test]
    fn activation_failure_is_atomic_and_never_reuses_consumed_invite() {
        let dir = tempfile::tempdir().unwrap();
        let store = AuthStore::open(&dir.path().join("store")).unwrap();
        let code = store.invite_at("ana", 60, 100).unwrap();
        store.fail_at.set(Some("before_publish"));
        assert!(store.activate_at(&code, "pc", |_| true, 101).is_err());
        assert!(store.devices().unwrap().is_empty());
        store.fail_at.set(Some("after_publish"));
        assert!(store.activate_at(&code, "pc", |_| true, 101).is_err());
        assert_eq!(store.devices().unwrap().len(), 1);
        assert!(store.activate_at(&code, "pc", |_| true, 101).is_err());
        let bytes = fs::read(store.dir.join("state.json")).unwrap();
        assert!(!String::from_utf8(bytes).unwrap().contains(&code));
    }
}
