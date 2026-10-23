# Pacotes e publicação

## Construir e testar

Uso diário dos pacotes dispensa Rust, Cargo e Python. A construção exige Rust
stable, compilador C, musl-tools para targets musl; scripts de build usam Python
padrão. Os targets de distribuição são x86_64-unknown-linux-musl e
aarch64-unknown-linux-musl, com Linux kernel >=5.6 e `/proc`.

```bash
rustup target add x86_64-unknown-linux-musl
cargo build --release --workspace --locked --target x86_64-unknown-linux-musl
python3 scripts/package.py --bin-dir target/x86_64-unknown-linux-musl/release --target x86_64-unknown-linux-musl
python3 scripts/check-package.py target/packages/*.tar.gz
```

O tar.gz contém binários, install.sh e setup.sh, exemplos públicos de configuração/systemd,
gerador de certificados e documentação. Nenhum dado privado é coletado da raiz
do piloto. Inclui checksum SHA-256 e avisos das dependências. PREFIX controla a
instalação; default ~/.local. `./install.sh --client` instala somente o cliente,
`--server` instala servidor e auxiliares; sem opção, instala ambos. Manual e GPL
ficam em share/crewsync. install.sh não inicia serviços. setup.sh server|client
é o assistente guiado que configura e inicia; setup-online.sh baixa a última
release e valida o checksum antes de chamar esse assistente. Não há sudo ou
ajustes de PATH. Serviços de usuário systemd, ou nohup quando indisponível.
O tar.gz também pode ser instalado como serviço de sistema por `install-system.sh`:
ele coloca os arquivos estáticos em `/opt/crewsync`, configurações em
`/etc/opt/crewsync` e dados em `/var/opt/crewsync`. `install.sh` continua adequado
para cliente ou instalação manual por usuário.

## Servidor em pacote de sistema

Cada release Linux publica `.pkg.tar.zst` para Arch x86_64 e `.deb`, `.rpm` e
tar.gz upstream para x86_64 e ARM64. O pacote Arch é construído com `makepkg`;
Arch Linux ARM não está incluído nesta versão. Os pacotes da distribuição instalam
em `/usr/bin`, `/usr/share/crewsync`, `/etc/crewsync` e
`/var/lib/crewsync`. O daemon roda como usuário de sistema `crewsync`, sem login.
As raízes de armazenamento são privadas (`0700`) e pertencem à conta de serviço.
A instalação do pacote não inicia nem habilita o serviço antes da configuração.
O diretório da configuração é root-owned e gravável pelo grupo do serviço para
permitir que o painel publique mudanças atomicamente; o subdiretório TLS é root-owned
e sem escrita pelo daemon.

Instale o arquivo correspondente (`pacman -U`, `apt install ./arquivo.deb` ou
`dnf install ./arquivo.rpm`) e execute:

```sh
sudo crewsync-server-setup
```

O assistente prepara a raiz de dados, configura HTTPS direto com CA privada ou
backend HTTP em loopback para um proxy HTTPS, habilita `crewsync-server.service`
e mostra a chave administrativa. Não pede usuário nem nome de compartilhamento:
cadastre ambos no painel depois de iniciar o servidor e gere os convites por lá.
A alpha.10 inclui esse fluxo nos pacotes de sistema e no tar.gz. O deploy Docker
Image e o Compose também iniciam sem usuários e compartilhamentos.
No tar.gz, extraia o arquivo e execute `sudo ./install-system.sh` na pasta
extraída. A instalação por tar.gz precisa de systemd e OpenSSL; pacotes nativos
declaram essas dependências para o gerenciador.

Atualizações preservam configuração e dados e reiniciam o daemon para carregar o
binário novo. O instalador tar.gz para o serviço antes de substituir executáveis.
Remover o pacote não apaga
`/etc/crewsync` nem `/var/lib/crewsync`; faça backup antes de atualizar. O tar.gz
não remove automaticamente configuração ou dados. Os artefatos são arquivos
instaláveis diretamente, com checksum SHA-256; ainda não há repositórios apt,
pacman ou dnf.

## Docker e Coolify

A release publica a imagem OCI em `ghcr.io/goncalvesl/crewsync-server`, para
`linux/amd64` e `linux/arm64`. O pacote do GHCR é público e pode ser baixado sem
login. No Coolify, o caminho mais direto é criar um recurso **Docker Image** e
informar a imagem versionada:

```text
ghcr.io/goncalvesl/crewsync-server:0.1.0-alpha.10
```

Antes do deploy, prepare no host Linux o diretório de dados persistente:

```sh
sudo install -d -o 10001 -g 10001 -m 0700 /opt/crewsync/data
```

Configure o recurso no Coolify:

