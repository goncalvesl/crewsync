# Estado e retomada

## Versão pública atual

**0.1.0-alpha.1**, publicada para testes a pedido do mantenedor:
https://github.com/goncalvesl/crewsync/releases/tag/v0.1.0-alpha.1.
Tag imutável `v0.1.0-alpha.1`, commit `df2495b`; licença GPL-3.0-only.

A release oferece pacotes Linux estáticos musl x86_64 e ARM64, checksums SHA-256
e fonte correspondente com dependências vendorizadas. Os seis downloads públicos
retornaram HTTP 200; os três arquivos baixados tiveram checksums conferidos.
O pacote x86_64 publicado foi extraído/instalado em PREFIX temporário e executou
--version, --help e check-config; os dois binários informam 0.1.0-alpha.1.
ARM64 foi executado e validado nativamente na CI.

## Publicar novamente

Tarefa recorrente registrada em AGENTS.md e docs/RELEASING.md. Quando o mantenedor
pedir publicação, incrementar versão, testar, publicar e verificar a release,
atualizar os downloads de cada arquitetura no manual/README/site e verificar o
site publicado. Não encerrar no commit ou no início de workflow.

`python3 scripts/prepare-release.py VERSAO` atualiza Cargo/Cargo.lock, dependências
internas, docs/release.json e blocos de download. `--check` detecta divergências.
Usar URLs versionados, inclusive alpha (GitHub latest não seleciona prereleases).
Enviar tag antes de main; publicar/verificar os assets antes de expor seus links
em Pages. Não mover tags ou sobrescrever assets publicados.

## Implementado

- Rust nativo para HTTPS/TOML/SHA-256, sem Python/Cargo/OpenSSL no runtime.
- Servidor central raw por hash, deduplicação, revisões/tombstones, streaming,
  temporários, publicação durável, recuperação e caminhos protegidos.
- Convite de uso único, token por dispositivo, hashes no servidor, revogação,
  ACLs read/write, negação padrão e recarga que mantém última configuração válida.
- Sync/watch com eventos e reconciliação periódica, backoff, snapshots e conflitos
  preservados; diretórios vazios, exclusões e rename como exclusão/criação.
- Histórico e restauração por revisão; retenção/lixeira configuráveis e coleta
  offline com lock exclusivo e simulação; backups/restauração verificáveis.
- Instalação --client/--server ou ambos; manual/licença/auxiliares em share/crewsync.
  Objetivo: simples de instalar, manter e usar, além de rápido. Manual simples
  primeiro e detalhes técnicos na última seção; sem exigir ajuste do PATH.
- Site/Pages, pacotes e automação de publicação por tag, fonte compilável offline.

## Validação desta release

- `scripts/check-pilot.sh`: build/fmt/Clippy, 72 Rust locais, 5 HTTPS explícitos,
  3 TLS, gerador de certificados, instalador e testes de preparação de publicação.
- 4 testes de release: sincronização de arquitetura/versão/URLs, links obsoletos,
  gates de licença/tag/aceite e Cargo.lock desatualizado. 3 testes do renderizador
  do manual e 4 cenários de instalação passaram. Links/âncoras estáticos passaram.
- Fonte local compilou offline com CARGO_HOME vazio. Pacote local e pacote
  efetivamente baixado da release passaram checksum e instalação.
- QA Chromium local: 390/768/1440px, navegação/âncoras/overflow/console passaram.
- CI completa (incluindo navegador) da tag passou:
  https://github.com/goncalvesl/crewsync/actions/runs/39817489511.
- Packages da tag passou integralmente, inclusive publicação:
  https://github.com/goncalvesl/crewsync/actions/runs/39817489697.
  x86_64 e ARM64: build/fmt/Clippy/testes, HTTPS, ENOSPC real em tmpfs isolado,
  instalador, pacote e checksum. Fonte: build offline sem cache de crates.
- `git diff --check` passou. Nenhum serviço ou dado real foi alterado.

## Site e documentação

Manual, README e site têm downloads diretos da 0.1.0-alpha.1 para cada arquitetura,
checksums e fonte. Site: https://goncalvesl.github.io/crewsync/.
Pages passou na execução 39818128099:
https://github.com/goncalvesl/crewsync/actions/runs/39818128099.
Início e manual públicos retornaram HTTP 200 com a versão e os quatro links de
pacote/checksum corretos. QA Chromium no endereço público passou em
390/768/1440px. Assets foram conferidos antes de expor os novos links no site.

## Pendências e limites

O piloto básico em duas máquinas foi confirmado pelo mantenedor. A matriz física
ampliada e medições entre máquinas continuam pendentes; o registro físico permanece
basic_pilot_accepted=true e extended_matrix_accepted=false. A solicitação expressa
autoriza alpha pública de teste; versão estável continua bloqueada sem aceite
físico. Decisão 0008 registra essa atualização do escopo, sem inventar resultados.

Linux kernel >=5.6 com /proc e filesystem local; rede privada, servidor serializado.
Sem quotas totais, compactação, retomada parcial ou limpeza das cópias locais.
Não rebaixar após histórico v2; backup independente necessário. Windows, GUI e
Android ficam para etapas futuras. Benchmark localhost em docs/reports não equivale
a ensaio físico ou promessa de desempenho.
