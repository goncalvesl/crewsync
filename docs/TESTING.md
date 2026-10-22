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

Nesta entrega há 72 testes Rust locais (incluindo integração de autenticação,
worker em processo separado, cliente+armazenamento e falhas simuladas), além de
cinco testes HTTPS e um ensaio ENOSPC marcados `ignored` por exigirem
recursos externos (TCP e tmpfs isolado). O motivo é explícito:
o ambiente inicial proibia sockets; a sessão atual permite rede e os testes são
executados explicitamente. Não suprimir esses testes na validação real/CI:

```bash
cargo build --workspace --locked
cargo test -p crewsync-server --test pilot --locked -- --ignored
```

O runner `tests/e2e/pilot.rs` está ligado ao Cargo do servidor por `[[test]]`.
Ele executa servidor e dois clientes em processos separados, cria certificados
sintéticos, usa porta efêmera e verifica convergência, conflitos, exclusões,
rename/deduplicação, reinício, permissões, revogação, symlinks e recusa de TLS
incorreto. O terceiro cenário cobre diagnósticos de inicialização: TLS ausente ou
inválido, porta/armazenamento em uso, raiz 0755 e configuração inválida sem ecoar
seus valores. Os binários de ambos os pacotes precisam estar compilados no mesmo
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

A ativação tem regressões para confirmar com Enter mantendo stdin aberto, aceitar
EOF em pipes/CRLF, limitar a entrada e não imprimir o convite.

Runtime: Linux >=5.6 com /proc e filesystem local; binários nativos sem Python.
Testes auxiliares/build do site/pacotes: Python padrão e OpenSSL CLI. O quinto
cenário HTTPS interrompe upload, confirma recuperação do worker e ausência de
conteúdo parcial publicado. Retenção/coleta têm relógio controlado, simulação,
prazo de graça, lixeira, dados atuais preservados e falhas de publicação. Backup
cobre locks, instalação em destino novo, preservação de tokens e marcador ausente.
notify tem teste real de eventos e exclusão da área interna. HTTP recusa headers
ambíguos, duplicados, chunked e cabeçalhos acima do limite.

Site: `python3 scripts/build-site.py` e `python3 tests/site/check.py target/site`.
Navegador: tests/site/browser.cjs, 390/768/1440px, console, overflow, links e âncoras.
Pacotes: scripts/check-package.py valida checksum, extração segura, instalação em
prefixo temporário e comandos básicos. Nenhum teste acessa dados reais.

Benchmark: `python3 scripts/benchmark.py --report target/benchmark.json`.
Relatório contém wall/CPU/ru_maxrss por processo cliente e bytes de conteúdo, não
tráfego no fio. ru_maxrss inclui o pico inicial herdado do harness no fork;
comparar só ensaios com a mesma metodologia. Não representa latência entre máquinas.
ENOSPC real passou em tmpfs Docker limitado a 1 MiB, preservando arquivo confirmado
e limpando o upload incompleto. Execute `scripts/check-disk-full.sh DEPS_MUSL`.
Falhas físicas de energia e matriz ampliada em duas máquinas permanecem
pendentes, conforme docs/reports/physical-acceptance.json.

## Evidências

Registrar no status ou relatório da etapa os comandos executados, resultados, ambiente e limitações. Não afirmar que testes passaram quando não foram executados. Benchmarks devem indicar hardware, volume e tipos de arquivo, sem tratar tempo exato como teste determinístico.