- Porta interna exposta: `8080`.
- Domínio: `https://sync.exemplo.com`, encaminhado pelo proxy Coolify para a porta
  `8080`. O container atende HTTP privado; o proxy termina TLS e renova o
  certificado ACME. Aponte o DNS `A` para o IPv4 público do host e permita TCP
  `80` e `443` para o proxy.
- Variável `CREWSYNC_PUBLIC_URL`: `https://sync.exemplo.com`.
- Variável secreta `CREWSYNC_ADMIN_PASSWORD`: segredo aleatório hexadecimal de
  64 caracteres, gerado com `openssl rand -hex 32`. Ele será a chave de acesso ao
  painel; não use uma senha curta escolhida por pessoa.
- Montagem persistente de volume Docker em `/etc/crewsync`, para a configuração.
- Bind mount `/opt/crewsync/data` do host em `/var/lib/crewsync`. O caminho no host
  precisa pertencer ao UID/GID 10001 e ter modo `0700`; arquivos, índices e
  credenciais ficam nesse filesystem fora da camada do container.

O container executa como UID/GID 10001 e inicia o servidor automaticamente. No
primeiro boot, grava somente o hash SHA-256 do segredo administrativo; reinícios
não trocam a chave. Guarde a chave em um gerenciador seguro. Mudar a variável no
Coolify não faz rotação; para isso use `admin-key` no terminal do container. Esse
segredo é necessário em todos os starts e para entrar no painel. Quem
administra o host Docker pode inspecionar o ambiente do container.

Após o deploy, acesse `https://sync.exemplo.com/` e entre com o segredo. Cadastre
usuários, compartilhamentos e permissões e gere um convite por dispositivo. Os
clientes usam a URL HTTPS pública e a CA nativa do sistema.

Fixe a tag da versão para controlar atualizações. Para atualizar, troque a imagem
pela tag da nova release e faça redeploy. A tag `alpha` acompanha a última versão
alpha, enquanto uma tag numerada mantém a instalação reproduzível. Faça backup
consistente do bind mount de dados e do volume `/etc/crewsync`; persistência não
substitui backup. Rode apenas uma réplica do servidor.

Como alternativa, o repositório também oferece `compose.yaml` para construir a
imagem localmente com Docker Compose, incluindo filesystem somente leitura,
capabilities removidas e validações do smoke test. No Coolify, selecione Docker
Compose e o arquivo na raiz; defina `CREWSYNC_DATA_DIR` com o diretório absoluto
no host. Não publique a porta 8080 diretamente na Internet.

Para migrar de uma instalação systemd, pare o serviço antigo e copie configuração
e dados com permissões corretas antes de iniciar o container. Nunca execute os
dois servidores simultaneamente sobre o mesmo armazenamento.

## Automação

- CI: build/fmt/Clippy/testes, HTTPS real e site.
- Packages, workflow_dispatch: compila/testa x86_64 e ARM64 musl, executa HTTPS
  com esses binários e gera tar.gz, .deb, .rpm e .pkg.tar.zst com checksums.
  Artefatos ficam na execução.
- Packages, tag vVERSAO: só publica prerelease se o número corresponder ao Cargo,
  não terminar em -dev e existir LICENSE; versão estável exige aceite físico ampliado
  registrado em docs/reports/physical-acceptance.json. Não editar esse aceite
  com base apenas no benchmark localhost. Nunca fabricar resultado físico.
- Pages: gera manual e publica site em https://goncalvesl.github.io/crewsync/.

A matriz de desenvolvimento foi validada em x86_64 e ARM64 na execução
[39804690647](https://github.com/goncalvesl/crewsync/actions/runs/39804690647),
com artefatos disponíveis na execução. Site publicado e verificado em
https://goncalvesl.github.io/crewsync/.

Licença GPL-3.0-only definida; o mantenedor autorizou alpha pública para teste.
Versão estável ainda depende do aceite físico ampliado. Publicações recorrentes:
[RELEASING.md](RELEASING.md). Não há
binário fictício ligado no site. Notas devem indicar versão, testes, compatibilidade
e limites. Não declarar versão estável com o piloto incompleto.

## Site

```bash
python3 scripts/build-site.py
python3 tests/site/check.py target/site
```

HTML/CSS estáticos, sem analytics. Links relativos funcionam sob /crewsync/.
O manual usa docs/MANUAL.md como fonte. QA de navegador adicional em
`tests/site/browser.cjs` usa Playwright instalado fora do projeto e servidor
local na porta 8765, com site sob /crewsync/. Não é dependência do aplicativo.

## Última release

0.1.0-alpha.4: https://github.com/goncalvesl/crewsync/releases/tag/v0.1.0-alpha.4.
Pacotes x86_64/ARM64 e fonte com dependências vendorizadas, cada um com checksum.
Downloads por arquitetura ficam no manual e são atualizados por prepare-release.py.
Fonte: package-source.py; check-source.py compila sem acesso ao cache de crates.
