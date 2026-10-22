# Repository Guidelines

## Leitura antes de implementar

Consultar `docs/PROJECT.md` para o escopo atual, `docs/STATUS.md` para o estado real e `docs/QUALITY.md` para os critérios de implementação. As propostas históricas abaixo devem ser lidas junto dessas decisões; divergências com instruções explícitas do usuário devem ser resolvidas em favor do usuário. Seguir o fluxo de `docs/WORKFLOW.md` e as verificações de `docs/TESTING.md`.

## Objetivo e escopo

Desenvolver em Rust um sincronizador de pastas para uma construtora com aproximadamente cinco usuários. Priorizar simplicidade para instalar, manter e usar, velocidade e integridade dos arquivos. Minimizar passos e dependências de instalação. Manter a documentação do usuário simples, com detalhes técnicos em uma seção final opcional. Licença definida: GPL-3.0-only. Os clientes iniciais serão Windows e Linux; Android fica para uma etapa futura, com acesso online, sem exigir sincronização offline no celular.

## Arquitetura e organização

O repositório possui uma fundação Cargo; consultar `docs/STATUS.md` para as funcionalidades efetivamente implementadas. A arquitetura proposta é cliente-servidor centralizada, inspirada no Git para identificação de conteúdo por hash, sem expor commits ou branches ao usuário. Evitar banco de dados externo; persistir índices, revisões e exclusões em arquivos internos com recuperação após falhas.

Manter o workspace Cargo com `crates/server/`, `crates/client/` e `crates/protocol/`. Colocar testes de integração em `tests/` de cada crate e exemplos de configuração sem segredos em `config/`. Não assumir que diretórios reservados já possuem funcionalidades.

## Compartilhamento e autenticação

- Compartilhar apenas pastas de primeiro nível; subpastas herdam o acesso.
- Configurar usuários e permissões no servidor em TOML. Proposta: `read` permite leitura; `write` inclui leitura, alteração e exclusão. Negar acesso por padrão.
- Adotar como proposta códigos de ativação temporários e de uso único, gerando um token por dispositivo vinculado ao usuário. Persistir somente hashes dos tokens no servidor e permitir revogação individual.
- Exigir HTTPS e validar autorização e caminhos em toda operação, impedindo escapes da pasta compartilhada, inclusive por links simbólicos.

## Sincronização e desempenho

Preservar versões conflitantes; nunca sobrescrever silenciosamente alterações concorrentes. Registrar exclusões para evitar ressurreição por clientes antigos. Usar temporários e publicação atômica, verificando o hash do conteúdo recebido. Combinar observação do sistema de arquivos com reconciliação periódica.

A proposta de compactação é Zstandard seletivo em streaming, inicialmente nível 1. Medir com arquivos reais antes de otimizar. Limitar memória e concorrência; evitar carregar arquivos inteiros na RAM.

## Desenvolvimento e testes

Após criar o workspace, executar `cargo build`, `cargo test --workspace`, `cargo fmt --all -- --check` e `cargo clippy --workspace --all-targets -- -D warnings`. Usar formatação padrão Rust, `snake_case` para funções e módulos e `PascalCase` para tipos.

Testar permissões, revogação, conflitos, exclusões, transferências interrompidas e recuperação após falhas em Windows e Linux. Não há meta de cobertura definida.

## Commits e revisão

Sem histórico Git disponível para inferir convenções. Usar mensagens curtas e descritivas. PRs devem explicar comportamento, validação e mudanças de configuração; nunca incluir credenciais ou arquivos reais da construtora.
