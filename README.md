# file-sync

Sincronizador open-source em Rust, com servidor próprio, permissões por usuário e armazenamento de conteúdo sem compactação (`raw`). Primeira versão somente Linux, para piloto em dois computadores.

## Comece por aqui

1. Leia [AGENTS.md](AGENTS.md), o guia original de contribuição.
2. Leia [docs/PROJECT.md](docs/PROJECT.md), com as decisões atuais que substituem propostas antigas do guia.
3. Consulte [docs/STATUS.md](docs/STATUS.md) para retomar o trabalho.
4. Siga [docs/PLAN.md](docs/PLAN.md) e [docs/TESTING.md](docs/TESTING.md).
5. Atualize a documentação conforme [docs/WORKFLOW.md](docs/WORKFLOW.md).

## Estado atual

Planejamento e estrutura de diretórios preparados. Não há código Rust, workspace Cargo, testes executáveis, CI ou site implementados. Não há versão disponível para download. Nome público e licença ainda precisam ser definidos.

## Organização

- `crates/server/`: servidor e administração.
- `crates/client/`: cliente Linux e sincronização local.
- `crates/protocol/`: contratos compartilhados de comunicação.
- `config/`: futuros exemplos sem segredos.
- `tests/e2e/` e `tests/fixtures/`: testes de ponta a ponta e dados sintéticos.
- `docs/`: plano, decisões, progresso, testes e futuro manual.
- `site/`: futuro site estático no GitHub Pages.

Os comandos Cargo documentados são metas para a criação do workspace; ainda não podem ser executados neste estado.
