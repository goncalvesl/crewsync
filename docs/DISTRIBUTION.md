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
Os pacotes alpha.9 já publicados ainda usam o assistente anterior; esse fluxo novo
será incluído na próxima release. O deploy Compose construído do repositório já
inicia sem usuários e compartilhamentos.
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

O repositório inclui `Dockerfile` e `compose.yaml`. O Coolify pode clonar o
repositório público e construir a imagem diretamente; não é necessário publicar
uma imagem num registry. O Compose mantém o backend acessível somente pela rede
Docker e expõe internamente a porta 8080 para o proxy do Coolify. HTTPS público
fica a cargo do proxy, que obtém/renova o certificado ACME. Também funciona com
outro host Linux público que rode Docker Compose e um proxy TLS.

Antes de criar o recurso no Coolify, prepare no host o diretório persistente dos
dados (substitua o caminho se preferir outro):

```sh
sudo install -d -o 10001 -g 10001 -m 0700 /opt/crewsync/data
```

No Coolify, crie uma aplicação usando o repositório
`https://github.com/goncalvesl/crewsync`, tipo Docker Compose e o arquivo
`compose.yaml` na raiz. Defina a variável de ambiente `CREWSYNC_DATA_DIR` com o
caminho absoluto criado no host, por exemplo `/opt/crewsync/data`. Essa pasta é
bind-mounted em `/var/lib/crewsync`; arquivos, índices e credenciais ficam no
filesystem do host, fora da camada gravável do container. O volume `crewsync_config`
guarda a configuração do servidor. O processo roda sem root e exige que o diretório
de dados pertença ao UID/GID 10001 com modo `0700`.

No campo Domains do Coolify, associe o serviço `server` à URL pública e à porta
interna 8080. Se a interface pedir a porta junto do endereço, use o formato
`https://sync.exemplo.com:8080`: 8080 seleciona o destino dentro da rede Coolify;
os clientes acessam `https://sync.exemplo.com` pela porta HTTPS 443. Aponte o DNS
`A` do domínio para o IPv4 público do host e permita entrada TCP 80 e 443 para o
proxy. Não publique 8080/8443 na Internet nem adicione `ports:` ao Compose.

Depois do primeiro deploy, abra o terminal do container `server` no Coolify e
gere a chave de acesso ao painel, substituindo a URL:

```sh
crewsync-server admin-key /etc/crewsync/server.toml https://sync.exemplo.com
```

Guarde a chave exibida. Acesse `https://sync.exemplo.com/`, cadastre usuários e
compartilhamentos, defina as permissões e gere um convite por dispositivo. Clientes
usam a URL HTTPS pública e a CA nativa do sistema; não precisam copiar `ca.crt`.
Execute `admin-key` novamente somente se quiser invalidar as sessões atuais e
gerar uma chave administrativa nova.

Execute uma réplica do servidor. Faça backup consistente tanto do caminho definido
em `CREWSYNC_DATA_DIR` quanto do volume `crewsync_config`, e teste uma restauração;
persistência em disco não substitui backup. A atualização da imagem mantém ambos.
Para migrar de uma instalação systemd, pare o serviço antigo e planeje uma cópia
de configuração e dados com permissões corretas antes de ativar o container. Não
execute os dois servidores simultaneamente sobre o mesmo armazenamento.

O repositório público é o ponto de distribuição do código-fonte para o Coolify
construir a aplicação. Para distribuir uma imagem pré-compilada publicamente,
será necessário adicionar um workflow que publique tags versionadas no GHCR; essa
publicação não faz parte do Compose atual.

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
