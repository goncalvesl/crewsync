# 0003 — Configuração e autorização local, formato 1

> Atualização: dependências temporárias e operação foram substituídas pela [decisão 0006](0006-operation-native-distribution.md). Este registro preserva o contexto histórico.
## Contexto e decisão

Primeira parte da etapa 3: configurar usuários, compartilhamentos e permissões
antes de introduzir identidade por dispositivo e transporte HTTPS. Configuração
TOML versionada, esquema estrito, negação por padrão. Usuários só ficam ativos
com `enabled = true`; permissões por compartilhamento são `read` ou `write`.
`write` inclui leitura, criação, alteração e exclusão; subpastas herdam o acesso.
Referências a usuários inexistentes, campos desconhecidos, valores incorretos,
nomes inválidos e versões desconhecidas impedem carga e recarga.

A raiz de armazenamento é absoluta e fixa durante a execução. O operador deve
reiniciar para mudá-la; isto evita trocar o destino dos dados silenciosamente.
O limite por upload é configurável e aplicado antes da publicação. A configuração
é limitada a 1 MiB e deve ser arquivo regular, sem symlink no componente final.

A fachada `AuthorizedStore` possui o armazenamento e verifica acesso a cada
operação. Não oferece acesso a objetos por hash arbitrário: leitura exige caminho
vivo de um manifesto autorizado. Deduplicação global não concede acesso a objetos
de outro compartilhamento. Usuário desconhecido, desativado, sem permissão ou
compartilhamento desconhecido produzem o mesmo erro `PermissionDenied`.

A identidade recebida pela API local é fornecida por código confiável. Ela não é
prova de autenticação e não pode ser usada diretamente como parâmetro vindo da
rede. A API de armazenamento de baixo nível continua interna ao domínio de
confiança do servidor. Tokens e HTTPS ainda precisam ser implementados.

Recarga é síncrona e transacional em memória: carregar e validar tudo, recusar
mudança de raiz, depois substituir a configuração. Falha mantém integralmente
o estado anterior. Toda operação posterior usa a configuração atual; arquivos
já abertos não são revogados retroativamente. Não há monitor automático nem
recarga por sinal nesta entrega.

## Parser e alternativas

Não foi possível obter a crate `toml`: DNS do crates.io indisponível e dependência
ausente no cache. Não implementar um parser TOML próprio nem aceitar apenas um
subconjunto silencioso. Temporariamente executar `/usr/bin/python3 -I -c` com
`tomllib` (Python >= 3.11), enviando os bytes por stdin e recebendo JSON. Não usar
shell, módulos do diretório atual ou texto de configuração em argumentos. Rust
aplica o esquema e as validações. Erros não reproduzem conteúdo da configuração.
Limite de entrada restringe o uso de recursos; o arquivo é administrado pelo
operador, não uma entrada de rede. Não há dependência Python de terceiros.

Referência primária: [tomllib](https://docs.python.org/3/library/tomllib.html).
Substituir o processo por uma biblioteca TOML Rust quando puder ser obtida e
validada, preservando os testes e o formato. Manter Python como dependência
permanente ou escrever um parser próprio seriam alternativas piores para a
simplicidade operacional desejada. Formato inicial usa recursos de TOML 1.0.

## Verificação

Testar ACLs com leitura, escrita e exclusão reais; isolamento de objetos;
negação antes de consumir uploads; caminhos e referências inválidas; limites;
recarga inválida preservando a última configuração; remoção, desativação e
redução de permissões com efeito imediato nas operações posteriores. Validar
CLI sem abrir/criar armazenamento e testar o exemplo versionado.
