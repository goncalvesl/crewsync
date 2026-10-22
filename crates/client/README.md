# client

Cliente Linux do piloto, com ativação por HTTPS e reconciliação por revisão-base.

```bash
cargo run -p crewsync-client -- --help
```

Comandos: `activate URL CA SESSION NAME` (convite por stdin), `shares SESSION`,
`sync-root SESSION ROOT` e `watch-root SESSION ROOT [SECONDS]`.
A raiz contém uma subpasta por compartilhamento autorizado, descoberto a cada ciclo.
Os comandos individuais `sync SESSION SHARE FOLDER` e `watch SESSION SHARE FOLDER [SECONDS]`
continuam disponíveis. A credencial privada deve ficar fora da árvore sincronizada.
O modo watch combina eventos locais com reconciliação periódica.

Baseline, cópias substituídas e conflitos ficam em `.crewsync/` (não sincronizado).
Links/arquivos especiais são recusados; acesso a arquivos usa descritores e
`openat2`, exigindo Linux >= 5.6 com `/proc` montado.

Veja o [roteiro completo para dois computadores](../../docs/PILOT.md).
