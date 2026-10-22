//! HTTP explícito ou HTTPS com CA privada/padrão do sistema, sempre em streaming.
use crate::{Frame, MAX_FRAME, MAX_JSON, Request, invalid};
use serde::de::DeserializeOwned;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

pub fn read_line(reader: &mut impl BufRead) -> io::Result<Vec<u8>> {
    let mut line = Vec::new();
    reader.take(MAX_FRAME + 1).read_until(b'\n', &mut line)?;
    if line.len() as u64 > MAX_FRAME || !line.ends_with(b"\n") {
        return Err(invalid("frame incompleto ou longo demais"));
    }
    Ok(line)
}

pub fn read_http_head(reader: &mut impl BufRead) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    loop {
        let remaining = (16384usize.saturating_sub(bytes.len())) as u64;
        if remaining == 0 {
            return Err(invalid("cabeçalho HTTP excede limite"));
        }
        let before = bytes.len();
        reader.take(remaining).read_until(b'\n', &mut bytes)?;
        if bytes.len() == before || !bytes.ends_with(b"\r\n") {
            return Err(invalid("HTTP incompleto"));
        }
        if bytes.ends_with(b"\r\n\r\n") {
            return Ok(bytes);
        }
    }
}

pub fn content_length(headers: &[httparse::Header<'_>]) -> io::Result<u64> {
    let mut length = None;
    for header in headers {
        if header.name.eq_ignore_ascii_case("transfer-encoding")
            || header.name.eq_ignore_ascii_case("expect")
        {
            return Err(invalid("codificação HTTP não suportada"));
        }
        if header.name.eq_ignore_ascii_case("content-length") {
            if length.is_some()
                || header.value.is_empty()
                || !header.value.iter().all(u8::is_ascii_digit)
            {
                return Err(invalid("Content-Length inválido"));
            }
            length = Some(
                std::str::from_utf8(header.value)
                    .map_err(|_| invalid("Content-Length inválido"))?
                    .parse()
                    .map_err(|_| invalid("Content-Length inválido"))?,
            );
        }
    }
    length.ok_or_else(|| invalid("Content-Length ausente"))
}

/// HTTP precisa ser escolhido explicitamente; nunca há downgrade ou redirects.
#[derive(Clone, Copy)]
pub enum Trust<'a> {
    CustomCa(&'a Path),
    System,
    PlainHttp,
}

pub fn validate_url(value: &str, trust: Trust<'_>) -> io::Result<url::Url> {
    let url = url::Url::parse(value).map_err(|_| invalid("URL HTTP/HTTPS inválida"))?;
    let expected = if matches!(trust, Trust::PlainHttp) {
        "http"
    } else {
        "https"
    };
    if url.scheme() != expected
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
        || url.host_str().is_none()
    {
        return Err(invalid(
            "use URL HTTPS da raiz; HTTP requer modo insecure-http explícito; sem credenciais ou parâmetros",
        ));
    }
    Ok(url)
}

fn root_store(trust: Trust<'_>) -> io::Result<rustls::RootCertStore> {
    let certs = match trust {
        Trust::CustomCa(path) => {
            rustls_pemfile::certs(&mut BufReader::new(std::fs::File::open(path)?))
                .collect::<Result<Vec<_>, _>>()?
        }
        Trust::System => rustls_native_certs::load_native_certs().certs,
        Trust::PlainHttp => return Err(invalid("HTTP não usa certificados")),
    };
    if certs.is_empty() {
        return Err(invalid(
            "nenhuma CA disponível; confira ca.crt ou ca-certificates do sistema",
        ));
    }
    let mut roots = rustls::RootCertStore::empty();
    for cert in certs {
        roots.add(cert).map_err(|_| invalid("CA inválida"))?;
    }
    Ok(roots)
}

trait Connection: Read + Write {}
impl<T: Read + Write> Connection for T {}

pub fn exchange(
    url: &str,
    ca: &Path,
    request: Request,
    reader: &mut dyn Read,
    size: u64,
    output: &mut dyn Write,
    limit: u64,
) -> io::Result<()> {
    exchange_with(
        url,
        Trust::CustomCa(ca),
        request,
        reader,
        size,
        output,
        limit,
    )
}

pub fn exchange_with(
    url: &str,
    trust: Trust<'_>,
    request: Request,
    reader: &mut dyn Read,
    size: u64,
    output: &mut dyn Write,
    limit: u64,
) -> io::Result<()> {
    let url = validate_url(url, trust)?;
    let host = url.host_str().ok_or_else(|| invalid("host ausente"))?;
    let host = host.trim_start_matches('[').trim_end_matches(']');
    let port = url
        .port_or_known_default()
        .ok_or_else(|| invalid("porta ausente"))?;
    let tls = if matches!(trust, Trust::PlainHttp) {
        None
    } else {
        let config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store(trust)?)
            .with_no_client_auth();
        let name = rustls::pki_types::ServerName::try_from(host.to_owned())
            .map_err(|_| invalid("host TLS inválido"))?;
        Some(rustls::ClientConnection::new(Arc::new(config), name).map_err(io::Error::other)?)
    };
    let mut last = io::Error::new(io::ErrorKind::AddrNotAvailable, "host sem endereço");
    let mut connected = None;
    for address in (host, port).to_socket_addrs()? {
        match TcpStream::connect_timeout(&address, Duration::from_secs(10)) {
            Ok(tcp) => {
                connected = Some(tcp);
                break;
            }
            Err(e) => last = e,
        }
    }
    let tcp = connected.ok_or(last)?;
    tcp.set_read_timeout(Some(Duration::from_secs(30)))?;
    tcp.set_write_timeout(Some(Duration::from_secs(30)))?;
    let mut connection: Box<dyn Connection> = match tls {
        Some(tls) => Box::new(rustls::StreamOwned::new(tls, tcp)),
        None => Box::new(tcp),
    };
    let mut line = serde_json::to_vec(&Frame {
        request,
        body_bytes: size,
    })
    .map_err(io::Error::other)?;
    line.push(b'\n');
    if line.len() as u64 > MAX_FRAME {
        return Err(invalid("requisição longa demais"));
    }
    let total = size
        .checked_add(line.len() as u64)
        .ok_or_else(|| invalid("corpo longo demais"))?;
    write!(
        connection,
        "POST /v1/rpc HTTP/1.1\r\nHost: {}\r\nContent-Type: application/octet-stream\r\nConnection: close\r\nContent-Length: {total}\r\n\r\n",
        url.authority()
    )?;
    connection.write_all(&line)?;
    if io::copy(&mut reader.take(size), &mut connection)? != size {
        return Err(invalid("arquivo mudou durante envio"));
    }
    connection.flush()?;
    let mut input = BufReader::new(connection);
    let head = read_http_head(&mut input)?;
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut response = httparse::Response::new(&mut headers);
    if !response
        .parse(&head)
        .map_err(|_| invalid("resposta HTTP inválida"))?
        .is_complete()
    {
        return Err(invalid("resposta HTTP incompleta"));
    }
    let status = response.code.ok_or_else(|| invalid("status ausente"))?;
    if status != 200 {
        let kind = match status {
            401 | 403 => io::ErrorKind::PermissionDenied,
            404 => io::ErrorKind::NotFound,
            409 => io::ErrorKind::AlreadyExists,
            400 => io::ErrorKind::InvalidInput,
            _ => io::ErrorKind::ConnectionAborted,
        };
        return Err(io::Error::new(
            kind,
            format!("servidor respondeu HTTP {status}"),
        ));
    }
    let length = content_length(response.headers)?;
    if length > limit {
        return Err(invalid("resposta excede limite"));
    }
    if io::copy(&mut input.take(length), output)? != length {
        return Err(invalid("download interrompido"));
    }
    Ok(())
}

