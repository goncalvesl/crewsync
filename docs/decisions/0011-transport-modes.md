# 0011 — HTTP explícito, proxy HTTPS e HTTPS direto

O mantenedor autorizou três opções de transporte. HTTP serve para testes isolados;
HTTPS pode terminar em proxy do operador ou no próprio CrewSync. A emissão e
renovação de certificados públicos (como Let's Encrypt) ficam com o operador/proxy.
A CA privada automática continua disponível para instalação sem proxy.

## Comportamento

- `serve CONFIG HOST PORT CERT KEY` preserva HTTPS direto; aceita certificado e
  chave existentes, incluindo fullchain público. A CA privada gerada é opcional.
- `serve-http CONFIG HOST PORT` é explícito; o assistente de proxy fixa loopback.
  HTTP de teste também usa loopback por padrão; outro bind exige escolha consciente.
- O cliente usa uma CA privada, confiança nativa do sistema (`system`) ou HTTP
  explicitamente habilitado (`insecure-http`). Não há fallback, redirect, nem
  opção de ignorar verificação de certificados TLS.
- O assistente confirma HTTP com a palavra HTTP antes de gravar configuração ou
  pedir convite. HTTPS direto permanece opção padrão para preservar a instalação
  independente de infraestrutura adicional. Proxy gera Caddyfile, sem instalar,
  configurar firewall ou declarar pronto o endpoint HTTPS externo.

Sessões antigas com caminho de CA permanecem válidas. Novas sessões guardam CA
opcional e allow_http, falso por padrão. Combinações CA+HTTP e URL incompatível
são recusadas. rustls-native-certs carrega a confiança da plataforma; Linux precisa
do conjunto de CAs do sistema (normalmente ca-certificates). Alterar o endpoint
exige nova sessão/raiz ou migração explícita, pois estados locais são vinculados à URL.

O framing, streaming, limites, autorização, token por dispositivo e worker são os
mesmos nos três modos. Não confiar em X-Forwarded-* para autenticação e não expor
uma API de administração nesta etapa. O backend do proxy fica privado; containers
em outra rede exigem bind/rede restritos configurados pelo operador, pois loopback
do container não é o loopback do host.

## Evolução

Uma interface simples para cadastrar pastas e usuários é o próximo pedido do
mantenedor. Ela deverá ter autenticação administrativa própria e publicação
validada/atômica da configuração. A escolha de HTTP/TLS não cria permissões de
administração e não torna o RPC de sincronização uma interface de gestão.

## Evidências e fontes

Testes: URL/modo explícito, sessões legadas, confirmação/recusa de HTTP,
certificado existente, confiança nativa sem ca.crt no cliente e Caddy real com
backend HTTP, 1 MiB transferido e revogação. ACME público depende de DNS/Internet;
o teste usa certificados sintéticos e portas efêmeras, sem emitir certificados reais.

- [Caddy: proxy reverso](https://caddyserver.com/docs/caddyfile/directives/reverse_proxy)
- [Caddy: HTTPS automático](https://caddyserver.com/docs/automatic-https)
- [Confiança nativa e variáveis SSL_CERT_FILE/SSL_CERT_DIR](https://docs.rs/rustls-native-certs/latest/rustls_native_certs/fn.load_native_certs.html)
