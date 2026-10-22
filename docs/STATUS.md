# Estado e retomada

## Versão pública atual — 0.1.0-alpha.7

Release alpha para testes: https://github.com/goncalvesl/crewsync/releases/tag/v0.1.0-alpha.7.
Tag v0.1.0-alpha.7 aponta ao commit 732c0d7. A CI 39860997484 passou nos jobs
de fonte offline, pacote musl x86_64 e ARM64 e publicação. Release não draft e
prerelease, com oito assets. Pacotes, checksums, fonte e install.sh foram baixados;
quatro checksums conferidos. check-package passou no tar.gz x86_64; ambos binários
reportam 0.1.0-alpha.7. O site e README estão atualizados para alpha.7 neste commit.

A versão inclui painel administrativo Rust no mesmo listener, HTTP explícito para
testes, HTTPS por proxy e HTTPS direto. Painel gerencia usuários, pastas virtuais,
ACLs, convites e dispositivos. Chave administrativa independente, rotação local,
sessões em memória, CSRF/origem e TOML validado com publicação atômica. Manual,
configuração, README e release notes atualizados.

Validação local completa: check-pilot passou com build, fmt, Clippy, 82 testes
Rust, cinco testes HTTPS explícitos, seis TLS MemoryBIO, dois testes de certificado,
onze cenários do instalador, painel HTTP/HTTPS, Caddy real, site e gates. Chromium
passou painel em 390/768/1440px sem overflow ou erros de console. Fonte da release
compilou offline; ambos pacotes musl passaram a matriz CI, inclusive teste ENOSPC,
HTTPS e instalação. Nenhum servidor de produção ou certificado real foi alterado.

O mantenedor definiu o servidor Linux-only. Decisão 0013 registra o plano de
pacotes Arch, Debian, Fedora e tar.gz independente; layout distro em /usr/bin,
/etc/crewsync e /var/lib/crewsync, e tar em /opt/crewsync com /etc/opt e /var/opt.
Isso é planejamento, ainda não implementado; migração do instalador por usuário
precisa ser desenhada.

## Histórico da implementação alpha.7 — transporte

Pedido atual implementado: HTTP explícito para testes, HTTPS por proxy e HTTPS
direto (CA privada automática ou certificado/chave existentes). HTTPS permanece
validado; cliente aceita CAs nativas do sistema ou CA privada. HTTP exige opt-in
no assistente e na sessão; não há downgrade nem redirects. Sessões antigas com
CA continuam válidas. Modo proxy usa backend em loopback e gera Caddyfile sem
instalar proxy, alterar firewall ou declarar o endpoint público pronto.

Decisão 0011, PROJECT/PLAN, manual, CONFIGURATION, README, exemplos e site local
atualizados. Manual traz instalação/configuração simples de Caddy, reuso de
Coolify, rede de containers, migração e renovação. O painel administrativo foi implementado na etapa seguinte, descrita acima.
A alpha.6 dos downloads não inclui os três modos; não publicar esses binários
como assets da tag existente nem mudar as versões antes de uma nova release.

Validação local: CREWSYNC_CADDY_BIN=/tmp/crewsync-caddy-qa/caddy
scripts/check-pilot.sh passou: build/fmt/Clippy, 76 Rust, cinco HTTPS, seis TLS
sem sockets, dois testes de entrada, site, onze cenários do assistente e quatro
gates de release. Caddy 2.11.7 real (arquivo oficial com SHA-512 conferido):
backend HTTP privado, dois clientes HTTPS com confiança nativa, 1 MiB transferido,
CA incorreta recusada e revogação. CI configurada para executar o teste com Caddy instalado.
Não houve emissão ACME nem alteração de certificados, serviços ou dados reais.

Pacote GNU local gerado/check-package passou checksum, instalação e onze cenários
do assistente; não é pacote musl de distribuição. Musl x86_64/ARM64 e fonte offline
precisam da CI na próxima release. Verificação do site: build/check, três testes
do gerador, quatro de release e prepare-release --check passaram. Nenhuma nova
release, tag, commit ou push foi feita. Chromium local passou
em 390/768/1440px, sem problemas de navegação, âncoras, overflow ou console.

## Versão pública atual

**0.1.0-alpha.6**, para testes, GPL-3.0-only:
https://github.com/goncalvesl/crewsync/releases/tag/v0.1.0-alpha.6.
Tag imutável em `1a48023`. Oito assets públicos HTTP 200: pacotes musl x86_64 e
ARM64, checksums, fonte vendorizado, bootstrap install.sh e checksum. Quatro
hashes conferidos após download. Não mover tags ou sobrescrever assets publicados.
Alpha.5 permanece tag em `8a2808f`, sem release: a CI ARM64 detectou uma corrida
na limpeza de processos dos testes; correção incluída na alpha.6.

## Atualização do manual e site

Orientações práticas para adicionar usuários e compartilhamentos no server.toml,
validar a configuração e receber novas pastas sem reiniciar ou reativar clientes.
Seção Dispositivos reúne convite de uso único por computador (inclusive para o
mesmo usuário), validade, credencial persistente e revogação individual. Homepage
e README apontam para os procedimentos. CONFIGURATION corrige referência histórica
a Python: o parser distribuído é Rust nativo. Publicação apenas documental;
essa etapa antecedeu a publicação alpha.7.

