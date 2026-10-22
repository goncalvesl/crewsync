# Decisões atuais do projeto

Este documento registra o escopo mais recente acordado. O `AGENTS.md` original permanece preservado; suas referências a Windows inicial e compactação não representam o escopo atual.

## Requisitos definidos

- Implementação em Rust, simples de operar, rápida e com integridade dos arquivos.
- Servidor central e clientes; piloto inicial Linux em dois computadores, com aproximadamente cinco usuários no uso previsto.
- Windows depois; Android no futuro, com acesso online sem exigir operação offline.
- Apenas pastas de primeiro nível compartilhadas; subpastas herdam permissões.
- Permissões por usuário em arquivo no servidor: leitura ou leitura e escrita.
- Sem banco de dados externo; estado persistido em arquivos internos.
- Armazenamento `raw`, sem compactação. Não implementar armazenamento compactado.
- Projeto aberto no GitHub; site de apresentação e manual no GitHub Pages, sem domínio próprio; downloads no GitHub Releases.
- Toda funcionalidade deve ter testes automatizados e documentação correspondente.

## Arquitetura proposta para implementação

Conteúdo identificado por hash e manifestos por compartilhamento, inspirados no Git, sem branches ou commits manuais. Tokens individuais por dispositivo vinculados ao usuário, ativados por códigos temporários de uso único; servidor guarda hashes dos tokens. HTTPS, autorização por operação, revogação individual e negação de acesso por padrão.

Cliente inicial por CLI e serviço em segundo plano. Observação de arquivos com reconciliação periódica, streaming, limites de recursos, temporários e publicação atômica. Revisões ordenadas pelo servidor, preservação de conflitos e registros de exclusão. Links simbólicos e arquivos especiais não serão sincronizados inicialmente; caminhos não podem escapar do compartilhamento.

Histórico com versões completas e deduplicação por hash. Proposta de retenção: até 30 dias e no máximo 10 versões anteriores por arquivo; lixeira de 30 dias. A versão atual permanece. Restauração cria nova revisão. Limpeza só remove objetos sem referências e precisa respeitar transferências em andamento. Histórico não substitui backup.

## Decisões ainda abertas

Nome público, licença, bibliotecas, algoritmo de hash, formatos persistentes, protocolo, estratégia de TLS, distribuições/arquiteturas Linux suportadas e política exata de retenção. Documentar escolhas antes de estabilizar contratos. Compactação na transmissão é apenas uma otimização futura a avaliar; não é requisito da primeira versão.

## Limites de hospedagem

GitHub Pages hospeda apenas o site estático, em `https://USUARIO.github.io/REPOSITORIO/`. Documentos privados e o serviço de sincronização ficam na infraestrutura do operador. Nunca publicar dados reais, credenciais ou configurações privadas.
