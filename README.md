# CrewSync

Sincronização de pastas com servidor próprio, permissões por usuário e recuperação
de versões. Implementação em Rust; primeira versão Linux para uma equipe pequena.
O objetivo é ser **simples de instalar, manter e usar, além de rápido**, preservando
a integridade dos arquivos. Instalação com o mínimo de passos e dependências;
manual direto, com aprofundamento técnico apenas na seção final.
Armazenamento raw, sem banco de dados externo. Windows e Android são posteriores.

## Instalar com um comando

Servidor:

```sh
curl -fsSL https://raw.githubusercontent.com/goncalvesl/crewsync/main/scripts/setup-online.sh | sh -s -- server
```

Cliente:

```sh
curl -fsSL https://raw.githubusercontent.com/goncalvesl/crewsync/main/scripts/setup-online.sh | sh -s -- client
```

O assistente baixa a última release e pergunta os diretórios. No servidor Linux,
escolhe HTTPS direto, HTTPS por proxy ou HTTP para teste, inicia o serviço e mostra
a chave do painel. Cadastre usuários e compartilhamentos no painel e gere um convite
para cada dispositivo. No cliente, pede endereço,
certificado ou CAs do sistema, convite e uma raiz local para ativar e sincronizar todas as pastas
autorizadas, cada uma em sua subpasta. Requer curl/tar/sha256sum e,
no servidor, OpenSSL; dispensa Rust/Cargo/Python. Veja o manual para firewall,
início sem login, atualização e instalações existentes.

A alpha.10 adiciona o bootstrap vazio do servidor nos pacotes e a imagem Docker
multiarch pública para Coolify. Os usuários e compartilhamentos iniciais são
cadastrados pelo painel administrativo.

Escolha uma raiz local: cada pasta autorizada aparece dentro dela automaticamente.
Cada computador precisa de um convite próprio, mesmo quando pertence ao mesmo
usuário. Veja como [gerenciar pastas e permissões](docs/MANUAL.md#configurar) e
[ativar ou revogar dispositivos](docs/MANUAL.md#dispositivos).

Para o servidor em Linux, a release também fornece pacotes Arch, Debian e Fedora
com serviço systemd e usuário dedicado, além de tar.gz instalável em `/opt`.
Consulte [os formatos e comandos de instalação](docs/DISTRIBUTION.md#servidor-em-pacote-de-sistema).

Para instalar com Coolify, use **Docker Image** e `ghcr.io/goncalvesl/crewsync-server:0.1.0-alpha.10`; configure TLS no proxy e monte os dados no host. Veja [Docker e Coolify](docs/DISTRIBUTION.md#docker-e-coolify).

## Downloads

<!-- downloads:start -->
Versão atual: 0.1.0-alpha.10 (alpha, para testes).

- **Arch Linux:** [x86_64](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.10/crewsync-0.1.0_alpha.10-1-x86_64.pkg.tar.zst) · [SHA-256](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.10/crewsync-0.1.0_alpha.10-1-x86_64.pkg.tar.zst.sha256)
- **Debian/Ubuntu:** [amd64](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.10/crewsync_0.1.0.alpha.10-1_amd64.deb) · [SHA-256](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.10/crewsync_0.1.0.alpha.10-1_amd64.deb.sha256) · [ARM64](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.10/crewsync_0.1.0.alpha.10-1_arm64.deb) · [SHA-256](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.10/crewsync_0.1.0.alpha.10-1_arm64.deb.sha256)
- **Fedora:** [x86_64](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.10/crewsync-0.1.0.alpha.10-1.x86_64.rpm) · [SHA-256](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.10/crewsync-0.1.0.alpha.10-1.x86_64.rpm.sha256) · [ARM64](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.10/crewsync-0.1.0.alpha.10-1.aarch64.rpm) · [SHA-256](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.10/crewsync-0.1.0.alpha.10-1.aarch64.rpm.sha256)
- **Tar.gz independente:** [x86_64](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.10/crewsync-0.1.0-alpha.10-linux-x86_64.tar.gz) · [SHA-256](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.10/crewsync-0.1.0-alpha.10-linux-x86_64.tar.gz.sha256) · [ARM64](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.10/crewsync-0.1.0-alpha.10-linux-arm64.tar.gz) · [SHA-256](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.10/crewsync-0.1.0-alpha.10-linux-arm64.tar.gz.sha256)

[Notas da versão](https://github.com/goncalvesl/crewsync/releases/tag/v0.1.0-alpha.10) · [Código-fonte e dependências](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.10/crewsync-0.1.0-alpha.10-source.tar.gz)
<!-- downloads:end -->

## Usar

- [Manual completo](docs/MANUAL.md): instalação, início, usuários/dispositivos,
  sincronização, conflitos, histórico, retenção, backup e diagnóstico.
- [Piloto em dois computadores](docs/PILOT.md).
- [Pacotes e publicação](docs/DISTRIBUTION.md); executáveis nativos sem Cargo/Python.
- [Operação](docs/OPERATIONS.md) e [recuperação de versões](docs/HISTORY.md).

O piloto básico em duas máquinas foi confirmado pelo usuário. A matriz ampliada
física continua pendente. A publicação alpha é autorizada para testes; não há
versão estável anunciada. Site publicado em https://goncalvesl.github.io/crewsync/.
Consulte [STATUS.md](docs/STATUS.md) para publicação e resultados efetivos.

## Implementado

HTTPS com CA e hostname verificados, ativação temporária por dispositivo,
revogação, ACLs read/write, sync/watch com eventos e reconciliação periódica,
conflitos preservados, exclusões persistentes, histórico/restauração, retenção
configurável, coleta offline com simulação e backup verificável em destino novo.
Não substitui backup independente. Servidor remoto ou local por HTTPS; veja limites no manual.

## Servidor e conexão

A alpha.8 publica pacotes Arch, Debian, Fedora e tar.gz para instalar o servidor
Linux como serviço systemd sob usuário dedicado. Mantém o painel administrativo,
HTTPS direto, HTTPS por proxy e HTTP explícito para testes. Consulte o
[manual de instalação](docs/MANUAL.md#instalar) e a [documentação dos pacotes](docs/DISTRIBUTION.md)
para escolher o formato. É uma versão alpha para testes; mantenha backup independente.

## Desenvolver

```bash
cargo build --workspace --locked
scripts/check-pilot.sh
```

Rust stable e compilador C para construir; Python padrão e OpenSSL CLI somente
para ferramentas de build/testes/certificados. Runtime Linux kernel >=5.6 com /proc.
Binários estáticos musl x86_64/ARM64; releases oferecem `.pkg.tar.zst`, `.deb`,
`.rpm` e tar.gz para o servidor Linux.

Leia [AGENTS.md](AGENTS.md), [PROJECT.md](docs/PROJECT.md), [STATUS.md](docs/STATUS.md),
[PLAN.md](docs/PLAN.md), [QUALITY.md](docs/QUALITY.md), [WORKFLOW.md](docs/WORKFLOW.md)
e [TESTING.md](docs/TESTING.md). Organização: crates/server, client e protocol;
config contém exemplos sem segredos; tests/e2e testa processos e HTTPS;
site contém o site público; scripts contém ferramentas operacionais e de build.

Licença: [GNU GPL versão 3 somente (GPL-3.0-only)](LICENSE). Avisos das dependências são preservados nos pacotes.

O painel administrativo está disponível na alpha.7: usuários, pastas, permissões,
convites e dispositivos no mesmo endereço do servidor. Veja [acesso e recuperação
da chave](docs/MANUAL.md#painel-do-servidor).
