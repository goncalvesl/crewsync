# Fluxo de trabalho e documentação

## Ao retomar com o Codex

Ler README.md, AGENTS.md, PROJECT.md, STATUS.md, PLAN.md e TESTING.md. Respeitar as decisões mais recentes em PROJECT.md quando divergirem das propostas antigas do guia original. Inspecionar os arquivos atuais antes de assumir que uma etapa está implementada.

## Durante a implementação

- Trabalhar na etapa solicitada, mantendo alterações revisáveis.
- Documentar configuração, comandos, erros, limitações e comportamento público junto ao código.
- Registrar decisões relevantes em `docs/decisions/`, como `0001-storage-format.md`, explicando contexto, escolha, alternativas e consequências.
- Não inventar recursos, resultados de testes ou comandos ainda inexistentes.
- Manter exemplos sintéticos e sem credenciais.

## Antes de encerrar uma etapa

1. Executar os testes e verificações aplicáveis de TESTING.md.
2. Atualizar o manual e os exemplos afetados.
3. Atualizar STATUS.md com entregas, decisões, pendências e próxima ação concreta.
4. Registrar comandos executados e resultados; explicar verificações não realizadas.
5. Atualizar o plano caso o escopo mude e registrar a motivação.

O status deve ser curto e suficiente para outra sessão continuar sem reconstruir toda a conversa. A documentação é parte da entrega, não uma tarefa opcional posterior.