Validação: build/check do site, três testes do gerador, quatro gates de release e
prepare-release --check passaram. Exemplos TOML do manual foram validados pelo
binário real: um usuário e dois compartilhamentos, sem criar armazenamento.
Chromium local passou em 390/768/1440px: navegação, âncoras, overflow e console.
Pages passou na execução 39851520560, commit db91f1a:
https://github.com/goncalvesl/crewsync/actions/runs/39851520560.
Início e manual públicos retornaram HTTP 200 com as novas orientações e downloads
alpha.6. Chromium público passou em 390/768/1440px (navegação, âncoras, overflow e
console). Nenhum programa ou asset de release foi alterado.

## Comportamento novo do cliente

O assistente pergunta somente a raiz local, como `/home/voce/Terna`. sync-root e
watch-root consultam os compartilhamentos permitidos a cada ciclo e sincronizam
RAIZ/NOME (por exemplo, Terna/Obras). A descoberta de novas permissões é automática;
pastas sem acesso permanecem locais, sem sincronizar. Arquivos avulsos da raiz não
são enviados. Layout antigo exige migração explícita com serviço parado, incluindo
.crewsync; comandos individuais sync/watch continuam disponíveis. Decisão 0010,
manual, README, notas e exemplo systemd atualizados. Protocolo compatível com
servidor alpha.4, conferido com dois clientes; não é necessário reinstalar o servidor.

Fallback nohup herda HUP ignorado antes do exec, evitando morte ao fechar o terminal
imediatamente após instalar. Testes encerram e aguardam serviço e workers via pidfd
antes de remover fixtures. Regressões para HUP antes do exec e encerramento dos filhos.

## Validação da alpha.6

- scripts/check-pilot.sh passou: build/fmt/Clippy, 74 Rust, cinco HTTPS, seis TLS
  reais sem sockets, dois testes de entrada de certificado, site, instalação,
  sete cenários do assistente e quatro gates de release.
- Assistente cobre dois clientes HTTPS, convite oculto em PTY, subpastas por
  compartilhamento e nova permissão descoberta durante watch. Três repetições
  adicionais passaram após corrigir HUP. Regressão HUP falhou contra o script
  anterior e passou no novo.
- Chromium local 390/768/1440px: navegação, âncoras, overflow e console passaram.
- Packages passou: https://github.com/goncalvesl/crewsync/actions/runs/39848766589.
  x86_64/ARM64 nativos: build/testes/HTTPS/ENOSPC e instalação/setup de cada pacote;
  fonte correspondente compilou offline com CARGO_HOME vazio.
- Pacote x86_64 efetivamente baixado da release passou checksum, check-package,
  --version e sete testes de setup. ARM64 foi executado nativamente na CI.
- Bootstrap público em PTY baixou alpha.6, conferiu checksum, instalou cliente e
  sincronizou com outro cliente, descobrindo novo compartilhamento sem reconfigurar.
  Servidor e dados sintéticos isolados; nenhum serviço ou dado real foi alterado.
- Compatibilidade com servidor alpha.4 passou após corrigir HUP do assistente;
  o timeout visto antes da correção vinha do processo encerrado junto ao terminal.

## Operação e limites

Servidor remoto por domínio/IPv4 público ou VPN, além de LAN (decisão 0009).
CA privada automática e HTTPS validado; só ca.crt vai aos clientes por meio
confiável. Certificado do servidor: 90 dias; renovação no manual técnico.
DNS A deve apontar ao IPv4 público e firewall Linux/provedor permitir a porta TCP.
O operador confirmou acesso externo DNS/HTTPS em VPS Oracle após liberar a porta.
Isso não substitui a matriz física ampliada de sincronização.

systemd de usuário quando disponível; sem login SSH exige enable-linger explícito.
Fallback nohup sem autostart no reboot. Configurações existentes preservadas;
atualização por install.sh com serviço parado e backup. Não rebaixar após histórico
v2. Não há configuração automática de firewall nem limpeza das cópias locais.

Piloto básico em duas máquinas confirmado; matriz ampliada e medições entre máquinas
pendentes, extended_matrix_accepted=false. Alpha para testes; estável exige aceite
físico real. Linux >=5.6 com /proc e disco local, servidor serializado, sem quotas
totais/retomada parcial/compactação. Windows, GUI e Android futuros. Manter backup.

## Manual, site e próxima publicação

README/manual/site públicos com downloads versionados da alpha.6. Assets foram
conferidos antes de enviar main. Pages passou na execução 39849242000:
https://github.com/goncalvesl/crewsync/actions/runs/39849242000.
Início/manual retornaram HTTP 200 com alpha.6, downloads para ambas as arquiteturas
e instruções de raiz automática. Chromium no site público passou em 390/768/1440px
(navegação, âncoras, overflow e console).
https://goncalvesl.github.io/crewsync/manual.html#instalar.

Seguir docs/RELEASING.md até a release e documentação estarem públicas. Usar
prepare-release.py; enviar tag antes de main. Não encerrar em commit/tag/workflow.
