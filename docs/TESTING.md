# Política de testes automatizados

## Regra de conclusão

Toda funcionalidade deve incluir testes automatizados do comportamento e dos erros relevantes. Toda correção de bug deve incluir teste de regressão. Não considerar uma etapa pronta com testes ausentes, ignorados sem justificativa ou falhando. Verificações manuais complementam os testes, sem substituí-los.

Não há meta percentual de cobertura definida. Priorizar invariantes de integridade, autorização e recuperação; testes de funções isoladas não substituem integração e ponta a ponta.

## Camadas

- Unitários próximos dos módulos Rust: validação, permissões, revisões e regras de retenção.
- Integração em `crates/<crate>/tests/`: persistência, protocolo e comportamento entre componentes.
- Ponta a ponta em `tests/e2e/`: servidor e dois clientes em processos separados, com diretórios temporários e isolamento.
- Site e documentação: links, navegação, renderização em telas distintas, exemplos executáveis e pacotes de instalação, conforme forem implementados.

## Matriz mínima por funcionalidade

Cobrir hash incorreto, caminhos maliciosos, compartilhamentos isolados, leitura sem escrita, tokens inválidos/revogados, convites expirados/reutilizados, conflitos, exclusões após reconexão, diretórios vazios, arquivos mudando durante envio e ausência de ciclos.

Para persistência e operação, cobrir reinício após interrupção, transferências incompletas, configuração inválida, retenção, restauração, coleta de objetos ainda referenciados e disco cheio. Usar abstrações ou ambientes controlados para falhas determinísticas.

Testes não devem acessar dados reais, portas fixas ou serviços externos obrigatórios. Usar fixtures sintéticas, limites de tempo e espera por condições; evitar sleeps arbitrários. Testes devem funcionar em CI Linux desde a fundação do código.

## Comandos do workspace

Executar na raiz:

```bash
cargo build --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

O workflow `.github/workflows/ci.yml` executa build, fmt, Clippy e testes com
`--locked`. Também verifica dependências do sistema e roda os testes TLS e de rede.

Nesta entrega há 55 testes Rust locais (incluindo integração de autenticação,
worker em processo separado, cliente+armazenamento e falhas simuladas), além de
dois testes HTTPS marcados `ignored` por exigirem sockets. O motivo é explícito:
o ambiente de implementação retorna `Operation not permitted` ao criar sockets,
inclusive localhost. Não suprimir esses dois testes na validação real/CI:

```bash
cargo build --workspace --locked
cargo test -p crewsync-server --test pilot --locked -- --ignored
```

O runner `tests/e2e/pilot.rs` está ligado ao Cargo do servidor por `[[test]]`.
Ele executa servidor e dois clientes em processos separados, cria certificados
sintéticos, usa porta efêmera e verifica convergência, conflitos, exclusões,
rename/deduplicação, reinício, permissões, revogação, symlinks e recusa de TLS
incorreto. Os binários de ambos os pacotes precisam estar compilados no mesmo
perfil/target; por isso o build do workspace antecede o teste. A CI limita a etapa
a três minutos. `scripts/check-pilot.sh` executa a sequência completa, com timeout
na etapa de rede.

Os testes `reconciliation.rs` usam o reconciliador real e o armazenamento real
com transporte determinístico sem sockets, cobrindo também resposta perdida,
editar durante envio, corrupção de download, mudanças arquivo/diretório e
conflitos entre exclusão de diretório e edição concorrente de filhos. Não são
substitutos do teste HTTPS. `worker.rs` exercita framing e autenticação em um
processo servidor separado, inclusive EOF prematuro sem publicação.

TLS real pode ser testado sem sockets, via OpenSSL MemoryBIO:

```bash
/usr/bin/python3 -I tests/e2e/tls_memory.py
```

São três testes: CA confiável/hostname válido, CA não confiável e hostname
incorreto. O setup usa o gerador real `scripts/pilot-cert.sh`, validando também a
cadeia e permissões da chave. Esta verificação não testa TCP, firewall ou HTTP.

Pré-requisitos: `/usr/bin/sha256sum` (coreutils), `/usr/bin/python3` >= 3.11 com
`tomllib`/`ssl`, OpenSSL CLI. O cliente exige Linux >= 5.6 com `/proc` montado para
operações relativas a descritores com `openat2` e recusa de symlinks.

Falhas de publicação/disco são simuladas; não foi executado ensaio físico de
disco cheio, queda de energia ou piloto em duas máquinas nesta sessão. Não há
testes de retenção/histórico/restauração/coleta, pois ainda não estão implementados.

## Evidências

Registrar no status ou relatório da etapa os comandos executados, resultados, ambiente e limitações. Não afirmar que testes passaram quando não foram executados. Benchmarks devem indicar hardware, volume e tipos de arquivo, sem tratar tempo exato como teste determinístico.
