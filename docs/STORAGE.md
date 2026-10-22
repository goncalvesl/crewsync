# Armazenamento local do servidor

A API Rust `crewsync_server::storage` é um núcleo local para Linux. A fachada autorizada e o serviço HTTPS do [piloto](PILOT.md) usam este núcleo;
esta API de baixo nível não autentica clientes. Requer
SHA-256 nativo (sha2) e filesystem local com suporte a `fsync`
de diretórios e renomeação atômica.

## API

- `Store::open(root)`: cria a raiz (seu pai deve existir), obtém bloqueio exclusivo
  não bloqueante, limpa temporários e valida manifestos e objetos referenciados.
  A raiz e os subdiretórios internos devem ter permissão 0700; links são recusados
  inclusive nos ancestrais da raiz. Uma segunda instância recebe erro de bloqueio.
- `put_object(reader, expected_sha256, max_bytes)`: recebe conteúdo raw em buffer
  de 64 KiB, limita o tamanho e verifica hash antes de publicar. Retorna o tamanho.
  Conteúdo igual ocupa um único objeto, inclusive entre compartilhamentos.
- `object(hash, size)`: verifica integridade e devolve `File` para leitura streaming.
- `manifest(share)`: devolve o estado atual; compartilhamento sem manifesto é
  vazio. Isto não concede autorização nem expõe essa API a clientes remotos.
- `history(share, path)`: lista versões disponíveis, incluindo o estado atual.
- `restore(share, path, revision, base)`: publica conteúdo histórico como nova revisão.
- `apply(share, path, base_revision, content)`: publica arquivo, diretório ou
  exclusão. O objeto deve estar publicado antes de referenciá-lo. Pais precisam
  existir como diretórios. Diretórios vazios são entradas explícitas.

`base_revision` é a revisão da entrada, não a global do compartilhamento: zero
somente para caminho nunca usado. Alteração concorrente retorna
`io::ErrorKind::AlreadyExists` sem mudar o manifesto; o objeto concorrente é
preservado. O futuro cliente deverá manter a versão local e apresentar o conflito.
Exclusões usam `Content::Deleted`, conservando a revisão para impedir recriação
por um cliente desatualizado. Apague descendentes antes de excluir um diretório.

## Formato e operação

```text
root/                 # privado, 0700
  lock                # flock; arquivo permanece, bloqueio termina com o processo
  objects/<sha256>    # bytes originais, sem compactação
  manifests/<share>   # JSON formato 1
  history/<share>/<sha256-do-caminho> # versões anteriores, JSON formato 1
  tmp/                # gravações ainda não confirmadas
  auth/               # hashes de convites/tokens e estado de dispositivos
```

Não editar os arquivos internos, trocar diretórios ou compartilhar a raiz com
usuários locais não confiáveis. Entradas inválidas, arquivos especiais,
manifestos corrompidos e objetos referenciados ausentes/corrompidos causam erro.
Nenhum dado confirmado é reparado ou removido automaticamente. Temporários são
removidos ao reabrir; objetos órfãos são mantidos. Não remova o arquivo de lock
para tentar contornar uma instância em execução.

O hash deve ter 64 dígitos hexadecimais minúsculos. Caminhos são relativos, até
4096 bytes, com componentes UTF-8 de até 255 bytes. Caracteres de controle,
`\ : * ? " < > |`, `.`/`..`, componentes vazios, nomes reservados Windows e
pontos/espaços finais são recusados. Nomes com acentos são aceitos; não há
normalização Unicode e os caminhos distinguem maiúsculas de minúsculas no Linux. Compartilhamentos usam um único componente. `.crewsync` é reservado no primeiro
componente; a profundidade máxima é 128. Manifestos têm
limites de 100 mil entradas (incluindo exclusões) e 16 MiB.

A confirmação ocorre após sincronizar o manifesto e seu diretório. Um erro
após renomeação pode deixar a nova revisão visível: releia o manifesto antes de
repetir a operação. Em recuperação, uma interrupção anterior à publicação do
manifesto mantém o estado anterior e pode deixar objetos órfãos. A ordem e as
limitações de durabilidade estão na [decisão 0002](decisions/0002-storage.md).

O manifesto inteiro é reescrito e seus objetos são verificados em cada mutação;
a abertura também verifica os compartilhamentos. Esta escolha prioriza integridade
e simplicidade; desempenho com dados reais ainda não foi medido. Há [histórico consultável e restauração](HISTORY.md); há retenção configurável e coleta administrativa offline; não há quotas totais. Esta API de baixo nível não aplica
ACLs; a fachada [AuthorizedStore](CONFIGURATION.md) aplica as permissões locais. Não usar como substituto de backup; validar o roteiro do piloto antes de qualquer
uso real.

## Verificação

```bash
cargo test -p crewsync-server --locked
```

Os testes usam dados sintéticos e diretórios temporários, incluindo transferência
maior que o buffer, interrupções de leitura, hashes errados, corrupção, conflitos,
exclusões após reabertura, links e falhas simuladas em oito pontos de publicação.
Eles não simulam a perda de caches do kernel nem substituem ensaios reais de
queda de energia e disco cheio no piloto.

A publicação agora preserva os metadados anteriores antes de substituir o manifesto.
Detalhes, recuperação e compatibilidade: [decisão 0005](decisions/0005-history.md).
