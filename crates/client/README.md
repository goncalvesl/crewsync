# client

Cliente Linux do piloto, com ativação por HTTPS e reconciliação por revisão-base.

```bash
cargo run -p crewsync-client -- --help
```

Comandos: `activate URL CA SESSION NAME` (convite por stdin), `shares SESSION`,
`sync SESSION SHARE FOLDER` e `watch SESSION SHARE FOLDER [SECONDS]`.
Uma pasta por compartilhamento; credencial privada fora da árvore sincronizada.
O modo watch usa varredura periódica, ainda sem observador de eventos.

Baseline, cópias substituídas e conflitos ficam em `.crewsync/` (não sincronizado).
Links/arquivos especiais são recusados; acesso a arquivos usa descritores e
`openat2`, exigindo Linux >= 5.6 com `/proc` montado.

Veja o [roteiro completo para dois computadores](../../docs/PILOT.md).
