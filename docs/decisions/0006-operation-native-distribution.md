# 0006 — Operação, runtime Rust e distribuição Linux

> Atualização: GPL-3.0-only e o objetivo de simplicidade foram definidos pelo mantenedor na [decisão 0007](0007-license-and-simplicity.md). Pendências de licença abaixo são históricas.
## Contexto

O usuário autorizou executar todas as etapas restantes sem paradas intermediárias.
A confirmação do piloto básico não comprova a matriz física ampliada. Há acesso
à rede e crates; a restrição que motivou as pontes temporárias terminou.

## Retenção e coleta

Histórico formato 2 acrescenta `archived_at`, por revisão. O timestamp registra
quando a versão deixou de ser atual; não altera revisões nem vence conflitos.
Formato 1 permanece legível e ganha prazo de graça ao arquivar/manter pela primeira
vez. Datas futuras são conservadoras se o relógio recuar.

A política `[server.retention]` aceita history_days/trash_days (1..36500) e
previous_versions (1..1000); defaults 30/30/10. A versão atual nunca expira. A
última versão de arquivo excluído tem a janela própria de lixeira; os tombstones
do manifesto permanecem indefinidamente para clientes offline.

`maintain CONFIG` simula poda e contabiliza bytes lógicos de objetos (não blocos
alocados nem metadados); `--apply` publica os históricos retidos e sincroniza seus
diretórios antes de remover objetos sem referências. Examina todos os manifestos,
inclusive compartilhamentos retirados da configuração. Não existe GC concorrente:
o comando exige o mesmo lock exclusivo do servidor, que deve estar parado.
Assim não pode remover objeto em transferência. Erro/disco cheio antes de persistir
todos os metadados interrompe a coleta. Falha posterior só pode deixar órfãos.
A raiz interna continua exclusiva do serviço; modificações externas não são
suportadas. Cópias locais em `.crewsync` não são podadas.

## Backup e recuperação

Backup exige locks de armazenamento e autenticação. Só aceita destino novo fora
da origem, copia arquivos regulares em streaming, sincroniza e valida a cópia,
salvando marcador de conclusão por último. Restauração também exige destino novo;
valida referências e tokens, sem sobrescrever dados existentes. TLS fica fora da
raiz e deve ser copiado separadamente. Não conectar clientes antigos a um backup
com recuo de revisão sem recuperação assistida e pastas novas.

## Runtime nativo

SHA-256 usa sha2; TOML usa toml/serde; TLS usa rustls/ring e rustls-pemfile, com CA
explícita e verificação de hostname. HTTP/1.1 usa httparse, corpo delimitado por
Content-Length, cabeçalho limitado a 16 KiB, sem chunked, redirects ou Expect.
Conexões fecham após cada resposta. Comprimentos duplicados/ambíguos são recusados.
Streaming e limites permanecem; TLS 1.2/1.3 e timeout 30s. O worker Rust serial e
seu framing são mantidos; IPC interrompido faz reabrir o worker e recuperar tmp.
Logs contêm somente evento, código e tamanho, nunca identidade/caminho/segredos.
As pontes Python e os subprocessos sha256sum foram removidos.

notify acorda o cliente após eventos; fila tem uma vaga, debounce finito de 200ms,
sem seguir symlinks nem reagir a `.crewsync`. A reconciliação periódica cobre perda
de eventos. Sem observador disponível, mantém o modo periódico. Erros de rede
mantêm backoff, sem permitir que eventos provoquem repetição acelerada.

## Distribuição e publicação

Pacotes musl estáticos x86_64 e aarch64, kernel >=5.6 com `/proc`; sem dependência
runtime Python/OpenSSL/glibc. Certificados usam OpenSSL CLI para geração, não para
servir/sincronizar. x86_64 foi construído em rust:alpine e testado no Arch; ARM64
foi depois validado nativamente na matriz CI 39804690647. Pacote contém somente binários, exemplos,
manual e avisos de licença. Instalador usa PREFIX, default ~/.local, sem iniciar
serviços nem alterar dados. Checksums e smoke de extração/instalação são obrigatórios.

GitHub Pages é hospedagem explícita do projeto. Site estático sem dependências de
runtime ou formulários; manual gerado de docs/MANUAL.md. Publicação de pacotes
por tag exige versão correspondente, LICENSE e aceite físico ampliado registrado.
Workflow manual gera artefatos para ensaio sem declarar release pública. Licença
permanece decisão do mantenedor; não atribuir licença nem aceite por suposição.
