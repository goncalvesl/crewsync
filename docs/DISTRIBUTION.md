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

Um pacote contém binários, install.sh, exemplos públicos de configuração/systemd,
gerador de certificados e documentação. Nenhum dado privado é coletado da raiz
do piloto. Inclui checksum SHA-256 e avisos das dependências. PREFIX controla a
instalação; default ~/.local. `./install.sh --client` instala somente o cliente,
`--server` instala servidor e auxiliares; sem opção, instala ambos. Manual e GPL
ficam em share/crewsync. Não há sudo, ajustes de PATH ou inicialização automática.

## Automação

- CI: build/fmt/Clippy/testes, HTTPS real e site.
- Packages, workflow_dispatch: compila/testa x86_64 e ARM64 musl, executa HTTPS
  com esses binários, empacota e testa instalação. Artefatos ficam na execução.
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
