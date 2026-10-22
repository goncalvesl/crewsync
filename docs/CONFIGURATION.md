# Configuração e permissões do servidor

A configuração TOML define o armazenamento e as permissões. Autenticação por
dispositivo e servidor HTTPS estão implementados no [piloto](PILOT.md). O exemplo
[server.example.toml](../config/server.example.toml) contém apenas dados sintéticos.

## Validar sem iniciar o servidor

```bash
cargo run -p crewsync-server -- check-config config/server.example.toml
```

O comando apenas lê e valida: não abre/cria o armazenamento, não obtém o bloqueio
nem verifica sua existência, permissões ou integridade. Sucesso retorna código 0
e as contagens de usuários e compartilhamentos; configuração inválida/ausente
retorna 1, invocação incorreta retorna 2. Erros não reproduzem valores do TOML.

Requer `/usr/bin/python3` >= 3.11 com `tomllib`. A biblioteca padrão é executada
em modo isolado, sem shell, apenas para converter TOML em JSON; toda a validação
de esquema e autorização permanece em Rust. É uma dependência temporária pela
indisponibilidade da crate `toml` no ambiente de implementação. Veja a
[decisão 0003](decisions/0003-config-authorization.md) e a
[documentação primária do tomllib](https://docs.python.org/3/library/tomllib.html).

## Esquema versão 1

```toml
version = 1

[server]
storage_root = "/var/lib/crewsync"
max_upload_bytes = 1_073_741_824

[users.ana]
enabled = true

[users.bruno]
enabled = true

[shares.Obras.permissions]
ana = "write"
bruno = "read"
```

- `version` e `server.storage_root` são obrigatórios. A raiz é absoluta, sem `..`,
  diferente de `/`; ao abrir o armazenamento ela precisa ser privada (0700), sem
  links, e seu pai deve existir. O caminho é do armazenamento interno, não uma
  pasta externa exportada diretamente.
- `max_upload_bytes` é opcional, padrão 1 GiB; aceita inteiros de 1 byte a 1 TiB.
  É um limite por envio, aplicado inclusive em atualizações; não limita o total
  armazenado nem remove arquivos anteriores maiores que o novo limite.
- Identificadores de usuário têm de 1 a 64 caracteres ASCII alfanuméricos, `_` e
  `-`, distinguindo maiúsculas/minúsculas. Somente `enabled = true` ativa o usuário;
  omitir `enabled` deixa-o desativado.
- Compartilhamentos são nomes de primeiro nível com as [regras de nomes do
  armazenamento](STORAGE.md), incluindo suporte a acentos. Subpastas herdam a
  permissão; não são configuradas separadamente. Não há grupos ou curingas.
- `read` permite listar manifesto e ler arquivos; `write` inclui tudo de `read`,
  além de criar diretórios, criar/alterar arquivos e excluir entradas.
- Permissão ausente, usuário desconhecido/desativado ou compartilhamento
  desconhecido negam acesso. Um compartilhamento sem `permissions` não concede
  acesso a ninguém. `users` e `shares` podem estar ausentes ou vazios.

Campos desconhecidos, chaves duplicadas, versão desconhecida, tipos incorretos,
permissão diferente de `read`/`write`, referência a usuário inexistente e nomes
inválidos rejeitam a configuração inteira. Remover um usuário exige remover suas
referências nas permissões; desativá-lo permite conservar essas referências.
Arquivo: UTF-8, regular (sem symlink no componente final), até 1 MiB, no máximo
1000 usuários e 1000 compartilhamentos. Somente o operador deve poder modificá-lo.
O arquivo não contém tokens, convites ou credenciais.

## API local e recarga

`crewsync_server::access::AuthorizedStore::open(config_path)` carrega a
configuração, abre o armazenamento e mantém seu bloqueio exclusivo. Oferece:

- `shares(user)` lista somente compartilhamentos legíveis.
- `manifest(user, share)` consulta estado após verificar leitura.
- `file(user, share, path)` resolve o hash através de um caminho vivo autorizado.
- `put_file(user, share, path, base, expected_hash, reader)` verifica escrita,
  caminho e revisão-base antes de consumir o envio e publica após verificar hash.
- `create_directory(user, share, path, base)` e `delete(user, share, path, base)`
  exigem escrita e preservam as regras de revisão do armazenamento.
- `reload_config()` relê o arquivo original e só troca a configuração depois de
  validar tudo; erros mantêm a última configuração válida. Mudar `storage_root`
  é recusado, exigindo reinício. O limite de upload pode mudar por recarga.

O código que chama essa API precisa fornecer identidade confiável. O parâmetro
`user` não autentica ninguém e não deve ser aceito diretamente de uma requisição
remota. No piloto, o worker deriva esse usuário exclusivamente do token do
dispositivo, verifica ativação/revogação e só então chama a fachada. `storage::Store` é uma API de baixo nível sem ACLs, destinada somente ao
código confiável do servidor; não expor seus métodos como endpoints.

Acesso negado retorna `PermissionDenied` com a mesma mensagem para identidade
ou compartilhamento desconhecido, desativado ou sem permissão. Sem conceder
escrita, a API não consome o leitor de upload. Não existe leitura pública por hash
arbitrário: conhecer um hash não concede acesso ao objeto global deduplicado.
Não há atalho para associar um hash existente a outro compartilhamento sem enviar
os bytes e verificar seu conteúdo.

Mudanças válidas de acesso se aplicam na próxima operação. Remover um
compartilhamento retira o acesso, sem apagar seu manifesto ou objetos; adicioná-lo
novamente devolve o acesso conforme as novas permissões. Um arquivo já aberto
continua legível; não há revogação retroativa de leitores existentes. A API é
síncrona e sequencial para alterações e recarga, sem monitor de arquivo ou sinal.

Para editar a configuração, prepare o arquivo completo e publique-o por rename
no mesmo diretório antes de chamar `reload_config()`; não exponha um arquivo
parcial. O worker HTTPS tenta recarregar antes de cada requisição; não há comando CLI de
recarga por sinal.

## Testar

```bash
cargo test -p crewsync-server --test config --test access --test cli --locked
```

Os testes verificam o exemplo, esquema, erros sem conteúdo de configuração,
read/write, subpastas, objetos isolados, uploads negados/conflitantes sem leitura,
limites e mudanças/erros de recarga. Autenticação, expiração de convites, tokens, revogação por dispositivo e TLS têm
testes adicionais descritos em [TESTING.md](TESTING.md).
