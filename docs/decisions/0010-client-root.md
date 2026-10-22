# 0010 — Raiz local de compartilhamentos

O mantenedor pediu que o cliente pergunte somente a raiz local e crie dentro
dela todas as pastas autorizadas. A lista vem da operação Shares já existente,
sem mudança no protocolo, e é consultada a cada ciclo de reconciliação.

O assistente usa sync-root/watch-root. Os comandos individuais permanecem para
compatibilidade. Cada compartilhamento mantém seu próprio estado, conflitos e
lixeira; arquivos avulsos na raiz não são enviados. A raiz é vinculada ao servidor
e bloqueada durante o ciclo; falha em uma pasta não impede tentar as demais.
A sincronização é serial para limitar recursos. A remoção de acesso preserva a
cópia local, sem sincronizá-la. Erros permanentes encerram o serviço; falhas
transitórias seguem o backoff existente.

A criação das pastas usa descritores ancorados e recusa links. O cliente valida
todos os nomes recebidos antes de criar subpastas. Layouts antigos não são
convertidos automaticamente: migração deve ser explícita, com serviço parado e
preservando os estados individuais. O assistente público só terá esse comportamento
quando uma release com os novos binários for publicada.
