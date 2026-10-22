# Estado e retomada

## Release em publicação — 0.1.0-alpha.5

Raiz local automática: sync-root/watch-root consultam os compartilhamentos
permitidos a cada ciclo e sincronizam RAIZ/NOME. Assistente pergunta somente a raiz.
Comandos individuais preservados; cópias sem acesso ficam locais, arquivos avulsos
não são enviados. Migração do layout antigo é explícita. Decisão 0010, manual,
README, notas e exemplo systemd atualizados; protocolo compatível com alpha.4.

Rede e escrita Git liberadas na retomada. Alpha.5 conferida livre no GitHub.
prepare-release.py sincronizou versões, lock, metadados e downloads. Enviar a tag
antes de main; só expor links após conferir os oito assets públicos e hashes.

Validação local: scripts/check-pilot.sh passou (build/fmt/Clippy, 74 Rust, cinco
HTTPS, seis TLS reais, dois testes de entrada de certificado, site, instalação,
cinco cenários do assistente e quatro gates de release). Teste adicional do
assistente verifica novo compartilhamento descoberto sem reconfiguração e pasta
não autorizada ausente. Pacotes musl x86_64/ARM64 e fonte offline pendem da CI.

## Versão pública atual

**0.1.0-alpha.4**, para testes, GPL-3.0-only:
https://github.com/goncalvesl/crewsync/releases/tag/v0.1.0-alpha.4.
Tag imutável no commit `3942c05`. Pacotes linux-x86_64/linux-arm64, fonte vendorizado,
bootstrap install.sh e checksums: oito URLs públicos HTTP 200, quatro hashes
conferidos. ARM64 executado nativamente na CI. Não mover tags ou sobrescrever
assets de versões publicadas.

## Pedido e implementação

O mantenedor quer instalação simples em servidor remoto por domínio ou IPv4
público/VPN, além de rede local. Decisão 0009 atualiza o escopo. setup.sh pergunta
endereço de acesso, sem esquema/porta, valida antes de instalar e gera SAN DNS/IP.
A inicialização verifica HTTPS por loopback, sem exigir DNS público resolvido ou
acesso ao IP público por dentro da VPS. Exibe URL, pasta, ca.crt e convite. Cliente
pede URL real, sem sugerir IP LAN arbitrário. Bootstrap consulta a última release,
inclusive alpha, identifica CPU, baixa e confere o pacote.

CA privada automática e HTTPS validado, sem ACME/CA pública automática. Só ca.crt
é transferido aos clientes por meio confiável. Certificado de servidor vale 90 dias.
URLs/portas/curingas/IPv6/nomes inválidos e injeções são recusados antes de gravar.
Configurações existentes preservadas; atualização usa install.sh, com serviço
parado/backup. Troca de endereço já instalado exige novo certificado, ajuste dos
caminhos do serviço e distribuição do novo ca.crt; procedimento técnico no manual.

systemd de usuário quando disponível; sem login SSH exige enable-linger explícito.
Fallback nohup informa PID/log/start.sh, sem autostart no reboot. DNS A deve apontar
para o IPv4 público; firewall Linux/provedor e NAT precisam permitir a porta TCP
escolhida para os clientes. Não há instalação automática de regras de firewall.

## Validação

- scripts/check-pilot.sh passou: build/fmt/Clippy, 72 Rust, 5 HTTPS, 6 TLS reais
  (IPv4/DNS, nome e CA incorretos), 2 testes de entrada de certificado, site,
  instalação e gates. Nenhum serviço ou dado real foi alterado.
- Cinco testes do assistente: dois clientes Rust em URL DNS localhost com watch,
  convite oculto em PTY, configuração preservada, entradas inválidas sem gravar,
  caminhos especiais, systemd-analyze, nohup e checksum incorreto.
- Novo cenário instala sync.example.test sem DNS externo, verifica SAN/hostname
  correto/incorreto e HTTPS/SNI por curl --resolve em loopback.
- Packages passou: https://github.com/goncalvesl/crewsync/actions/runs/39828480398.
  x86_64/ARM64 nativos: build/testes/HTTPS/ENOSPC isolado, instalação/setup do
  pacote; fonte correspondente compilou offline com CARGO_HOME vazio.
- Rust da tag: https://github.com/goncalvesl/crewsync/actions/runs/39828480327.
- Pacote x86_64 efetivamente baixado da release passou checksum, instalação,
  --version e cinco testes de setup. Bootstrap público instalou servidor com
  domínio e dois clientes em PTYs isolados; SAN/SNI/HTTPS e watch passaram,
  sem eco de convite. Testes não alteram DNS público ou /etc/hosts.
- Chromium local passou em 390/768/1440px, navegação/âncoras/overflow/console.
  Validação externa em VPS/Internet real continua a cargo do operador; testes
  loopback não são apresentados como acesso remoto físico validado.

## Manual e site

README/manual/site têm comandos únicos e downloads versionados da alpha.4.
Manual explica domínio/IPv4, DNS A, firewall/provedor, funcionamento sem SSH,
certificado privado e renovação; detalhes técnicos ficam na última seção.
https://goncalvesl.github.io/crewsync/manual.html#instalar.
Assets conferidos antes de enviar main. Pages passou na execução 39828921228:
https://github.com/goncalvesl/crewsync/actions/runs/39828921228.
Início/manual públicos retornaram HTTP 200 com alpha.4, downloads por arquitetura
e instruções de domínio/DNS/firewall/linger. Chromium no site público passou em
390/768/1440px (navegação, âncoras, overflow e console).

## Publicar novamente

Tarefa recorrente em AGENTS.md e RELEASING.md: incrementar, testar, publicar e
verificar release/downloads por arquitetura, manual/README/site. Não encerrar no
commit/tag. prepare-release.py mantém versão/URLs sincronizados. Enviar tag antes
de main; verificar assets antes de expor os links. Incluir bootstrap/checksum e
testar terminal real. Alpha.2 foi preservada e substituída pela alpha.3, que
corrigiu eco de convite em resposta imediata; regressão permanece nesta versão.

## Pendências externas

Piloto básico em duas máquinas confirmado. Matriz física ampliada e medições
pendentes; extended_matrix_accepted=false. Alpha autorizada para testes; estável
exige aceite físico real. Linux >=5.6 com /proc e disco local, servidor serializado,
sem quotas totais/retomada parcial/compactação/limpeza de cópias locais. Windows,
GUI e Android futuros. Não rebaixar após histórico v2; manter backup independente.
