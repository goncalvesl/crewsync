# Decisão 0014: deploy Docker pelo Coolify

## Contexto

É necessário disponibilizar uma instalação por Docker Compose em um host Linux
com Coolify, mantendo a porta de aplicação privada e os dados fora da camada
efêmera do container.

## Decisão

O repositório contém Dockerfile multi-stage e `compose.yaml`. O Coolify compila o
servidor do código-fonte público e termina TLS pelo proxy. O serviço escuta HTTP
na porta interna 8080, sem publicar portas no host. A configuração usa volume
Docker persistente; o `storage_root` usa bind mount configurável por
`CREWSYNC_DATA_DIR`, default `/opt/crewsync/data`, pertencente ao UID/GID 10001 e
modo 0700. O container roda sem root, com filesystem de raiz somente leitura,
capabilities removidas e uma réplica única.

O primeiro bootstrap contém somente a raiz de armazenamento. Usuários, pastas e
permissões são criados pelo painel administrativo após gerar a chave inicial.

## Consequências

O diretório de dados reside no filesystem do host e pode ser escolhido conforme
disco e política de backup. Atualizar ou recriar o container mantém os dados e a
configuração. O operador precisa preparar o diretório com as permissões exigidas,
fazer backup dos dois destinos persistentes e confiar no proxy TLS do Coolify.
Distribuição de imagem pré-compilada em GHCR fica para um workflow futuro; Coolify
constrói diretamente do repositório público.
