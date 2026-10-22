# 0005 — Histórico por caminho e restauração individual

> Atualização: dependências temporárias e operação foram substituídas pela [decisão 0006](0006-operation-native-distribution.md). Este registro preserva o contexto histórico.
O piloto básico em dois computadores foi concluído pelo usuário. A primeira
entrega da etapa 5 acrescenta recuperação, antes de implementar retenção/coleta.

Cada caminho possui `history/<share>/<sha256-do-caminho>`: JSON formato 1 com
`path` e `entries` (revisão e conteúdo). Mantemos somente metadados anteriores;
os bytes continuam deduplicados em `objects`. O manifesto atual continua formato
1, compatível com o piloto. Não há migração destrutiva: a primeira alteração
arquiva a versão atual existente; versões perdidas antes desta atualização não
podem ser reconstruídas apenas pelos objetos órfãos.

Ordem de publicação: objeto verificado e durável; novo manifesto validado e
limitado; histórico anterior escrito em temporário 0600, fsync, rename e fsync
do diretório; depois publicação durável do novo manifesto. O lock exclusivo e o
worker serial impedem mutações concorrentes. Se a operação falhar após arquivar,
a versão ainda atual pode aparecer também no arquivo de histórico; a consulta
a deduplica pela revisão. Temporários incompletos são removidos na abertura.
A abertura valida histórico e integridade dos objetos históricos; corrupção
recusa o serviço, sem descartar versões silenciosamente. As garantias de fsync
e as limitações físicas são as mesmas da decisão 0002.

Consulta exige read; restauração exige write atual e identidade autenticada.
Ambas usam compartilhamento e caminho, nunca hash arbitrário. Restaurar exige
revisão de origem e revisão-base atual explícitas; publica uma nova revisão,
sem mover o contador para trás. Não aceita tombstone como origem. Restaurar
diretório cria somente o diretório; pais devem existir, filhos são restaurados
individualmente. Conflitos seguem as regras existentes de sincronização.

Histórico é limitado a 16 MiB por caminho (reserva de 1024 bytes antes de aceitar
nova alteração); exceder o limite recusa a mutação, preservando os dados atuais.
Consulta revalida os objetos históricos. Não há timestamps, paginação, prazo de
retenção, limpeza automática ou GC nesta entrega. Isso mantém a recuperação
revisável antes de introduzir remoção de dados; consumo de espaço deve ser
acompanhado pelo operador. Clientes antigos continuam sincronizando, mas exigem
atualização para usar os novos comandos. Não rebaixar o servidor depois de gravar
histórico: binários antigos não mantêm esses metadados. Backup inclui toda a raiz
com o serviço parado. Histórico não substitui backup.
