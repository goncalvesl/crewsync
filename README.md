# CrewSync

Sincronizador open-source em Rust, com servidor próprio, permissões por usuário e armazenamento de conteúdo sem compactação (`raw`). Primeira versão somente Linux, para piloto em dois computadores.

## Comece por aqui

1. Leia [AGENTS.md](AGENTS.md), o guia original de contribuição.
2. Leia [docs/PROJECT.md](docs/PROJECT.md), com as decisões atuais que substituem propostas antigas do guia.
3. Consulte [docs/STATUS.md](docs/STATUS.md) para retomar o trabalho.
4. Siga [docs/PLAN.md](docs/PLAN.md) e [docs/TESTING.md](docs/TESTING.md).
5. Atualize a documentação conforme [docs/WORKFLOW.md](docs/WORKFLOW.md).
6. Aplique os critérios de simplicidade e revisão de [docs/QUALITY.md](docs/QUALITY.md).

## Estado atual

Piloto Linux implementado: servidor HTTPS, convites de uso único, tokens por dispositivo,
permissões read/write e cliente com sincronização manual/periódica, conflitos preservados
e exclusões persistentes. Siga o [roteiro para dois computadores](docs/PILOT.md).

Passaram os testes locais, TLS em memória e os dois testes HTTPS com servidor e
clientes em processos separados, após liberar as permissões da sessão. A validação
em duas máquinas físicas ainda está pendente. Histórico/retencão completos, site e releases continuam pendentes.
Não há download publicado ou licença atribuída.

## Organização

- `crates/server/`: servidor e administração.
- `crates/client/`: cliente Linux e sincronização local.
- `crates/protocol/`: contratos compartilhados de comunicação.
- `config/`: exemplo TOML do servidor, sem segredos.
- `tests/e2e/` e `tests/fixtures/`: testes de ponta a ponta e dados sintéticos.
- `docs/`: plano, decisões, progresso, testes e futuro manual.
- `site/`: futuro site estático no GitHub Pages.

Consulte [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) para instalar as ferramentas, compilar, testar e executar a fundação.
