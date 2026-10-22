# 0008 — Publicação alpha solicitada e documentação sincronizada

O mantenedor pediu expressamente uma release 0.x e downloads da última versão
para cada arquitetura no manual, além de registrar a tarefa para repeti-la.
Esta instrução substitui o bloqueio anterior de qualquer release sem piloto
ampliado: alpha pode ser pública para teste, claramente identificada. O aceite
físico permanece falso e continua obrigatório para versão estável.

Primeira versão preparada: 0.1.0-alpha.1. docs/release.json é a referência da versão
atual; prepare-release.py atualiza metadados e blocos de downloads juntos. Links
versionados são necessários porque GitHub latest não seleciona alpha. A tag é
enviada antes de main, permitindo publicar/verificar os assets antes de expor os
novos links em Pages. Não reescrever tags/assets publicados.

Packages compila e testa x86_64/ARM64 musl, gera checksums e publica após verificar
os gates. Fonte correspondente e dependências vendorizadas são oferecidas ao lado
dos binários; a compilação offline com cache vazio valida que estão utilizáveis.
Documentação do procedimento em RELEASING.md e instrução permanente em AGENTS.md.

A publicação atual não declara estabilidade, ensaios físicos concluídos ou
resultados de desempenho adicionais. Licença permanece GPL-3.0-only.
