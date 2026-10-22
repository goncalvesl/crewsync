# Estado e retomada

## Implementado para o piloto

- Workspace Rust Linux, armazenamento raw por SHA-256, deduplicação, manifestos,
  revisões, tombstones, publicação durável e recuperação de temporários.
- TOML estrito, ACLs read/write, recarga inválida mantém última configuração válida.
- Convites temporários de uso único, token por dispositivo, listagem/revogação;
  somente hashes no servidor. Ativação persiste consumo e dispositivo juntos.
- HTTPS obrigatório com validação de CA/hostname, worker autenticado e autorizado
  em cada operação. Recarga de configuração antes de cada requisição.
- Cliente `activate`, `shares`, `sync` e `watch`, baseline por revisão, snapshots
  de upload, downloads verificados, diretórios vazios, exclusões e rename como
  exclusão/criação. Conflitos locais preservados fora da árvore sincronizada.
- Proteção de caminhos por descritores/openat2 no cliente; arquivos especiais e
  links recusados; bloqueio por pasta, temporários e cópias em `.crewsync/`.
- Gerador de certificados, exemplos systemd e procedimento de backup. Roteiro
  completo: [PILOT.md](PILOT.md); decisão [0004-pilot.md](decisions/0004-pilot.md).
- Binários otimizados compilados em `target/release/crewsync-server` e
  `target/release/crewsync-client`. Não há release pública ou licença atribuída.

## Validação realizada e bloqueio de rede

Linux, Rust/Cargo 1.99.1, Python 3.14.8 e OpenSSL. Executados com sucesso:

- `cargo build --workspace --locked --offline`
- `cargo test --workspace --locked --offline`: **55 passaram, 2 ignorados**.
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --locked --offline -- -D warnings`
- `/usr/bin/python3 -I tests/e2e/tls_memory.py`: **3 passaram**, incluindo gerador
  real de certificados, handshake válido e rejeição de CA/hostname incorretos.
- `cargo build --release --workspace --locked --offline`
- `target/release/crewsync-server check-config config/server.example.toml`
- `target/release/crewsync-client --version`
- `git diff --check`

Inicialmente o sandbox proibia sockets e escrita em `.git`. Após o usuário
atualizar a sessão para acesso completo, foi executado com sucesso:

- `cargo test -p crewsync-server --test pilot --locked --offline -- --ignored`:
  **2 testes HTTPS passaram**, com servidor e dois clientes em processos separados.
  Cobrem convergência, conflitos, exclusões offline, reinício, permissões,
  revogação, symlinks e rejeição de TLS incorreto.

Os testes permanecem `ignored` por padrão para suportar ambientes restritos;
a CI e `scripts/check-pilot.sh` executam-nos explicitamente. Não houve teste em
duas máquinas físicas nem execução da CI do GitHub nesta sessão.

## Próxima ação concreta

Nas máquinas do piloto, executar `scripts/check-pilot.sh` e seguir [PILOT.md](PILOT.md) com dois Linux e dados sintéticos: máquina 1
servidor+cliente, máquina 2 cliente. Registrar distribuição, kernel, cenários,
volume de dados e resultados. Corrigir qualquer falha do teste TCP antes de usar
arquivos reais. Não considerar aceites de rede/piloto encerrados antes disso.

## Limites e pendências

- Sem crates HTTP/TLS/TOML disponíveis no cache e sem DNS para crates.io: ponte
  HTTPS e parser TOML temporários usam Python padrão embarcado, sem shell.
  SHA-256 usa `/usr/bin/sha256sum`. Migrar para bibliotecas Rust validadas depois.
- Cliente exige Linux >= 5.6 com `/proc`, Python >= 3.11 com ssl/tomllib, coreutils;
  OpenSSL CLI para certificados/testes. Servidor serializado para rede privada,
  sem promessa de endurecimento HTTP público. Sem observador de eventos: watch
  usa reconciliação periódica com atraso limitado em falhas transitórias.
- Raiz interna privada 0700, sem modificações externas durante operações. Snapshot
  JSON reescrito e objetos verificados; desempenho não medido. Backups requerem
  processos parados. Nenhum ensaio real de disco cheio/queda de energia foi feito.
- Etapa 5 incompleta: sem histórico consultável, restauração/retencão configurável,
  quotas ou coleta. Cópias locais e objetos órfãos são mantidos sem limpeza
  automática; acompanhar espaço livre. Histórico não substitui backup.
- Ainda pendentes: medições/piloto físico (etapa 6), site, distribuição, licença,
  definição de arquiteturas Linux e suporte Windows/Android. Alterações anteriores
  do workspace preservadas; não houve publicação nem alteração real de firewall.
