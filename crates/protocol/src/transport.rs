//! HTTPS nativo, TLS 1.2/1.3, CA explícita e corpos delimitados em streaming.
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

pub fn exchange(
    url: &str,
    ca: &Path,
    request: Request,
    reader: &mut dyn Read,
    size: u64,
    output: &mut dyn Write,
    limit: u64,
) -> io::Result<()> {
    let url = url::Url::parse(url).map_err(|_| invalid("URL HTTPS inválida"))?;
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
    {
        return Err(invalid(
            "use URL HTTPS da raiz, sem credenciais ou parâmetros",
        ));
    }
    let host = url.host_str().ok_or_else(|| invalid("host ausente"))?;
    let host = host.trim_start_matches('[').trim_end_matches(']');
    let port = url.port().unwrap_or(443);
    let certs = rustls_pemfile::certs(&mut BufReader::new(std::fs::File::open(ca)?))
        .collect::<Result<Vec<_>, _>>()?;
    if certs.is_empty() {
        return Err(invalid("CA vazia"));
    }
    let mut roots = rustls::RootCertStore::empty();
    for cert in certs {
        roots.add(cert).map_err(|_| invalid("CA inválida"))?;
    }
    let config = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let name = rustls::pki_types::ServerName::try_from(host.to_owned())
        .map_err(|_| invalid("host TLS inválido"))?;
    let connection =
        rustls::ClientConnection::new(Arc::new(config), name).map_err(io::Error::other)?;
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
    let mut tls = rustls::StreamOwned::new(connection, tcp);
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
        tls,
        "POST /v1/rpc HTTP/1.1\r\nHost: {}\r\nContent-Type: application/octet-stream\r\nConnection: close\r\nContent-Length: {total}\r\n\r\n",
        url.authority()
    )?;
    tls.write_all(&line)?;
    if io::copy(&mut reader.take(size), &mut tls)? != size {
        return Err(invalid("arquivo mudou durante envio"));
    }
    tls.flush()?;
    let mut input = BufReader::new(tls);
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
    let mut bytes = Vec::new();
    exchange(url, ca, request, &mut io::empty(), 0, &mut bytes, MAX_JSON)?;
    serde_json::from_slice(&bytes).map_err(|_| invalid("resposta JSON inválida"))
}

#[cfg(test)]
mod tests {
    use super::*;
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
