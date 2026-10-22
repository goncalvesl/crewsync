# Estado e retomada

## Preparação da 0.1.0-alpha.4

Pedido atual: instalação fácil em servidor remoto, por domínio ou IPv4 público.
Decisão 0009 amplia o escopo. setup.sh pede endereço de acesso sem esquema/porta,
valida por pilot-cert.sh --check-address antes de gravar, gera SAN DNS/IP e exibe
URL/pasta/CA/convite. Inicialização verifica loopback e independe do DNS externo.
Cliente pede URL real, sem sugerir um IP LAN arbitrário. CA privada e TLS validado
preservados; sem ACME/CA pública automático. Configurações existentes preservadas.

scripts/check-pilot.sh passou: build/fmt/Clippy, 72 Rust, 5 HTTPS, agora 6 TLS
(IP/DNS), 2 validações de certificado e 5 testes do assistente. Novo cenário instala
com sync.example.test sem DNS, verifica SAN/nome incorreto e HTTPS/SNI via --resolve.
Dois clientes Rust usam localhost como DNS. Manual explica DNS A, firewall Linux/
provedor, linger e renovação. Não há alegação de teste em servidor público real.
Pacote estático x86_64 passou checksum, instalação e os 5 testes de setup.
Chromium local 390/768/1440px passou. Publicar/verificar assets conforme
RELEASING.md antes de enviar main.

## Versão pública atual

**0.1.0-alpha.3**, para testes, GPL-3.0-only:
https://github.com/goncalvesl/crewsync/releases/tag/v0.1.0-alpha.3.
Tag imutável no commit `e10693b`. Downloads linux-x86_64/linux-arm64, SHA-256,
fonte vendorizado e bootstrap install.sh com checksum: oito URLs HTTP 200,
quatro hashes públicos conferidos. ARM64 executado nativamente na CI.

Alpha.2 preservada e marcada como substituída nas notas. O teste adicional do
bootstrap público detectou eco de convite em resposta imediata ao prompt; alpha.3
desativa o eco antes do prompt e inclui regressão em terminal real. Não mover tags
nem sobrescrever assets de versões já publicadas.

## Instalação e operação

scripts/setup-online.sh consulta a última release, inclusive alpha, detecta CPU,
baixa e confere o pacote. scripts/setup.sh também vem no pacote. Servidor pergunta
pastas/IP/porta/usuário/compartilhamento, cria configuração/certificados HTTPS,
inicia e fornece convite. Cliente pede diretório/CA/endereço/convite e ativa watch.
Sem Cargo/Python no runtime; OpenSSL só para gerar certificados. HTTPS permanece
validado, convite oculto, configurações existentes preservadas e binários trocados
atomicamente por install.sh. Atualização existente usa install.sh, com serviço
parado e backup; o assistente é para configurações novas.

systemd de usuário quando disponível; sem login exige enable-linger explícito.
Fallback nohup informa PID/log/start.sh, sem autostart no reboot. Firewall e acesso
entre computadores precisam permitir a porta TCP escolhida. Certificado de 90 dias;
renovação documentada. Diretórios com espaços/aspas/%/$ testados.

## Validação

- scripts/check-pilot.sh passou novamente na alpha.3: build/fmt/Clippy, 72 Rust,
  5 HTTPS, 3 TLS, certificado, site, instalador e gates de release.
- Quatro testes de setup: HTTPS real e dois clientes watch, ativação em PTY com
  resposta imediata sem eco, configuração preservada, entradas inválidas,
  systemd-analyze, nohup e bootstrap alpha simulado/checksum errado.
- Rust da tag passou: https://github.com/goncalvesl/crewsync/actions/runs/39823475705.
- Packages da tag: https://github.com/goncalvesl/crewsync/actions/runs/39823475726.
  Matriz x86_64/ARM64, HTTPS, ENOSPC isolado, instalação do pacote e build offline
  do fonte. Publicação após os três jobs de construção/testes bem-sucedidos.
- Pacote x86_64 público passou checksum, --version, instalação e quatro testes de
  setup. Bootstrap efetivamente baixado do GitHub instalou servidor e dois clientes
  em PTYs isolados, validou HTTPS e sincronizou arquivo por watch; convite oculto.
- Fonte local compilou offline com CARGO_HOME vazio; CI repete no fonte da tag.
  QA Chromium local 390/768/1440px passou. Nenhum serviço ou dado real foi alterado.

## Manual e site

README/manual/site oferecem comandos únicos de servidor/cliente e downloads
versionados da alpha.3. Detalhes técnicos na seção final do manual.
https://goncalvesl.github.io/crewsync/manual.html#instalar.
Assets verificados antes de enviar main. Pages passou na execução 39824036351:
https://github.com/goncalvesl/crewsync/actions/runs/39824036351.
Início/manual públicos retornaram HTTP 200 com alpha.3, downloads por arquitetura
e comandos de servidor/cliente. Chromium no site público passou em 390/768/1440px
(navegação, âncoras, overflow e console). O bootstrap em main foi baixado e conferido
byte a byte com o publicado/testado na release.

## Publicar novamente

Tarefa recorrente em AGENTS.md e RELEASING.md: incrementar, testar, publicar e
conferir release/downloads por arquitetura, manual/README/site. Não encerrar no
commit/tag. prepare-release.py mantém versão/URLs sincronizados. Enviar tag antes
de main; verificar os assets antes de expor seus links em Pages. Incluir bootstrap
e checksum e validar o fluxo com terminal real.

## Pendências externas

Piloto básico em duas máquinas confirmado pelo mantenedor. Matriz física ampliada
e medições continuam pendentes; extended_matrix_accepted=false. Alpha autorizada
para testes; estável depende de aceite físico real. Linux >=5.6 com /proc e disco
local, rede privada e servidor serializado. Windows/GUI/Android futuros. Sem quotas
totais, retomada parcial, compactação ou limpeza de cópias locais. Não rebaixar após
histórico v2; backup independente necessário. Benchmark local não equivale a ensaio
físico nem promessa de desempenho.
