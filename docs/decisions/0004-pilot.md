# 0004 — Piloto Linux com autenticação e HTTPS

> Atualização: dependências temporárias e operação foram substituídas pela [decisão 0006](0006-operation-native-distribution.md). Este registro preserva o contexto histórico.
## Objetivo autorizado

Prosseguir sequencialmente até servidor e dois clientes serem testáveis. O piloto
precede histórico/retencão completos e distribuição pública; não equivale a
concluir as etapas 5–8. Usar inicialmente dados sintéticos.

## Identidade e persistência

Convites e tokens: 32 bytes aleatórios de `/dev/urandom`, representados em hex.
SHA-256 apenas no servidor; códigos e tokens não aparecem em logs nem argumentos.
Convite emitido por administrador local, expiração em Unix UTC, uso único.
Dispositivo tem identificador aleatório independente e pode ser revogado.
Estado JSON privado com flock transacional, temporário exclusivo, fsync de arquivo,
rename e fsync do diretório antes da confirmação. Ativação publica consumo do
convite e novo dispositivo juntos. Perda da resposta pode exigir novo convite;
nunca reutilizar convite consumido. Mudança regressiva de relógio deve ser evitada
pelo operador. Estado corrompido falha fechado. Administração pode executar com
serviço ativo, usando lock separado das transferências.

## Transporte temporário

Sem crates TLS/HTTP disponíveis no cache e sem DNS para crates.io. Não escrever
TLS/HTTP próprios. Ponte Python padrão (`ssl`, `http.server`, `http.client`) embarcada
no binário; Rust continua dono de autenticação, ACLs e dados. HTTPS obrigatório,
TLS >= 1.2, validação da cadeia e hostname, sem opção insegura ou redirects.
Certificado/chave fornecidos pelo operador. Referências primárias:
https://docs.python.org/3/library/ssl.html e
https://docs.python.org/3/library/http.client.html.

Uma requisição por vez no piloto, buffers de 64 KiB, timeout de socket, frame
JSON limitado e corpo raw com tamanho explícito. RPC versão 1 sobre POST
`/v1/rpc`; sem compressão. Falha da ponte/worker fecha operação; temporários não
confirmados são recuperados ao reabrir. Limitar exposição a rede privada do piloto,
sem prometer endurecimento de um servidor HTTP público. Migrar ponte e parsers
para bibliotecas Rust quando disponíveis, preservando os testes observáveis.

## Reconciliação inicial

Uma pasta local por compartilhamento. Baseline local privado com entradas e
revisões remotas; comparar baseline, disco e manifesto remoto. Upload exige
revisão-base e snapshot imutável verificado. Downloads verificados antes de
publicação. Não inferir exclusão de arquivo nunca sincronizado. Tombstone impede
ressurreição por cliente antigo. Diretórios vazios são explícitos; movimentação
inicial é exclusão/criação. Links e arquivos especiais abortam a reconciliação.

Preservar arquivos substituídos/excluídos e conflitos em `.crewsync/`, fora da
árvore sincronizada. Publicar com operações relativas a descritores e recusar
symlinks. Estado local só avança depois de confirmar a operação. Identidade do
servidor/compartilhamento vinculada ao baseline. Duas instâncias na mesma pasta
são recusadas. Repetir uma operação com resposta perdida deve comparar estado
atual, sem sobrescrever conteúdo concorrente.

Monitor inicial: reconciliação periódica em primeiro plano, com atraso limitado
em falhas transitórias e parada em erros permanentes. Observação de eventos,
histórico consultável, retenção/coleta e serviço de sistema completos permanecem
pendentes; preservar cópias internas sem limpeza automática no piloto.
