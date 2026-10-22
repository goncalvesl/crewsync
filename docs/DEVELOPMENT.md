# Desenvolvimento

## Pré-requisitos

Ambiente Linux com Rust stable, Cargo, rustfmt e Clippy. Na instalação por rustup:

```bash
rustup toolchain install stable --profile minimal --component rustfmt --component clippy
rustup default stable
```

Também é possível usar os pacotes da distribuição que forneçam essas ferramentas e suportem a edição 2024. Esta entrega foi validada em Linux com rustc e Cargo 1.99.1; não há MSRV definida. A CI usa Ubuntu 24.04 x86_64 e stable. O servidor usa serde/serde_json para manifestos e rustix para flock; testes usam tempfile. O armazenamento exige GNU coreutils com `/usr/bin/sha256sum`; o parser TOML temporário usa `/usr/bin/python3` >= 3.11 com `tomllib` e `ssl`, sem pacotes Python externos. OpenSSL CLI gera certificados e executa os testes TLS. O cliente exige kernel Linux >= 5.6 e `/proc` montado.

## Compilar e verificar

Na raiz do repositório:

```bash
cargo build --workspace --locked
cargo test --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Para formatar alterações, executar `cargo fmt --all`. Os binários de desenvolvimento ficam em `target/debug/`. Cargo.lock é versionado; mudanças de dependências devem atualizar o lockfile intencionalmente.

## Executar a fundação

```bash
cargo run -p crewsync-server -- --help
cargo run -p crewsync-client -- --help
cargo run -p crewsync-server -- --version
cargo run -p crewsync-client -- --version
cargo run -p crewsync-server -- check-config config/server.example.toml
```

O servidor valida configuração com `check-config` e inicia HTTPS com `serve`. O cliente oferece `activate`, `shares`, `sync` e `watch`; consulte [PILOT.md](PILOT.md) para o fluxo completo. Invocações não suportadas retornam código 2 e uma orientação em stderr. Esta é uma implementação de piloto, sem release estável.

## Organização

`server` implementa armazenamento, revisões, configuração, autenticação, ACLs e worker HTTPS; `client`, estado local e reconciliação. `protocol` contém os contratos comuns e a ponte temporária. Os testes em `tests/e2e/pilot.rs` estão ligados explicitamente ao Cargo do servidor. Veja [TESTING.md](TESTING.md) para validação sem sockets e para o teste de rede externo.

Consulte o [manual do armazenamento](STORAGE.md) para a API e seus limites.

Consultar [a decisão de fundação](decisions/0001-foundation.md), [qualidade](QUALITY.md) e [testes](TESTING.md) antes de implementar. O [manual de configuração](CONFIGURATION.md) descreve o esquema e a fachada autorizada; o exemplo em `config/server.example.toml` é validado nos testes. O roteiro do piloto mostra como iniciar o serviço HTTPS e os dois clientes.
