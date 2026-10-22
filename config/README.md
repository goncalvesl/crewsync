# Exemplos de configuração

[server.example.toml](server.example.toml) define o esquema inicial do servidor,
com usuários e caminhos sintéticos, sem credenciais. As opções, limites e regras
de acesso estão no [manual](../docs/CONFIGURATION.md).

Validar a partir da raiz do repositório, sem criar/abrir o armazenamento:

```bash
cargo run -p crewsync-server -- check-config config/server.example.toml
```

O parsing e a validação deste exemplo são cobertos pelos testes do servidor.
Certificados e ativação por dispositivo estão descritos em [PILOT.md](../docs/PILOT.md).
Unidades systemd de exemplo ficam em `systemd/`; precisam de adaptação local.
