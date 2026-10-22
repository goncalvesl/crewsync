//! Fachada de ACLs; o worker fornece a identidade autenticada pelo token.
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use crate::config::{Config, Operation};
use crate::storage::{Content, Manifest, Store, validate_path};

/// Não aceitar `user` vindo diretamente da rede: usar a identidade do token autenticado.
/// A raiz e a configuração devem ser administradas somente pelo operador.
pub struct AuthorizedStore {
    store: Store,
    config: Config,
    config_path: PathBuf,
}

impl AuthorizedStore {
    pub fn open(config_path: impl AsRef<Path>) -> io::Result<Self> {
        let config_path = std::path::absolute(config_path)?;
        let config = Config::load(&config_path)?;
        let store = Store::open(config.storage_root())?;
        Ok(Self {
            store,
            config,
            config_path,
        })
    }

    /// Substitui somente após validação integral. Mudança de raiz exige reinício.
    pub fn reload_config(&mut self) -> io::Result<()> {
        let next = Config::load(&self.config_path)?;
        if next.storage_root() != self.config.storage_root() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "mudança de storage_root exige reinício",
            ));
        }
        self.config = next;
        Ok(())
    }

    pub fn user_enabled(&self, user: &str) -> bool {
        self.config.user_enabled(user)
    }

    fn authorize(&self, user: &str, share: &str, operation: Operation) -> io::Result<()> {
        if !self.config.allows(user, share, operation) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "acesso negado",
            ));
        }
        Ok(())
    }

    /// Usuário desconhecido/desativado recebe lista vazia, sem revelar nomes.
    pub fn shares(&self, user: &str) -> Vec<String> {
        self.config
            .readable_shares(user)
            .map(str::to_owned)
            .collect()
    }

    pub fn manifest(&self, user: &str, share: &str) -> io::Result<Manifest> {
        self.authorize(user, share, Operation::Read)?;
        self.store.manifest(share)
    }

    /// Resolve hash pelo caminho autorizado, nunca por hash arbitrário do cliente.
    pub fn file(&self, user: &str, share: &str, path: &str) -> io::Result<File> {
        self.authorize(user, share, Operation::Read)?;
        validate_path(path)?;
        let manifest = self.store.manifest(share)?;
        match manifest.entries.get(path).map(|entry| &entry.content) {
            Some(Content::File { hash, size }) => self.store.object(hash, *size),
            _ => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "arquivo não encontrado",
            )),
        }
    }

    /// Autoriza e verifica revisão antes de consumir bytes. Publica depois do hash.
    pub fn put_file(
        &mut self,
        user: &str,
        share: &str,
        path: &str,
        base: u64,
        expected_hash: &str,
        reader: impl Read,
    ) -> io::Result<u64> {
        self.authorize(user, share, Operation::Write)?;
        validate_path(path)?;
        let manifest = self.store.manifest(share)?;
        if manifest.entries.get(path).map_or(0, |entry| entry.revision) != base {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "conflito de revisão; preserve a versão concorrente",
            ));
        }
        let size = self
            .store
            .put_object(reader, expected_hash, self.config.max_upload_bytes())?;
        self.store.apply(
            share,
            path,
            base,
            Content::File {
                hash: expected_hash.to_owned(),
                size,
            },
        )
    }

    pub fn create_directory(
        &mut self,
        user: &str,
        share: &str,
        path: &str,
        base: u64,
    ) -> io::Result<u64> {
        self.authorize(user, share, Operation::Write)?;
        self.store.apply(share, path, base, Content::Directory)
    }

    pub fn delete(&mut self, user: &str, share: &str, path: &str, base: u64) -> io::Result<u64> {
        self.authorize(user, share, Operation::Write)?;
        self.store.apply(share, path, base, Content::Deleted)
    }
}
