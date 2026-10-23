# Decisão 0014: deploy Docker pelo Coolify

## Contexto

É necessário disponibilizar uma instalação por Docker Compose em um host Linux
com Coolify, mantendo a porta de aplicação privada e os dados fora da camada
efêmera do container.

## Decisão

O repositório contém Dockerfile multi-stage e `compose.yaml`, e o workflow de
release publica a imagem multiarch `ghcr.io/goncalvesl/crewsync-server` no GHCR.
O Coolify pode usar Docker Image sem compilar o código e termina TLS pelo proxy.
O serviço escuta HTTP na porta interna 8080, sem publicar portas no host. A configuração usa volume
Docker persistente; o `storage_root` usa bind mount configurável por
`CREWSYNC_DATA_DIR`, default `/opt/crewsync/data`, pertencente ao UID/GID 10001 e
modo 0700. O container roda sem root, com filesystem de raiz somente leitura,
capabilities removidas e uma réplica única.

O primeiro bootstrap contém somente a raiz de armazenamento. `CREWSYNC_PUBLIC_URL`
e `CREWSYNC_ADMIN_PASSWORD` são obrigatórias no Compose. A senha é um segredo
hexadecimal aleatório de 256 bits; no primeiro boot persiste-se somente seu hash.
O bootstrap é idempotente: reinícios e alterações posteriores da variável não
rotacionam a credencial. A tag numerada da imagem fixa a versão; a tag `alpha`
aponta para a release alpha mais recente. Usuários, pastas e permissões são
criados pelo painel.

## Consequências

O diretório de dados reside no filesystem do host e pode ser escolhido conforme
disco e política de backup. Atualizar ou recriar o container mantém os dados e a
configuração. O operador precisa preparar o diretório com as permissões exigidas,
fazer backup dos dois destinos persistentes e confiar no proxy TLS do Coolify.
A imagem é publicada para amd64 e arm64 na release junto com os pacotes de sistema.
