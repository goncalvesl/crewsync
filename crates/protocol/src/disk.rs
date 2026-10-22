//! Primitivas Linux para estado interno privado e conteúdo em streaming.
use crate::invalid;
use serde::{Serialize, de::DeserializeOwned};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::Path;
use std::process::{Command, Stdio};

pub fn private_dir(path: &Path) -> io::Result<()> {
    match fs::DirBuilder::new().mode(0o700).create(path) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
        Err(e) => return Err(e),
    }
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_dir() || metadata.permissions().mode() & 0o077 != 0 {
        return Err(invalid(
            "diretório interno deve ser privado, regular e 0700",
        ));
    }
    if let Some(parent) = path.parent() {
        File::open(parent)?.sync_all()?;
    }
    Ok(())
}

pub fn regular_file(path: &Path) -> io::Result<File> {
    if !fs::symlink_metadata(path)?.is_file() {
        return Err(invalid("arquivo interno não regular"));
    }
    OpenOptions::new()
        .read(true)
        .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
        .open(path)
}

pub fn read_json<T: DeserializeOwned>(path: &Path, limit: u64) -> io::Result<T> {
    let mut data = Vec::new();
    regular_file(path)?.take(limit + 1).read_to_end(&mut data)?;
    if data.len() as u64 > limit {
        return Err(invalid("estado excede limite"));
    }
    serde_json::from_slice(&data).map_err(|_| invalid("estado JSON inválido"))
}

pub fn atomic_json(path: &Path, value: &impl Serialize) -> io::Result<()> {
    let parent = path.parent().ok_or_else(|| invalid("caminho sem pai"))?;
    let temp = parent.join(format!(".tmp-{}", random_hex(16)?));
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temp)?;
        serde_json::to_writer(&mut file, value).map_err(io::Error::other)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        fs::rename(&temp, path)?;
        File::open(parent)?.sync_all()
    })();
    let _ = fs::remove_file(temp);
    result
}

pub fn random_hex(bytes: usize) -> io::Result<String> {
    let mut data = vec![0; bytes];
    File::open("/dev/urandom")?.read_exact(&mut data)?;
    Ok(data.iter().map(|b| format!("{b:02x}")).collect())
}

pub fn hash_file(file: &File) -> io::Result<String> {
    let mut input = file.try_clone()?;
    input.rewind()?;
    let output = Command::new("/usr/bin/sha256sum")
        .stdin(Stdio::from(input))
        .output()?;
    if !output.status.success() {
        return Err(invalid("falha SHA-256"));
    }
    let hash = std::str::from_utf8(
        output
            .stdout
            .get(..64)
            .ok_or_else(|| invalid("saída SHA-256 inválida"))?,
    )
    .map_err(|_| invalid("saída SHA-256 inválida"))?
    .to_owned();
    validate_hash(&hash)?;
    Ok(hash)
}

pub fn hash_secret(value: &str) -> io::Result<String> {
    let mut child = Command::new("/usr/bin/sha256sum")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let input_result = match child.stdin.take() {
        Some(mut stdin) => stdin.write_all(value.as_bytes()),
        None => Err(invalid("entrada SHA-256 indisponível")),
    };
    let output = child.wait_with_output()?;
    input_result?;
    if !output.status.success() {
        return Err(invalid("falha SHA-256"));
    }
    let hash = String::from_utf8(output.stdout).map_err(|_| invalid("saída SHA-256 inválida"))?;
    let hash = hash
        .get(..64)
        .ok_or_else(|| invalid("saída SHA-256 inválida"))?
        .to_owned();
    validate_hash(&hash)?;
    Ok(hash)
}

pub fn validate_hash(hash: &str) -> io::Result<()> {
    if hash.len() != 64
        || !hash
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    {
        return Err(invalid("SHA-256 inválido"));
    }
    Ok(())
}
