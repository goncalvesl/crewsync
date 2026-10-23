//! Painel administrativo: HTML no servidor, chave própria, sessão e CSRF.
mod config;
mod views;

use crate::{auth::AuthStore, config::Config};
use crewsync_protocol::{disk, invalid, transport};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use subtle::ConstantTimeEq;

const MAX_BODY: u64 = 16 * 1024;
const SESSION_SECONDS: u64 = 8 * 3600;
const GUEST_SECONDS: u64 = 1200;
const MAX_SESSIONS: usize = 64;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Credentials {
    format: u32,
    key_hash: String,
    origin: String,
}
fn time() -> io::Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_secs())
}
fn origin(value: &str) -> io::Result<String> {
    let trust = if value.starts_with("http://") {
        transport::Trust::PlainHttp
    } else {
        transport::Trust::System
    };
    Ok(transport::validate_url(value, trust)?
        .origin()
        .ascii_serialization())
}

/// Gera uma chave aleatória de 256 bits; somente o hash é persistido.
pub fn reset_key(config: &Path, public_url: &str) -> io::Result<String> {
    let origin = origin(public_url)?;
    let config = Config::load(config)?;
    AuthStore::open(config.storage_root())?;
    let path = config.storage_root().join("auth/admin.json");
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
        .open(config.storage_root().join("auth/admin.lock"))?;
    if !lock.metadata()?.is_file() {
        return Err(invalid("lock administrativo inválido"));
    }
    rustix::fs::flock(&lock, rustix::fs::FlockOperation::LockExclusive)?;
    if let Ok(metadata) = fs::symlink_metadata(&path)
        && (!metadata.is_file() || metadata.permissions().mode() & 0o077 != 0)
    {
        return Err(invalid(
            "credencial administrativa deve ser arquivo privado",
        ));
    }
    let key = disk::random_hex(32)?;
    disk::atomic_json(
        &path,
        &Credentials {
            format: 1,
            key_hash: disk::hash_secret(&key)?,
            origin,
        },
    )?;
    Ok(key)
}