pub struct TlsServer(Arc<rustls::ServerConfig>);
impl TlsServer {
    pub fn new(cert: &Path, key: &Path) -> io::Result<Self> {
        let certs = rustls_pemfile::certs(&mut BufReader::new(std::fs::File::open(cert)?))
            .collect::<Result<Vec<_>, _>>()?;
        let key = rustls_pemfile::private_key(&mut BufReader::new(std::fs::File::open(key)?))?
            .ok_or_else(|| invalid("chave TLS ausente ou inválida"))?;
        let config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|_| invalid("certificado/chave TLS inválidos"))?;
        Ok(Self(Arc::new(config)))
    }
    pub fn accept(&self, tcp: TcpStream) -> io::Result<impl Read + Write + use<>> {
        tcp.set_read_timeout(Some(Duration::from_secs(30)))?;
        tcp.set_write_timeout(Some(Duration::from_secs(30)))?;
        let connection = rustls::ServerConnection::new(self.0.clone()).map_err(io::Error::other)?;
        Ok(rustls::StreamOwned::new(connection, tcp))
    }
}

pub fn request_length(head: &[u8]) -> io::Result<u64> {
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut request = httparse::Request::new(&mut headers);
    if !request
        .parse(head)
        .map_err(|_| invalid("HTTP inválido"))?
        .is_complete()
        || request.method != Some("POST")
        || request.path != Some("/v1/rpc")
        || request.version != Some(1)
    {
        return Err(invalid("requisição HTTP inválida"));
    }
    content_length(request.headers)
}

