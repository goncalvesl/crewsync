# server

Servidor Linux do piloto: armazenamento raw, configuração TOML, ACLs, convites,
tokens por dispositivo e worker autenticado atrás da ponte HTTPS embarcada.

```bash
cargo run -p crewsync-server -- --help
cargo run -p crewsync-server -- check-config config/server.example.toml
```

Comandos: `invite CONFIG USER SECONDS`, `devices CONFIG`,
`revoke CONFIG DEVICE_ID`, `serve CONFIG HOST PORT CERT KEY`.
Convites são exibidos apenas ao emitir; tokens não são impressos nem registrados.
`worker CONFIG` é a interface interna de stdin/stdout da ponte, não um endpoint
ou protocolo para expor por outro transporte.

Veja [piloto](../../docs/PILOT.md), [configuração](../../docs/CONFIGURATION.md),
[armazenamento](../../docs/STORAGE.md) e decisão [0004](../../docs/decisions/0004-pilot.md).
Testes de rede exigem sockets e são executados separadamente, conforme TESTING.md.