/// Inicializa a chave administrativa somente na primeira execução.
/// A chave precisa ser um segredo aleatório de 256 bits em hexadecimal.
pub fn bootstrap_key(config: &Path, public_url: &str, key: &str) -> io::Result<bool> {
    let origin = origin(public_url)?;
    let config = Config::load(config)?;
    AuthStore::open(config.storage_root())?;
    let path = config.storage_root().join("auth/admin.json");
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
        .open(config.storage_root().join("auth/admin.lock"))?;
    if !lock.metadata()?.is_file() {
        return Err(invalid("lock administrativo inválido"));
    }
    rustix::fs::flock(&lock, rustix::fs::FlockOperation::LockExclusive)?;
    match fs::symlink_metadata(&path) {
        Ok(metadata) => {
            if !metadata.is_file() || metadata.permissions().mode() & 0o077 != 0 {
                return Err(invalid(
                    "credencial administrativa deve ser arquivo privado",
                ));
            }
            return Ok(false);
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    disk::validate_hash(key)?;
    disk::atomic_json(
        &path,
        &Credentials {
            format: 1,
            key_hash: disk::hash_secret(key)?,
            origin,
        },
    )?;
    Ok(true)
}

pub(crate) struct Request {
    pub method: String,
    pub target: String,
    headers: BTreeMap<String, String>,
    length: Option<u64>,
}
impl Request {
    pub fn parse(head: &[u8]) -> io::Result<Self> {
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut request = httparse::Request::new(&mut headers);
        if !request
            .parse(head)
            .map_err(|_| invalid("HTTP inválido"))?
            .is_complete()
            || request.version != Some(1)
        {
            return Err(invalid("HTTP inválido"));
        }
        let method = request
            .method
            .ok_or_else(|| invalid("método ausente"))?
            .to_owned();
        let target = request
            .path
            .ok_or_else(|| invalid("caminho ausente"))?
            .to_owned();
        if !target.starts_with('/') || target.len() > 4096 {
            return Err(invalid("caminho inválido"));
        }
        let mut values = BTreeMap::new();
        for header in request.headers.iter() {
            let name = header.name.to_ascii_lowercase();
            let value = std::str::from_utf8(header.value)
                .map_err(|_| invalid("header inválido"))?
                .to_owned();
            if values.insert(name, value).is_some() {
                return Err(invalid("header duplicado"));
            }
        }
        if !values.contains_key("host")
            || values.contains_key("transfer-encoding")
            || values.contains_key("expect")
        {
            return Err(invalid("headers HTTP inválidos"));
        }
        let length = if values.contains_key("content-length") {
            Some(transport::content_length(request.headers)?)
        } else {
            None
        };
        Ok(Self {
            method,
            target,
            headers: values,
            length,
        })
    }
    fn header(&self, name: &str) -> &str {
        self.headers.get(name).map(String::as_str).unwrap_or("")
    }
    fn path(&self) -> &str {
        self.target.split('?').next().unwrap_or("")
    }
}

pub(crate) struct Response {
    status: u16,
    content_type: &'static str,
    body: String,
    cookie: Option<String>,
    location: Option<&'static str>,
}
impl Response {
    fn html(status: u16, body: String) -> Self {
        Self {
            status,
            content_type: "text/html; charset=utf-8",
            body,
            cookie: None,
            location: None,
        }
    }
    fn redirect(location: &'static str) -> Self {
        Self {
            status: 303,
            location: Some(location),
            ..Self::html(303, String::new())
        }
    }
    pub fn write(&self, output: &mut impl Write) -> io::Result<()> {
        write!(
            output,
            "HTTP/1.1 {} Result\r\nContent-Length: {}\r\nContent-Type: {}\r\nConnection: close\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nX-Frame-Options: DENY\r\nReferrer-Policy: same-origin\r\nContent-Security-Policy: default-src 'none'; style-src 'self'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'\r\n",
            self.status,
            self.body.len(),
            self.content_type
        )?;
        if let Some(cookie) = &self.cookie {
            write!(output, "Set-Cookie: {cookie}\r\n")?;
        }
        if let Some(location) = self.location {
            write!(output, "Location: {location}\r\n")?;
        }
        output.write_all(b"\r\n")?;
        output.write_all(self.body.as_bytes())?;
        output.flush()
    }
}
pub(crate) fn error(status: u16, message: &str) -> Response {
    Response::html(
        status,
        views::message("Não foi possível continuar", message),
    )
}

struct Session {
    csrf: String,
    authenticated: bool,
    expires: u64,
}
pub(crate) struct Admin {
    path: PathBuf,
    root: PathBuf,
    sessions: BTreeMap<String, Session>,
    key_hash: Option<String>,
}
type Form = BTreeMap<String, String>;
fn field<'a>(form: &'a Form, name: &str) -> io::Result<&'a str> {
    form.get(name)
        .map(String::as_str)
        .ok_or_else(|| invalid("campo ausente"))
}
fn form(bytes: &[u8]) -> io::Result<Form> {
    let mut fields = BTreeMap::new();
    for (name, value) in url::form_urlencoded::parse(bytes) {
        if name.len() > 64
            || value.len() > 1024
            || fields.len() >= 16
            || fields
                .insert(name.into_owned(), value.into_owned())
                .is_some()
        {
            return Err(invalid("formulário inválido"));
        }
    }
    Ok(fields)
}
fn equal(first: &str, second: &str) -> bool {
    bool::from(first.as_bytes().ct_eq(second.as_bytes()))
}
fn cookie_name(origin: &str) -> &'static str {
    if origin.starts_with("https://") {
        "__Host-crewsync_admin"
    } else {
        "crewsync_admin"
    }
}
fn cookie_value(origin: &str, token: &str, age: u64) -> String {
    format!(
        "{}={token}; Path=/; HttpOnly; SameSite=Strict; Max-Age={age}{}",
        cookie_name(origin),
        if origin.starts_with("https://") {
            "; Secure"
        } else {
            ""
        }
    )
}
impl Admin {
    pub fn new(path: &Path) -> io::Result<Self> {
        let path = std::path::absolute(path)?;
        let root = Config::load(&path)?.storage_root().to_owned();
        Ok(Self {
            path,
            root,
            sessions: BTreeMap::new(),
            key_hash: None,
        })
    }
    fn credentials(&mut self) -> io::Result<Option<Credentials>> {
        let path = self.root.join("auth/admin.json");
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                self.sessions.clear();
                return Ok(None);
            }
            Err(error) => return Err(error),
        };
        if !metadata.is_file() || metadata.permissions().mode() & 0o077 != 0 {
            return Err(invalid("chave administrativa não privada"));
        }
        let credentials: Credentials = disk::read_json(&path, 4096)?;
        disk::validate_hash(&credentials.key_hash)?;
        if credentials.format != 1 || origin(&credentials.origin)? != credentials.origin {
            return Err(invalid("credencial administrativa inválida"));
        }
        if self.key_hash.as_ref() != Some(&credentials.key_hash) {
            self.sessions.clear();
            self.key_hash = Some(credentials.key_hash.clone());
        }
        let now = time()?;
        self.sessions.retain(|_, session| session.expires > now);
        Ok(Some(credentials))
    }
    fn session_id(&self, request: &Request, origin: &str) -> Option<String> {
        let mut found = None;
        for entry in request.header("cookie").split(';') {
            let (name, value) = entry.trim().split_once('=')?;
            if name == cookie_name(origin) {
                if found.is_some() || disk::validate_hash(value).is_err() {
                    return None;
                }
                found = Some(disk::hash_secret(value).ok()?);
            }
        }
        found.filter(|hash| self.sessions.contains_key(hash))
    }
    fn create_session(&mut self, authenticated: bool) -> io::Result<(String, String)> {
        if self.sessions.len() >= MAX_SESSIONS {
            if let Some(guest) = self
                .sessions
                .iter()
                .find(|(_, s)| !s.authenticated)
                .map(|(hash, _)| hash.clone())
            {
                self.sessions.remove(&guest);
            } else {
                return Err(io::Error::other("limite de sessões"));
            }
        }
        let token = disk::random_hex(32)?;
        let hash = disk::hash_secret(&token)?;
        self.sessions.insert(
            hash.clone(),
            Session {
                csrf: disk::random_hex(32)?,
                authenticated,
                expires: time()?
                    + if authenticated {
                        SESSION_SECONDS
                    } else {
                        GUEST_SECONDS
                    },
            },
        );
        Ok((hash, token))
    }
    pub fn respond(&mut self, request: &Request, input: &mut impl Read) -> Response {
        self.handle(request, input)
            .unwrap_or_else(|error| match error.kind() {
                io::ErrorKind::AlreadyExists | io::ErrorKind::WouldBlock => error_response(409),
                io::ErrorKind::InvalidData
                | io::ErrorKind::InvalidInput
                | io::ErrorKind::UnexpectedEof => error_response(400),
                _ => error_response(503),
            })
    }
    fn handle(&mut self, request: &Request, input: &mut impl Read) -> io::Result<Response> {
        if request.method == "GET" && request.path() == "/favicon.ico" {
            return Ok(Response::html(204, String::new()));
        }
        if request.method == "GET" && request.path() == "/admin/style.css" {
            return Ok(Response {
                content_type: "text/css; charset=utf-8",
                body: include_str!("style.css").to_owned(),
                ..Response::html(200, String::new())
            });
        }
        let Some(credentials) = self.credentials()? else {
            return Ok(Response::html(
                503,
                views::message(
                    "Painel não ativado",
                    "Ative a administração pelo comando admin-key do servidor. A sincronização continua funcionando.",
                ),
            ));
        };
        let expected =
            url::Url::parse(&credentials.origin).map_err(|_| invalid("origem inválida"))?;
        let host = request.header("host");
        if host.contains(['/', '\\', '@', '?', '#']) {
            return Ok(error(421, "Use o endereço configurado para o painel."));
        }
        let incoming = url::Url::parse(&format!("{}://{host}", expected.scheme()))
            .map_err(|_| invalid("Host inválido"))?;
        if incoming.origin().ascii_serialization() != credentials.origin {
            return Ok(error(421, "Use o endereço configurado para o painel."));
        }
        let session_id = self.session_id(request, &credentials.origin);
        if request.method == "GET" {
            if request.length.unwrap_or(0) != 0 {
                return Ok(error_response(400));
            }
            if request.path() != "/" && request.path() != "/admin" {
                return Ok(error_response(404));
            }
            if let Some(session) = session_id.as_ref().and_then(|hash| self.sessions.get(hash)) {
                if session.authenticated {
                    let snapshot = config::Snapshot::load(&self.path)?;
                    if snapshot.config.storage_root() != self.root {
                        return Ok(error_response(409));
                    }
                    let devices = AuthStore::open(&self.root)?.devices()?;
                    let filter = request
                        .target
                        .split_once('?')
                        .map(|(_, query)| form(query.as_bytes()))
                        .transpose()?
                        .and_then(|fields| fields.get("user").cloned())
                        .unwrap_or_default();
                    return Ok(Response::html(
                        200,
                        views::dashboard(&snapshot, &devices, &session.csrf, &filter),
                    ));
                }
                return Ok(Response::html(200, views::login(&session.csrf)));
            }
            let (hash, token) = self.create_session(false)?;
            let mut response = Response::html(200, views::login(&self.sessions[&hash].csrf));
            response.cookie = Some(cookie_value(&credentials.origin, &token, GUEST_SECONDS));
            return Ok(response);
        }
        if request.method != "POST" {
            return Ok(error_response(405));
        }
        let length = request
            .length
            .ok_or_else(|| invalid("Content-Length ausente"))?;
        if length > MAX_BODY {
            return Ok(error_response(413));
        }
        let mut bytes = Vec::new();
        input.take(length).read_to_end(&mut bytes)?;
        if bytes.len() as u64 != length {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "formulário incompleto",
            ));
        }
        if request.header("origin") != credentials.origin
            || request.header("sec-fetch-site") == "cross-site"
            || !request
                .header("content-type")
                .split(';')
                .next()
                .is_some_and(|value| value.trim() == "application/x-www-form-urlencoded")
        {
            return Ok(error_response(403));
        }
        let fields = form(&bytes)?;
        let Some(hash) = session_id else {
            return Ok(error_response(403));
        };
        let session = &self.sessions[&hash];
        if fields
            .get("csrf")
            .is_none_or(|value| !equal(value, &session.csrf))
        {
            return Ok(error_response(403));
        }
        if request.path() == "/admin/login" {
            let key = field(&fields, "key")?.trim();
            if disk::validate_hash(key).is_err()
                || !equal(&disk::hash_secret(key)?, &credentials.key_hash)
            {
                return Ok(error(
                    401,
                    "Chave administrativa inválida. Volte ao painel e tente novamente.",
                ));
            }
            self.sessions.remove(&hash);
            let (_, token) = self.create_session(true)?;
            let mut response = Response::redirect("/");
            response.cookie = Some(cookie_value(&credentials.origin, &token, SESSION_SECONDS));
            return Ok(response);
        }
        if !session.authenticated {
            return Ok(error_response(403));
        }
        if request.path() == "/admin/logout" {
            self.sessions.remove(&hash);
            let mut response = Response::redirect("/");
            response.cookie = Some(cookie_value(&credentials.origin, "", 0));
            return Ok(response);
        }
        let snapshot = config::Snapshot::load(&self.path)?;
        if snapshot.config.storage_root() != self.root {
            return Ok(error_response(409));
        }
        let revision = field(&fields, "revision")?;
        if revision != snapshot.revision {
            return Ok(error_response(409));
        }
        match request.path() {
            "/admin/user" => {
                let enabled = match field(&fields, "enabled")? {
                    "true" => true,
                    "false" => false,
                    _ => return Ok(error_response(400)),
                };
                config::update(
                    &self.path,
                    revision,
                    &self.root,
                    config::Change::User {
                        name: field(&fields, "user")?,
                        enabled,
                    },
                )?;
            }
            "/admin/share" => config::update(
                &self.path,
                revision,
                &self.root,
                config::Change::Share {
                    name: field(&fields, "share")?,
                },
            )?,
            "/admin/permission" => config::update(
                &self.path,
                revision,
                &self.root,
                config::Change::Permission {
                    share: field(&fields, "share")?,
                    user: field(&fields, "user")?,
                    access: field(&fields, "access")?,
                },
            )?,
            "/admin/invite" => {
                let user = field(&fields, "user")?;
                if !snapshot.config.user_enabled(user) {
                    return Ok(error_response(400));
                }
                let ttl = field(&fields, "ttl")?
                    .parse::<u64>()
                    .map_err(|_| invalid("validade inválida"))?;
                let invite = AuthStore::open(&self.root)?.invite(user, ttl)?;
                return Ok(Response::html(200, views::invite(user, &invite, ttl)));
            }
            "/admin/revoke" => AuthStore::open(&self.root)?.revoke(field(&fields, "device")?)?,
            _ => return Ok(error_response(404)),
        }
        Ok(Response::redirect("/"))
    }
}
fn error_response(status: u16) -> Response {
    let message = match status {
        400 => {
            "Confira os nomes e permissões informados. Nenhuma configuração inválida foi publicada."
        }
        403 => "Sessão ou formulário inválido. Recarregue o painel e entre novamente.",
        404 => "Página não encontrada.",
        405 => "Operação não permitida.",
        409 => "A configuração mudou ou a pasta já existe. Recarregue a página antes de salvar.",
        413 => "Formulário muito grande.",
        _ => "Painel temporariamente indisponível. Confira a configuração e os logs do servidor.",
    };
    error(status, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Operation;

    fn fixture() -> (tempfile::TempDir, PathBuf, String) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("server.toml");
        fs::write(&path, format!("# manter comentário\nversion=1\n[server]\nstorage_root='{}'\nmax_upload_bytes=12345\n[users.ana]\nenabled=true\n[shares.Obras.permissions]\nana='write'\n", dir.path().join("store").display())).unwrap();
        let key = reset_key(&path, "https://sync.example.test").unwrap();
        (dir, path, key)
    }
    #[test]
    fn bootstrap_admin_is_private_hashed_and_does_not_rotate_on_restart() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("server.toml");
        fs::write(
            &path,
            format!(
                "version=1\n[server]\nstorage_root='{}'\n",
                dir.path().join("store").display()
            ),
        )
        .unwrap();
        let key = "a".repeat(64);
        assert!(bootstrap_key(&path, "https://sync.example.test", &key).unwrap());
        assert!(!bootstrap_key(&path, "https://sync.example.test", &"b".repeat(64)).unwrap());
        let contents = fs::read_to_string(dir.path().join("store/auth/admin.json")).unwrap();
        assert!(!contents.contains(&key));
        assert!(contents.contains(&disk::hash_secret(&key).unwrap()));
        assert_eq!(
            fs::metadata(dir.path().join("store/auth/admin.json"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        assert!(!bootstrap_key(&path, "https://sync.example.test", "senha-fraca").unwrap());
    }
    fn request(method: &str, target: &str, cookie: &str, body: &str, origin: &str) -> Request {
        let cookie = cookie.split(';').next().unwrap_or("");
        let head = format!(
            "{method} {target} HTTP/1.1\r\nHost: sync.example.test\r\nCookie: {cookie}\r\nOrigin: {origin}\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\n\r\n",
            body.len()
        );
        Request::parse(head.as_bytes()).unwrap()
    }
    fn token(page: &str, name: &str) -> String {
        page.split(&format!("name=\"{name}\" value=\""))
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap()
            .to_owned()
    }
    fn authenticated(admin: &mut Admin, key: &str) -> (String, String, String) {
        let response = admin.respond(&request("GET", "/", "", "", ""), &mut io::empty());
        let cookie = response.cookie.unwrap();
        assert!(
            cookie.contains("Secure")
                && cookie.contains("HttpOnly")
                && cookie.contains("SameSite=Strict")
        );
        let csrf = token(&response.body, "csrf");
        let body = format!("csrf={csrf}&key={key}");
        let response = admin.respond(
            &request(
                "POST",
                "/admin/login",
                &cookie,
                &body,
                "https://sync.example.test",
            ),
            &mut body.as_bytes(),
        );
        assert_eq!(response.status, 303);
        let cookie = response.cookie.unwrap();
        let response = admin.respond(&request("GET", "/", &cookie, "", ""), &mut io::empty());
        assert_eq!(response.status, 200);
        (
            cookie,
            token(&response.body, "csrf"),
            token(&response.body, "revision"),
        )
    }
    #[test]
    fn config_edits_preserve_settings_comments_and_reject_stale_or_invalid_changes() {
        let (_dir, path, _) = fixture();
        let base = config::Snapshot::load(&path).unwrap();
        config::update(
            &path,
            &base.revision,
            base.config.storage_root(),
            config::Change::User {
                name: "bruno",
                enabled: true,
            },
        )
        .unwrap();
        let updated = config::Snapshot::load(&path).unwrap();
        assert_eq!(updated.config.max_upload_bytes(), 12345);
        let bytes = fs::read(&path).unwrap();
        assert!(String::from_utf8_lossy(&bytes).contains("# manter comentário"));
        let error = config::update(
            &path,
            &base.revision,
            base.config.storage_root(),
            config::Change::Share { name: "Projetos" },
        )
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
        for change in [
            config::Change::User {
                name: "../escape",
                enabled: true,
            },
            config::Change::Share { name: "../escape" },
            config::Change::Permission {
                share: "Obras",
                user: "inexistente",
                access: "write",
            },
        ] {
            assert!(
                config::update(
                    &path,
                    &updated.revision,
                    updated.config.storage_root(),
                    change
                )
                .is_err()
            );
            assert_eq!(fs::read(&path).unwrap(), bytes);
        }
        config::update(
            &path,
            &updated.revision,
            updated.config.storage_root(),
            config::Change::Permission {
                share: "Obras",
                user: "bruno",
                access: "read",
            },
        )
        .unwrap();
        let config = Config::load(&path).unwrap();
        assert!(config.allows("bruno", "Obras", Operation::Read));
        assert!(!config.allows("bruno", "Obras", Operation::Write));
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    #[test]
    fn inline_toml_and_quoted_share_names_are_supported() {
        let (dir, path, _) = fixture();
        fs::write(&path, format!("version=1\nserver={{storage_root='{}'}}\nusers={{ana={{enabled=true}}}}\nshares={{Obras={{permissions={{ana='write'}}}}}}\n", dir.path().join("store").display())).unwrap();
        let current = config::Snapshot::load(&path).unwrap();
        config::update(
            &path,
            &current.revision,
            current.config.storage_root(),
            config::Change::Share {
                name: "Projetos.2026",
            },
        )
        .unwrap();
        let current = config::Snapshot::load(&path).unwrap();
        config::update(
            &path,
            &current.revision,
            current.config.storage_root(),
            config::Change::Permission {
                share: "Projetos.2026",
                user: "ana",
                access: "write",
            },
        )
        .unwrap();
        assert!(
            Config::load(&path)
                .unwrap()
                .allows("ana", "Projetos.2026", Operation::Write)
        );
    }
    #[test]
    fn key_hash_is_private_and_rotation_preserves_device_credentials() {
        let (_dir, path, key) = fixture();
        let config = Config::load(&path).unwrap();
        let auth = AuthStore::open(config.storage_root()).unwrap();
        let invite = auth.invite("ana", 600).unwrap();
        let device = auth.activate(&invite, "pc", |_| true).unwrap();
        let saved = config.storage_root().join("auth/admin.json");
        let data = fs::read_to_string(&saved).unwrap();
        assert!(!data.contains(&key));
        assert!(data.contains(&disk::hash_secret(&key).unwrap()));
        assert_eq!(
            fs::metadata(saved).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_ne!(reset_key(&path, "https://sync.example.test").unwrap(), key);
        assert_eq!(auth.authenticate(&device.token).unwrap(), "ana");
    }
    #[test]
    fn authentication_csrf_origin_logout_and_key_rotation_protect_mutations() {
        let (_dir, path, key) = fixture();
        let mut admin = Admin::new(&path).unwrap();
        let initial = fs::read(&path).unwrap();
        let (cookie, csrf, revision) = authenticated(&mut admin, &key);
        let body = format!("csrf={csrf}&revision={revision}&user=bruno&enabled=true");
        for (bad_cookie, bad_origin, bad_body) in [
            ("", "https://sync.example.test", body.clone()),
            (cookie.as_str(), "https://attacker.example", body.clone()),
            (
                cookie.as_str(),
                "https://sync.example.test",
                body.replace(&csrf, "wrong"),
            ),
            (
                cookie.as_str(),
                "https://sync.example.test",
                format!("revision={revision}&user=bruno&enabled=true"),
            ),
        ] {
            assert_eq!(
                admin
                    .respond(
                        &request("POST", "/admin/user", bad_cookie, &bad_body, bad_origin),
                        &mut bad_body.as_bytes()
                    )
                    .status,
                403
            );
            assert_eq!(fs::read(&path).unwrap(), initial);
        }
        assert_eq!(
            admin
                .respond(
                    &request(
                        "POST",
                        "/admin/user",
                        &cookie,
                        &body,
                        "https://sync.example.test"
                    ),
                    &mut body.as_bytes()
                )
                .status,
            303
        );
        assert!(Config::load(&path).unwrap().user_enabled("bruno"));
        assert_eq!(
            admin
                .respond(
                    &request(
                        "POST",
                        "/admin/user",
                        &cookie,
                        &body,
                        "https://sync.example.test"
                    ),
                    &mut body.as_bytes()
                )
                .status,
            409
        );
        reset_key(&path, "https://sync.example.test").unwrap();
        let response = admin.respond(&request("GET", "/", &cookie, "", ""), &mut io::empty());
        assert!(response.body.contains("Entrar no painel"));
        let fresh_key = reset_key(&path, "https://sync.example.test").unwrap();
        let (cookie, csrf, _) = authenticated(&mut admin, &fresh_key);
        let body = format!("csrf={csrf}");
        let response = admin.respond(
            &request(
                "POST",
                "/admin/logout",
                &cookie,
                &body,
                "https://sync.example.test",
            ),
            &mut body.as_bytes(),
        );
        assert_eq!(response.status, 303);
        assert!(response.cookie.unwrap().contains("Max-Age=0"));
        assert!(
            admin
                .respond(&request("GET", "/", &cookie, "", ""), &mut io::empty())
                .body
                .contains("Entrar no painel")
        );
    }
    #[test]
    fn origin_binding_expiry_and_guest_sessions_cannot_be_bypassed() {
        let (_dir, path, key) = fixture();
        let mut admin = Admin::new(&path).unwrap();
        let (cookie, csrf, revision) = authenticated(&mut admin, &key);
        let mut head = request("GET", "/", &cookie, "", "");
        head.headers
            .insert("host".into(), "attacker.example".into());
        assert_eq!(admin.respond(&head, &mut io::empty()).status, 421);
        for session in admin.sessions.values_mut() {
            session.expires = 0;
        }
        let body = format!("csrf={csrf}&revision={revision}&user=bruno&enabled=true");
        assert_eq!(
            admin
                .respond(
                    &request(
                        "POST",
                        "/admin/user",
                        &cookie,
                        &body,
                        "https://sync.example.test"
                    ),
                    &mut body.as_bytes()
                )
                .status,
            403
        );
        let guest = admin.respond(&request("GET", "/", "", "", ""), &mut io::empty());
        let body = format!(
            "csrf={}&revision={revision}&user=bruno&enabled=true",
            token(&guest.body, "csrf")
        );
        assert_eq!(
            admin
                .respond(
                    &request(
                        "POST",
                        "/admin/user",
                        &guest.cookie.unwrap(),
                        &body,
                        "https://sync.example.test"
                    ),
                    &mut body.as_bytes()
                )
                .status,
            403
        );
    }
    #[test]
    fn html_and_header_boundaries_do_not_expose_tokens_or_execute_labels() {
        assert_eq!(
            views::escape("<script>\"&'"),
            "&lt;script&gt;&quot;&amp;&#39;"
        );
        for headers in [
            "Host: a\r\nHost: b",
            "Host: a\r\nContent-Length: 0\r\nContent-Length: 0",
            "Host: a\r\nTransfer-Encoding: chunked",
            "Host: a\r\nExpect: 100-continue",
        ] {
            assert!(
                Request::parse(
                    format!("POST /admin/user HTTP/1.1\r\n{headers}\r\n\r\n").as_bytes()
                )
                .is_err()
            );
        }
        let (_dir, path, key) = fixture();
        let mut admin = Admin::new(&path).unwrap();
        let (cookie, csrf, revision) = authenticated(&mut admin, &key);
        let invite = AuthStore::open(&admin.root)
            .unwrap()
            .invite("ana", 600)
            .unwrap();
        AuthStore::open(&admin.root)
            .unwrap()
            .activate(&invite, "<script>alert('device')</script>", |_| true)
            .unwrap();
        let response = admin.respond(&request("GET", "/", &cookie, "", ""), &mut io::empty());
        assert!(!response.body.contains("<script>"));
        assert!(response.body.contains("&lt;script&gt;"));
        assert!(!response.body.contains("token_hash"));
        assert!(!response.body.contains(&key));
        let body = format!("csrf={csrf}&revision={revision}");
        let mut request = request(
            "POST",
            "/admin/share",
            &cookie,
            &body,
            "https://sync.example.test",
        );
        request.length = Some(MAX_BODY + 1);
        assert_eq!(admin.respond(&request, &mut io::empty()).status, 413);
    }
}
