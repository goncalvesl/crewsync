# Estado e retomada

## Escopo e autorização

O usuário confirmou o piloto básico em dois computadores e autorizou executar
todas as etapas restantes sem paradas. Implementação Linux e preparação de
operação/site/distribuição foram realizadas. Windows/GUI/Android seguem como
evolução posterior do escopo definido. Licença foi perguntada ao mantenedor;
sem resposta até este registro, não foi atribuída por suposição.

## Implementado

- Armazenamento raw SHA-256 deduplicado, revisões/tombstones, streaming, temporários,
  publicação durável e recuperação. Limite de profundidade validado no protocolo.
- HTTPS/TOML/SHA-256 nativos em Rust (rustls, toml, sha2); sem Python, sha256sum,
  Cargo ou OpenSSL no runtime. Pontes temporárias removidas, protocolo preservado.
- Convites de uso único, token por dispositivo, hashes no servidor, revogação,
  ACLs read/write, deny padrão e recarga que mantém última configuração válida.
- Sync e watch por eventos notify com reconciliação periódica; fila limitada,
  debounce finito, ignorando metadata e symlinks; fallback periódico e backoff.
- Histórico e restauração como nova revisão; retenção configurável de versões e
  lixeira no servidor; histórico v1 legível com graça, v2 com datas de arquivamento.
- `maintain CONFIG [--apply]`: simulação, poda e coleta de objetos sem referências,
  inclusive compartilhamentos retirados da configuração; requer servidor parado
  e lock exclusivo. Publica metadados duráveis antes de remover objetos. Tombstones
  atuais nunca são coletados. Cópias locais não são limpas automaticamente.
- Backup com locks de armazenamento/autenticação, validação e marcador ao final;
  restauração em destino novo, sem sobrescrita. TLS deve ser copiado separadamente.
- Logs sem credenciais/usuário/caminho/conteúdo; systemd, manual de atualização,
  diagnóstico e recuperação de desastre. Veja MANUAL.md e OPERATIONS.md.
- Site estático responsivo, manual gerado de Markdown, workflows Pages e Packages;
  pacotes musl x86_64/ARM64, checksum, avisos de dependências, smoke de instalação.
  Publicação por tag exige licença, versão correspondente e aceite físico ampliado.

## Evidências locais

Arch Linux, kernel 6.18.57-2-lts, Ryzen 5 8600G, Rust/Cargo 1.99.1.
Dados sintéticos e temporários, sem alterar serviço, instalação ou dados reais.

- `scripts/check-pilot.sh`: build, fmt, Clippy, **72 Rust passaram**, 6 ignorados
  por padrão; **5 HTTPS passaram** explicitamente, **3 TLS/certificados passaram**,
  2 páginas verificadas e 1 regressão do renderizador do manual passou.
- HTTPS cobre dois clientes, conflitos, exclusões offline, revogação, TLS inválido,
  reinício, restauração de duas versões e upload truncado com recuperação do worker.
- Retenção/coleta: quantidade, idade, lixeira, prazo de graça, dados atuais,
  falhas de publicação antes/depois da poda. Backup: locks, tokens, destino novo,
  links e corrupção recusados, marcadores de conclusão/incompletude.
- `cargo build --release --workspace --locked`: passou.
- Build musl x86_64 em rust:alpine: passou; binário static-pie verificado por file.
- `scripts/check-disk-full.sh target/musl-build/release/deps`: **1 ENOSPC real passou**
  em tmpfs Docker descartável de 1 MiB; conteúdo confirmado preservado.
- Binário estático executou check-config em BusyBox sem Python, Cargo ou glibc.
- Pacote x86_64: checksum, extração, instalação em PREFIX temporário, --help,
  --version e check-config passaram. Arquivos em target/packages/.
- QA Chromium/Playwright: 390/768/1440px, navegação, âncoras, console e overflow
  passaram; screenshots locais em target/site-*.png, inspeção visual realizada.
- Benchmark real localhost: docs/reports/synthetic-local.json, 101 arquivos,
  4.219.904 bytes de conteúdo. CPU/tempo/ru_maxrss de processos cliente; tráfego
  no fio não medido. Não representa duas máquinas físicas ou promessa de desempenho.
- `git diff --check`: passou.

## Publicação e pendências externas

Site preparado para https://goncalvesl.github.io/crewsync/; estado de implantação
será registrado após verificar a execução remota. Workflows e fontes prontos.
Artefatos CI não equivalem a release publicada. ARM64 requer execução da matriz.

Faltam escolha da licença e matriz física ampliada/medições entre computadores.
Esses dados não foram inventados; docs/reports/physical-acceptance.json conserva
basic_pilot_accepted=true e extended_matrix_accepted=false. A automação impede
publicar uma tag -dev ou release sem esses aceites. Não há versão estável.

Limites: servidor serial para rede privada, kernel >=5.6 com /proc/filesystem local;
sem quotas totais, retomada parcial, compressão ou limpeza das cópias locais.
Não rebaixar servidor após gravar histórico v2. Backup independente permanece
necessário. Decisões e compatibilidade: 0005 e 0006.
