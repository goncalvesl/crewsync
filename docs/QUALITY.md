# Qualidade e simplicidade

Este guia orienta agentes e contribuidores. Aplicar os critérios à etapa em andamento, sem antecipar funcionalidades do roadmap. Documentação ajuda a manter decisões consistentes; a qualidade precisa ser demonstrada por revisão, testes e operação.

## Código e responsabilidades

- Preferir a solução mais simples que preserve integridade, segurança e recuperação. Não criar frameworks internos, plugins ou abstrações para necessidades hipotéticas.
- Manter funções e módulos com responsabilidades claras. Separar regras de sincronização de rede, disco e interface quando isso facilitar entendimento e testes; não criar camadas apenas por convenção.
- `protocol` contém contratos compartilhados e validações comuns, sem depender de cliente ou servidor. `server` controla autorização, revisões e persistência central; `client` controla estado local e reconciliação.
- Usar tipos para representar estados e identificadores quando isso impedir combinações inválidas. Evitar booleanos ambíguos, strings com múltiplos significados e duplicação de regras.
- Tratar erros esperados com `Result` e contexto útil. Evitar `unwrap`, `expect` e `panic!` nos caminhos de produção; exceções precisam de uma invariável clara e documentada. Nunca registrar tokens, códigos de ativação ou conteúdo de arquivos.
- Preferir Rust seguro. Qualquer `unsafe` exige necessidade concreta, justificativa das invariantes e validação específica.
- Adicionar dependências apenas para uma necessidade presente, considerando manutenção, licença e custo operacional. Não escolher bibliotecas ou estabilizar formatos apenas para preencher documentação.

## Integridade antes de desempenho

- O servidor central armazena o conteúdo e ordena as revisões. P2P, travessia de NAT e relays ficam fora da versão atual.
- Validar entradas de rede e disco. Autorizar cada operação no servidor; um caminho validado não substitui verificação de acesso nem proteção contra troca de links durante a operação.
- Publicar conteúdo somente após verificar seu hash. Definir a ordem de persistência de conteúdo e metadados, o momento da confirmação ao cliente e o procedimento de recuperação antes de implementar gravações.
- Não tratar renomeação atômica como garantia suficiente de durabilidade após queda. Documentar as garantias do sistema de arquivos e testar falhas nos pontos de publicação.
- Comparar a revisão-base antes de aceitar alterações e exclusões. Preservar conteúdo concorrente e manter registros de exclusão; não usar somente o relógio do computador para decidir qual versão vence.
- Limitar buffers, filas, concorrência e tentativas. Fazer transferências em streaming, permitir encerramento controlado e evitar repetição infinita de erros permanentes.
- Otimizar após medir com dados sintéticos representativos. O armazenamento inicial permanece raw, conforme PROJECT.md.

## Como entregar uma mudança

1. Inspecionar a implementação existente e definir o comportamento esperado e os erros relevantes.
2. Registrar uma decisão em `docs/decisions/` quando houver escolha duradoura de protocolo, persistência, segurança ou operação. Pequenas escolhas locais não precisam de registro.
3. Implementar uma parte pequena e verificável. Não misturar refatorações sem relação com a tarefa.
4. Testar comportamento observável e invariantes conforme TESTING.md. Priorizar concorrência, falhas e recuperação nos componentes que alteram dados.
5. Executar as verificações aplicáveis, revisar o diff e atualizar documentação, configuração e status afetados.

Uma mudança está pronta quando resolve o comportamento solicitado, passa nas verificações aplicáveis e tem suas limitações registradas. Enquanto não existir workspace, não declarar verificações Cargo como executadas. Evitar testes que apenas repitam a implementação e documentação que prometa recursos ainda ausentes.
