# Histórico e recuperação

Atualize servidor e cliente antes de testar. Pare o servidor, faça backup de toda
a raiz interna, compile com `cargo build --release --workspace --locked`, instale
os dois binários conforme PILOT.md e reinicie o serviço. Configuração e sessões
existentes continuam válidas. Não rebaixe o servidor após essa atualização.

A consulta lista revisões disponíveis de um caminho relativo ao compartilhamento,
em ordem crescente; a última linha é o estado atual, inclusive `excluído`:

```bash
crewsync-client history "$HOME/crewsync-pilot/session.json" Obras "planta.txt"
```

Exemplo sintético: revisão 1 = arquivo original, 2 = arquivo alterado, 3 = excluído.
Para recuperar a revisão 1, usando a revisão 3 como base atual:

```bash
crewsync-client restore "$HOME/crewsync-pilot/session.json" Obras "planta.txt" 1 3
crewsync-client sync "$HOME/crewsync-pilot/session.json" Obras "$HOME/crewsync-pilot/Obras"
```

Use o caminho real da sua sessão e pasta local. A recuperação acontece primeiro
no servidor como uma nova revisão. Execute `sync` em cada computador ou aguarde
`watch`. Mudanças locais concorrentes são preservadas pelas regras normais de
conflito. Se houver conflito de revisão na restauração, consulte novamente e
revise a alteração concorrente antes de repetir com a nova base.

Quem tem `read` consulta; somente `write` restaura. Não é possível restaurar uma
revisão de exclusão: escolha uma versão de arquivo ou diretório anterior. Para
recuperar uma árvore excluída, restaure os diretórios pais primeiro e os arquivos
depois; a operação é individual, sem restauração recursiva.

A versão atual do piloto anterior é preservada na próxima alteração. Versões
anteriores à atualização que já foram sobrescritas não aparecem no histórico.
Os nomes e revisões acompanham o caminho: rename continua exclusão/criação;
consulte o nome antigo para encontrar as versões antigas.

A retenção e coleta offline estão disponíveis por `crewsync-server maintain CONFIG [--apply]`, conforme [MANUAL.md](MANUAL.md#manutencao). Não há coleta agendada automaticamente. O limite de metadados por caminho
é 16 MiB; quando atingido, novas alterações são recusadas. Não apague objetos ou
históricos manualmente. Acompanhe espaço livre e mantenha backup da raiz completa
com o servidor parado. Lixeira e conflitos locais continuam em `.crewsync/`.

Teste sugerido com dados sintéticos nos dois computadores: sincronize um arquivo,
altere e sincronize, exclua e sincronize; consulte o histórico e restaure o
original. Confira o conteúdo nos dois clientes e que uma base antiga ou usuário
somente leitura não consegue restaurar. Testes automatizados cobrem também
reinício, corrupção, falhas de publicação e deduplicação.
