# Estado e retomada

## Versão pública atual

**0.1.0-alpha.6**, para testes, GPL-3.0-only:
https://github.com/goncalvesl/crewsync/releases/tag/v0.1.0-alpha.6.
Tag imutável em `1a48023`. Oito assets públicos HTTP 200: pacotes musl x86_64 e
ARM64, checksums, fonte vendorizado, bootstrap install.sh e checksum. Quatro
hashes conferidos após download. Não mover tags ou sobrescrever assets publicados.
Alpha.5 permanece tag em `8a2808f`, sem release: a CI ARM64 detectou uma corrida
na limpeza de processos dos testes; correção incluída na alpha.6.

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
