# 0001 — Fundação do workspace

Status: adotado para a fundação; escolhas locais de implementação, revisáveis.

## Contexto

Precisamos de uma base verificável antes de implementar persistência e rede. O produto segue cliente-servidor centralizado, sem P2P, com armazenamento raw e piloto Linux.

## Decisão

- Workspace Cargo com três pacotes: `crewsync-server`, `crewsync-client` e `crewsync-protocol`. Nomes alinhados à marca CrewSync, definida pelo mantenedor.
- Rust edição 2024 e toolchain stable. Não declaramos MSRV ainda. Registrar a versão efetivamente testada a cada entrega; adotar versão mínima quando houver requisitos de distribuição.
- CI inicial em Ubuntu 24.04 x86_64. Esse alvo é de validação, não uma promessa de compatibilidade de binários com todas as distribuições. Outras arquiteturas e distribuições precisam de testes próprios.
- Sem dependências externas nesta etapa. Acrescentar bibliotecas conforme as próximas tarefas, com justificativa.
- Proibir código unsafe nas crates do workspace. Pacotes não publicáveis no crates.io enquanto a licença está pendente. Versionar Cargo.lock.
- CLI inicial limitada a `--help` e `--version`. Sem argumentos ou com argumentos não suportados, encerrar com código 2. Nenhum serviço é iniciado nesta etapa.
- Manter `protocol` sem contratos fictícios. Introduzir tipos e dependências entre crates quando houver comportamento compartilhado real.

## Alternativas e consequências

Um pacote único seria menor inicialmente, mas misturaria as responsabilidades futuras já previstas. Mais crates ou camadas acrescentariam organização sem necessidade presente.

Não escolher agora HTTP, runtime assíncrono, hash, TLS e formato persistente evita estabilizar contratos sem implementar seus requisitos. Essas decisões devem anteceder os respectivos componentes. Não há configuração carregável; exemplos TOML serão entregues e testados junto ao parser.

Nome público definido pelo mantenedor: CrewSync. A licença permanece pendente. Nenhuma licença foi atribuída nesta entrega.

## Validação

Build, formatação, Clippy com warnings tratados como erros e testes de integração dos dois executáveis. A CI repete essas verificações; testes de sincronização, persistência e rede pertencem às próximas etapas.