pub fn json<T: DeserializeOwned>(url: &str, ca: &Path, request: Request) -> io::Result<T> {
    json_with(url, Trust::CustomCa(ca), request)
}

pub fn json_with<T: DeserializeOwned>(
    url: &str,
    trust: Trust<'_>,
    request: Request,
) -> io::Result<T> {
    let mut bytes = Vec::new();
    exchange_with(
        url,
        trust,
        request,
        &mut io::empty(),
        0,
        &mut bytes,
        MAX_JSON,
    )?;
    serde_json::from_slice(&bytes).map_err(|_| invalid("resposta JSON inválida"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transport_mode_is_explicit_and_urls_cannot_inject_credentials_or_paths() {
        for trust in [Trust::System, Trust::CustomCa(Path::new("unused"))] {
            assert!(validate_url("https://sync.example.test:8443", trust).is_ok());
            assert!(validate_url("http://127.0.0.1:8080", trust).is_err());
        }
        assert!(validate_url("http://127.0.0.1:8080", Trust::PlainHttp).is_ok());
        assert!(validate_url("https://sync.example.test", Trust::PlainHttp).is_err());
        for url in [
            "https://user:password@sync.example.test",
            "https://sync.example.test/path",
            "https://sync.example.test/?token=secret",
            "https://sync.example.test/#secret",
            "ftp://sync.example.test",
        ] {
            assert!(validate_url(url, Trust::System).is_err());
        }
    }

    #[test]
    fn http_rejects_ambiguous_framing_and_bounded_headers() {
        for headers in [
            "Content-Length: 1\r\nContent-Length: 1",
            "Transfer-Encoding: chunked\r\nContent-Length: 1",
            "Content-Length: +1",
            "Content-Length: 18446744073709551616",
            "Expect: 100-continue\r\nContent-Length: 1",
            "X-Test: yes",
        ] {
            let head = format!("POST /v1/rpc HTTP/1.1\r\n{headers}\r\n\r\n");
            assert!(request_length(head.as_bytes()).is_err(), "{headers}");
        }
        assert_eq!(
            request_length(b"POST /v1/rpc HTTP/1.1\r\nContent-Length: 12\r\n\r\n").unwrap(),
            12
        );
        assert!(read_http_head(&mut &vec![b'x'; 20000][..]).is_err());
    }
}
