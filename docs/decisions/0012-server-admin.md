# 0012 — Painel administrativo no servidor

## Contexto e decisão

O operador precisa gerenciar usuários, compartilhamentos e dispositivos sem editar
TOML a cada mudança. O servidor Rust renderiza HTML/CSS no mesmo listener e origem
pública da sincronização; o proxy existente publica ambos. Não há build de
frontend, banco externo, JavaScript obrigatório ou serviço adicional.

O painel permite adicionar/ativar/bloquear usuários, criar compartilhamentos,
alterar permissões, gerar convites e revogar dispositivos. Compartilhamentos são
nomes virtuais gerenciados pelo armazenamento, sem importação de caminhos do disco.
Não há exclusão de dados, múltiplos administradores ou recuperação por email.

## Autenticação

O comando local `admin-key CONFIG URL_PUBLICA` cria uma chave aleatória de 256 bits,
mostrada somente na saída do comando/instalador. Persistimos apenas SHA-256 da
chave e a origem pública em auth/admin.json privado. SHA-256 aqui verifica um
segredo aleatório de alta entropia, não uma senha escolhida por uma pessoa.
Tokens de sincronização não autorizam o painel. Rotação invalida sessões do painel
sem revogar dispositivos.

Sessões em memória expiram em oito horas, com no máximo 64 sessões; sessão de login
não autenticada dura 20 minutos. Cookie HttpOnly e SameSite Strict, Secure e prefixo
__Host quando a origem pública é HTTPS, inclusive no backend HTTP do proxy. Login
rotaciona o cookie; logout e reinício encerram sessões. POST exige origem exata,
CSRF e formulário limitado a 16 KiB. Host é vinculado à origem configurada, sem
confiar em forwarded. Respostas não têm cache e escapam conteúdo dinâmico, com CSP
restritiva. HTTP explícito fica restrito a testes ou backend protegido do proxy.

## Persistência e consequências

TOML permanece fonte de configuração. Edição preserva campos/comentários, valida o
resultado completo, exige revisão atual, usa lock adjacente, temporário privado,
fsync, rename e fsync do diretório. Configuração desatualizada recebe 409, sem
sobrescrever a edição anterior. A raiz de armazenamento não é editável pelo painel.
Convites e revogação reutilizam o AuthStore existente e suas garantias.

A chave única reduz instalação e recuperação: troca exige acesso local à conta do
servidor. Não oferece auditoria por administrador ou segundo fator. Chaves não
podem ser entregues a usuários comuns. Dispositivos revogados mantêm cópias locais.
O limite visual é de 100 dispositivos por filtro, suficiente ao piloto; não há
paginação nesta etapa. Sessões não persistem em backup ou reinício.

## Validação

Testes unitários de configuração, revisão, autenticação, sessão, origem, CSRF,
rotação, limites e escape; integração com clientes reais em HTTP/HTTPS/Caddy;
fluxo Chromium e layout em 390/768/1440 pixels. Publicação depende de nova release;
a alpha.6 permanece inalterada.
