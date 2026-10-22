# 0002 — Núcleo de armazenamento Linux, formato 1

> Atualização: dependências temporárias e operação foram substituídas pela [decisão 0006](0006-operation-native-distribution.md). Este registro preserva o contexto histórico.
## Decisão

Objetos raw imutáveis identificados por SHA-256 (64 caracteres hexadecimais
minúsculos). Nesta implementação Linux, o cálculo usa `/usr/bin/sha256sum`
(GNU coreutils), em processo separado, sem shell. A dependência Rust `sha2` não
estava disponível no ambiente offline; usar a ferramenta do sistema evita
implementar criptografia própria. Substituir por biblioteca Rust antes de ampliar
as plataformas. O executável precisa estar instalado e ser confiável.

Manifesto JSON versionado por compartilhamento, com revisão monotônica global,
entradas de arquivo/diretório e tombstones. Cada entrada guarda sua revisão;
criações e alterações exigem a revisão-base da entrada (zero para caminho nunca
usado). Conflitos são rejeitados antes da publicação: o chamador conserva o
conteúdo concorrente e decide como apresentá-lo. Não existe resolução automática.
Nomes são UTF-8, sem controles ou caracteres `\ : * ? " < > |`; sem componentes vazios, `.`/`..`, barra invertida, nomes
reservados Windows ou pontos/espaços finais. Limites: 255 bytes por componente,
4096 por caminho, 100 mil entradas e manifesto de até 16 MiB.

## Publicação e recuperação

Diretório privado do serviço, sem acesso de escrita de outros usuários. Lock
`flock` exclusivo e não bloqueante por armazenamento, liberado pelo kernel ao
encerrar o processo. Uma instância possui acesso mutável sequencial. Arquivos
internos especiais e links são recusados; o operador não pode trocar caminhos
enquanto o serviço opera. Isto não é uma sandbox para diretórios hostis. Não há
importação direta de árvores de usuários nesta etapa.

1. Criar temporário exclusivo; copiar em streaming com buffer de 64 KiB e limite
   de tamanho explícito; sincronizar arquivo; verificar SHA-256 esperado.
2. Renomear objeto para seu hash; sincronizar diretório de objetos.
3. Serializar manifesto completo em temporário; sincronizar arquivo; renomear
   sobre manifesto anterior; sincronizar diretório de manifestos.
4. Só então retornar sucesso. Falha após rename pode deixar a nova revisão
   visível mesmo se o chamador recebeu erro; deve reler antes de tentar novamente.

Objetos existentes são verificados antes de deduplicar. O manifesto nunca aponta
para objeto ainda não publicado. Na abertura, remover temporários, validar
manifestos, caminhos, hierarquia, revisões e hashes de todos os objetos
referenciados. Corrupção causa erro, sem reparar nem apagar dados confirmados.
Objetos órfãos são preservados; coleta e retenção pertencem à etapa 5. Exclusão
de diretório com descendentes vivos é recusada; tombstones permanecem.

As garantias dependem de filesystem Linux local que respeite `fsync` em arquivos,
diretórios e rename atômico; não prometemos durabilidade em NFS ou hardware que
ignore flush. Testes simulam interrupções entre os passos por estados de disco;
não substituem ensaios físicos de queda de energia.

## Limites

Não há rede, autenticação, CLI operacional, histórico consultável, coleta,
monitoramento, Windows ou sincronização nesta entrega. JSON reescreve o manifesto
inteiro, adequado ao piloto; não estabilizar protocolo externo a partir dele.
