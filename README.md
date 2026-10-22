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

O assistente baixa a última release, pergunta os diretórios e inicia o serviço.
No servidor Linux, escolhe HTTPS direto, HTTPS por proxy ou HTTP para teste, cria
configuração e mostra a chave do painel e o primeiro convite. No cliente, pede endereço,
certificado ou CAs do sistema, convite e uma raiz local para ativar e sincronizar todas as pastas
autorizadas, cada uma em sua subpasta. Requer curl/tar/sha256sum e,
no servidor, OpenSSL; dispensa Rust/Cargo/Python. Veja o manual para firewall,
início sem login, atualização e instalações existentes.

Escolha uma raiz local: cada pasta autorizada aparece dentro dela automaticamente.
Cada computador precisa de um convite próprio, mesmo quando pertence ao mesmo
usuário. Veja como [gerenciar pastas e permissões](docs/MANUAL.md#configurar) e
[ativar ou revogar dispositivos](docs/MANUAL.md#dispositivos).

## Downloads

<!-- downloads:start -->
Versão atual: 0.1.0-alpha.7 (alpha, para testes).

- [Linux Intel/AMD (x86_64) — baixar pacote](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.7/crewsync-0.1.0-alpha.7-linux-x86_64.tar.gz) · [SHA-256](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.7/crewsync-0.1.0-alpha.7-linux-x86_64.tar.gz.sha256)
- [Linux ARM64 (aarch64) — baixar pacote](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.7/crewsync-0.1.0-alpha.7-linux-arm64.tar.gz) · [SHA-256](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.7/crewsync-0.1.0-alpha.7-linux-arm64.tar.gz.sha256)

[Notas da versão](https://github.com/goncalvesl/crewsync/releases/tag/v0.1.0-alpha.7) · [Código-fonte e dependências](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.7/crewsync-0.1.0-alpha.7-source.tar.gz)
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

A alpha.7 adiciona o painel administrativo para usuários, pastas compartilhadas,
permissões e dispositivos, no mesmo endereço da sincronização. Também oferece
HTTPS direto, HTTPS por proxy e HTTP explícito para testes. Consulte o
[manual](docs/MANUAL.md#painel-do-servidor) para configurar o proxy e administrar
o servidor. É uma versão alpha para testes; mantenha backup independente.

## Desenvolver

```bash
cargo build --workspace --locked
scripts/check-pilot.sh
```

Rust stable e compilador C para construir; Python padrão e OpenSSL CLI somente
para ferramentas de build/testes/certificados. Runtime Linux kernel >=5.6 com /proc.
Pacotes estáticos musl x86_64/ARM64; matriz CI verifica distribuição.

Leia [AGENTS.md](AGENTS.md), [PROJECT.md](docs/PROJECT.md), [STATUS.md](docs/STATUS.md),
[PLAN.md](docs/PLAN.md), [QUALITY.md](docs/QUALITY.md), [WORKFLOW.md](docs/WORKFLOW.md)
e [TESTING.md](docs/TESTING.md). Organização: crates/server, client e protocol;
config contém exemplos sem segredos; tests/e2e testa processos e HTTPS;
site contém o site público; scripts contém ferramentas operacionais e de build.

Licença: [GNU GPL versão 3 somente (GPL-3.0-only)](LICENSE). Avisos das dependências são preservados nos pacotes.

O painel administrativo está disponível na alpha.7: usuários, pastas, permissões,
convites e dispositivos no mesmo endereço do servidor. Veja [acesso e recuperação
da chave](docs/MANUAL.md#painel-do-servidor).
