# CrewSync 0.1.0-alpha.10

A instalação do servidor no Coolify agora pode usar uma imagem pré-compilada e
pública do GHCR: `ghcr.io/goncalvesl/crewsync-server:0.1.0-alpha.10`. A imagem
oferece `linux/amd64` e `linux/arm64`; também existe a tag móvel `alpha`. Não é
necessário compilar Rust no Coolify. O deploy direto do código com Docker Compose
continua disponível.

No primeiro boot, `CREWSYNC_ADMIN_PASSWORD` e `CREWSYNC_PUBLIC_URL` inicializam a
credencial do painel automaticamente. Use um segredo hexadecimal aleatório de 64
caracteres (`openssl rand -hex 32`). O servidor armazena somente o hash e não
substitui a credencial em reinícios. A configuração inicial continua vazia:
cadastre usuários, pastas e permissões no painel e crie convites por dispositivo.

A imagem atende HTTP na porta interna 8080; use o proxy do Coolify para HTTPS e
monte persistentemente `/etc/crewsync` e `/var/lib/crewsync`. A documentação traz
as permissões exigidas para o bind mount de dados, o domínio e o backup.

Esta versão é alpha para testes. A instalação de Docker Image foi testada com o
smoke Docker Compose e os testes locais do workspace; o deploy real no Coolify e
a matriz física ampliada continuam pendentes. Os pacotes Arch, Debian, Fedora e
tar.gz permanecem publicados junto com seus checksums e código-fonte.
