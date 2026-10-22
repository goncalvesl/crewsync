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

O parser TOML agora é nativo em Rust. `[server.retention]` controla a manutenção
administrativa offline (defaults 30 dias de histórico/lixeira e 10 versões).
`maintain CONFIG` simula; `--apply` aplica com servidor parado. Consulte o manual.

Na próxima versão, o assistente do servidor oferece HTTPS direto, proxy HTTPS e
HTTP explícito para testes. O modo proxy gera Caddyfile com upstream em loopback;
veja a receita e cuidados de rede no [manual](../docs/MANUAL.md#conexao-e-rede).
Os três modos usam o mesmo TOML e não alteram permissões. Alpha.6 pública ainda
não inclui os novos modos.
