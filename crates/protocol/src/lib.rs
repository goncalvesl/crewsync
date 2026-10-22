//! Contratos da API HTTPS de piloto, versão 1.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io;
pub mod disk;
pub mod transport;
pub const MAX_JSON: u64 = 16 * 1024 * 1024;
pub const MAX_FRAME: u64 = 64 * 1024;
pub fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
/// Caminho relativo conservador, sem componentes especiais ou escapes.
pub fn validate_path(path: &str) -> io::Result<()> {
    if path.is_empty() || path.len() > 4096 || path.split('/').next() == Some(".crewsync") {
        return Err(invalid("caminho vazio ou longo demais"));
    }
    for part in path.split('/') {
        let stem = part
            .split('.')
            .next()
            .unwrap_or_default()
            .trim_end_matches(' ')
            .to_ascii_uppercase();
        if part.is_empty()
            || part.len() > 255
            || part == "."
            || part == ".."
            || part.ends_with([' ', '.'])
            || part
                .chars()
                .any(|c| c.is_control() || "\\:*?\"<>|".contains(c))
            || matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            || (stem.len() == 4
                && (stem.starts_with("COM") || stem.starts_with("LPT"))
                && matches!(stem.as_bytes()[3], b'1'..=b'9'))
        {
            return Err(invalid("componente de caminho inválido"));
        }
    }
    Ok(())
}

/// Conteúdo atual ou registro persistente de exclusão.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Content {
    File { hash: String, size: u64 },
    Directory,
    Deleted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub revision: u64,
    pub content: Content,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub format: u32,
    pub revision: u64,
    pub entries: BTreeMap<String, Entry>,
}

impl Default for Manifest {
    fn default() -> Self {
        Self {
            format: 1,
            revision: 0,
            entries: BTreeMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub token: Option<String>,
    pub operation: Operation,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    Activate {
        code: String,
        name: String,
    },
    Shares,
    Manifest {
        share: String,
    },
    Get {
        share: String,
        path: String,
    },
    Put {
        share: String,
        path: String,
        base: u64,
        hash: String,
        size: u64,
    },
    Mkdir {
        share: String,
        path: String,
        base: u64,
    },
    Delete {
        share: String,
        path: String,
        base: u64,
    },
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    pub request: Request,
    pub body_bytes: u64,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Response {
    pub status: u16,
    pub length: u64,
}
