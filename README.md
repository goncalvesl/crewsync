# CrewSync

Sincronização de pastas com servidor próprio, permissões por usuário e recuperação
de versões. Implementação em Rust; primeira versão Linux para uma equipe pequena.
Armazenamento raw, sem banco de dados externo. Windows e Android são posteriores.

## Usar

- [Manual completo](docs/MANUAL.md): instalação, início, usuários/dispositivos,
  sincronização, conflitos, histórico, retenção, backup e diagnóstico.
- [Piloto em dois computadores](docs/PILOT.md).
- [Pacotes e publicação](docs/DISTRIBUTION.md); executáveis nativos sem Cargo/Python.
- [Operação](docs/OPERATIONS.md) e [recuperação de versões](docs/HISTORY.md).

O piloto básico em duas máquinas foi confirmado pelo usuário. A matriz ampliada
física e a licença continuam pendentes antes da release pública; não há versão
estável anunciada. Site preparado para https://goncalvesl.github.io/crewsync/.
Consulte [STATUS.md](docs/STATUS.md) para publicação e resultados efetivos.

## Implementado

HTTPS com CA e hostname verificados, ativação temporária por dispositivo,
revogação, ACLs read/write, sync/watch com eventos e reconciliação periódica,
conflitos preservados, exclusões persistentes, histórico/restauração, retenção
configurável, coleta offline com simulação e backup verificável em destino novo.
Não substitui backup independente. Rede privada; veja limites no manual.

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

Licença em definição: [LICENSE-STATUS.md](LICENSE-STATUS.md).
