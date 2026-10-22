# Manual do CrewSync

CrewSync sincroniza pastas entre computadores Linux usando um servidor central próprio. Esta versão é de desenvolvimento para rede privada. Comece com dados sintéticos e mantenha backup separado.

## Instalar

Os executáveis são Rust nativo: não exigem Cargo, Python, OpenSSL instalado ou `sha256sum` para funcionar. Linux kernel 5.6 ou superior, `/proc` montado e filesystem local com `fsync` e renomeação atômica são necessários. Não usar NFS como raiz interna.

Consulte os pacotes efetivamente publicados em [GitHub Releases](https://github.com/goncalvesl/crewsync/releases). Pacotes musl estáticos dispensam uma versão específica de glibc. x86_64 é validado localmente; ARM64 passa pela matriz de distribuição. Não execute binários de arquitetura diferente.

Depois de baixar o pacote e seu arquivo `.sha256` para o mesmo diretório:

```bash
sha256sum -c crewsync-VERSAO-ALVO.tar.gz.sha256
tar -xzf crewsync-VERSAO-ALVO.tar.gz
cd crewsync-VERSAO-ALVO
./install.sh
```

O instalador coloca os binários em `$HOME/.local/bin` por padrão. Ajuste `PREFIX` se necessário. Não altera sua configuração ou inicia serviços. Para compilar do código, instale Rust stable e um compilador C, clone o repositório e execute:

```bash
cargo build --release --workspace --locked
mkdir -p "$HOME/.local/bin"
install -m 755 target/release/crewsync-server target/release/crewsync-client "$HOME/.local/bin/"
```

## Configurar

Na máquina do servidor, crie uma pasta privada para configuração e copie `config/server.example.toml`. Adapte `storage_root` para um caminho absoluto privado, separado das pastas dos clientes. O pai precisa existir. A raiz interna exige permissão 0700 e é criada pelo serviço quando ausente.

```toml
version = 1
[server]
storage_root = "/home/SEU_USUARIO/crewsync-pilot/store"
max_upload_bytes = 1073741824
[server.retention]
history_days = 30
previous_versions = 10
trash_days = 30
[users.ana]
enabled = true
[users.bruno]
enabled = true
[shares.Obras.permissions]
ana = "write"
bruno = "read"
```

Somente pastas de primeiro nível são compartilhamentos. Subpastas herdam o acesso. `read` permite consultar e baixar; `write` inclui criar, alterar, restaurar e excluir. Usuários ausentes, desativados ou sem permissão não têm acesso. A recarga acontece antes de cada requisição; erro mantém a última configuração válida. Trocar `storage_root` exige reiniciar.

```bash
crewsync-server check-config "$HOME/crewsync-pilot/server.toml"
```

## HTTPS e rede

Descubra o IP do servidor e reserve esse endereço na sua rede. Gere uma CA privada e um certificado com o script fornecido, que exige OpenSSL CLI somente para a geração:

```bash
scripts/pilot-cert.sh "$HOME/crewsync-pilot/tls" 192.168.1.10
crewsync-server serve "$HOME/crewsync-pilot/server.toml" 0.0.0.0 8443 "$HOME/crewsync-pilot/tls/server.crt" "$HOME/crewsync-pilot/tls/server.key"
```

Substitua o IP pelo real. Aguarde `CrewSync HTTPS ready 8443`. Copie apenas `ca.crt` para os clientes por um meio confiável; mantenha `server.key` e `ca.key` privados. O certificado do servidor vence em 90 dias. Renove antes do vencimento e reinicie; mantendo a CA, os clientes não precisam trocar sua cópia.

O endereço usado pelo cliente precisa constar no certificado. Não existe opção de ignorar validação TLS. `0.0.0.0` é endereço de escuta, nunca o endereço usado pelos clientes. Se usar UFW, permita a porta TCP apenas da rede local real:

```bash
sudo ufw allow from 192.168.1.0/24 to any port 8443 proto tcp
```

Não é necessário abrir portas nos clientes. O servidor é serializado, com timeout de conexão de 30 segundos, para um piloto em rede privada.

## Dispositivos

Cada computador recebe seu próprio convite de ativação. No servidor:

```bash
crewsync-server invite "$HOME/crewsync-pilot/server.toml" ana 600
```

O convite é secreto, temporário e de uso único. No cliente, coloque a CA confiável em `$HOME/.config/crewsync/ca.crt` e crie uma pasta de sincronização nova:

```bash
mkdir -p "$HOME/.config/crewsync" "$HOME/CrewSync-Obras"
chmod 700 "$HOME/.config/crewsync"
crewsync-client activate https://192.168.1.10:8443 "$HOME/.config/crewsync/ca.crt" "$HOME/.config/crewsync/session.json" pc1
```

Cole o convite e pressione Enter. A sessão fica privada (0600), fora da pasta sincronizada. Não copie a sessão para outro computador: gere outro convite. Se a ativação perder a resposta depois de consumir o convite, emita um novo.

```bash
crewsync-client shares "$HOME/.config/crewsync/session.json"
crewsync-server devices "$HOME/crewsync-pilot/server.toml"
crewsync-server revoke "$HOME/crewsync-pilot/server.toml" ID_DO_DISPOSITIVO
```

Revogar um dispositivo bloqueia sua próxima operação. Desativar o usuário bloqueia todos os seus dispositivos.

## Sincronizar

```bash
crewsync-client sync "$HOME/.config/crewsync/session.json" Obras "$HOME/CrewSync-Obras"
crewsync-client watch "$HOME/.config/crewsync/session.json" Obras "$HOME/CrewSync-Obras" 5
```

`sync` executa uma reconciliação. `watch` reage a eventos locais e reconcilia pelo menos a cada cinco segundos, cobrindo eventos perdidos e mudanças remotas. Eventos têm uma janela de 200 ms para agrupar gravações. Se o observador falhar, permanece a reconciliação periódica. Falhas transitórias usam atraso limitado; falhas permanentes encerram o processo. Ctrl-C encerra.

Não execute duas instâncias para a mesma pasta. Movimentações são exclusão/criação, reaproveitando conteúdo igual. Arquivos são transferidos em streaming, verificados por SHA-256 e publicados por temporários e renomeação. Não há retomada parcial.

## Conflitos

Mudanças concorrentes nunca devem sobrescrever silenciosamente uma versão local. O conteúdo remoto ocupa o caminho sincronizado e a versão local concorrente fica em `.crewsync/conflicts/`, acompanhada de JSON com o caminho original. Inspecione e copie o conteúdo desejado de volta com outro nome para resolver.

`.crewsync/trash/` preserva cópias locais substituídas ou excluídas. Essas cópias locais não são limpas automaticamente. Não remova `state.json`: ele registra o que cada cliente confirmou, incluindo exclusões. Não edite a identidade da sessão para apontar um cliente antigo para outro servidor.

## Historico

```bash
crewsync-client history "$HOME/.config/crewsync/session.json" Obras planta.txt
crewsync-client restore "$HOME/.config/crewsync/session.json" Obras planta.txt 1 3
```

A lista está em ordem crescente e inclui o estado atual. No exemplo, 1 é a revisão desejada e 3 é a revisão atual do caminho. Use os números reais da consulta. A restauração cria uma nova revisão; execute `sync` em ambos os computadores ou aguarde `watch`. Base obsoleta é recusada: consulte novamente e revise a alteração concorrente.

Somente `write` restaura. Escolha uma versão de arquivo ou diretório, nunca uma revisão de exclusão. Recuperação de árvore é individual: pais primeiro, filhos depois. Versões já sobrescritas antes da implementação do histórico não podem ser reconstruídas. Rename mantém versões no caminho antigo.

## Manutencao

Com o servidor parado, simule a política configurada e confira o espaço recuperável:

```bash
crewsync-server maintain "$HOME/crewsync-pilot/server.toml"
crewsync-server maintain "$HOME/crewsync-pilot/server.toml" --apply
```

Sem `--apply`, nenhum histórico ou objeto é removido. O relatório informa objetos, bytes, versões a podar e espaço recuperável. `--apply` aplica a retenção e remove somente objetos sem referências em todos os compartilhamentos, inclusive os retirados da configuração. O bloqueio exclusivo impede executar durante transferências ou outro processo de armazenamento.

Por padrão são conservadas até dez versões anteriores com menos de trinta dias desde que foram arquivadas. A última versão de arquivo excluído fica recuperável na lixeira do servidor por trinta dias, mesmo se a janela de histórico for menor. A versão atual e os registros de exclusão continuam no manifesto. Histórico legado sem data recebe prazo de graça na primeira manutenção aplicada. Relógio que recua não acelera exclusões.

A coleta publica primeiro os metadados podados de forma durável; só depois remove objetos. Falha parcial pode deixar órfãos, que uma nova manutenção resolve. Acompanhe espaço livre e faça backup antes de aplicar. As cópias locais em `trash` e `conflicts` não participam dessa coleta.

## Backup

Pare o servidor e use um destino novo fora da raiz interna:

```bash
crewsync-server backup "$HOME/crewsync-pilot/server.toml" /caminho/backup-novo
crewsync-server restore /caminho/backup-novo /caminho/restauracao-nova
```

O backup bloqueia armazenamento e autenticação, copia configuração e estado completo, valida as referências e grava `backup.json` por último. Ausência desse marcador indica cópia incompleta. Restauração exige destino novo e produz `restauracao-nova/store`; não sobrescreve um servidor existente. Valide a configuração e aponte `storage_root` para essa raiz antes de iniciar.

Certificados e chaves TLS ficam fora da raiz interna: copie-os separadamente, junto das cópias locais de conflitos que desejar conservar. Armazene o backup em outra mídia, protegendo as credenciais. Faça um ensaio de restauração com dados sintéticos.

Um backup antigo pode ter menos revisões que os clientes. Para recuperação de desastre, pare todos os clientes, preserve suas pastas e metadados para inspeção, configure novas pastas locais e ative novos dispositivos. Não conecte automaticamente clientes antigos ao servidor restaurado; isso pode interromper a reconciliação por recuo de revisão. O histórico não substitui backup.

## Servicos e atualizacao

Os exemplos `config/systemd/crewsync-server.service` e `crewsync-client.service` são unidades de usuário. Adapte os caminhos, copie para `$HOME/.config/systemd/user/` e execute:

```bash
systemctl --user daemon-reload
systemctl --user enable --now crewsync-server.service
systemctl --user enable --now crewsync-client.service
journalctl --user -u crewsync-server.service -f
```

Logs registram status e tamanho de respostas, sem token, convite, usuário, caminho ou conteúdo de documentos. Falhas esperadas retornam códigos de saída diferentes de zero. Não capture a saída do comando `invite` em logs públicos.

Para atualizar: pare os serviços, faça backup, instale os novos binários e confira `--version` e `check-config`; inicie o servidor e depois os clientes. Mantenha configuração, sessão e raiz interna. Não rebaixe para binários que não conhecem o formato de histórico atualizado. Se a nova versão falhar, preserve os dados e restaure uma cópia isolada do backup para diagnóstico.

Para desinstalar, pare/desabilite as unidades, remova os binários instalados e as unidades copiadas. Não remova a raiz interna ou as pastas locais sem backup e conferência.

## Diagnostico

- TLS recusado: confira CA, IP/hostname, relógio, validade e correspondência entre chave e certificado. Certificados usados como CA não devem ser usados como certificado final de servidor.
- Falha antes de `HTTPS ready`: confira configuração, porta em uso, permissão 0700 da raiz e outro processo com lock. Não apague o arquivo de lock para contornar um servidor ativo.
- HTTP 401/403: confira ativação/revogação, usuário habilitado e permissão no compartilhamento.
- HTTP 409: outro cliente alterou a revisão; reconcilie ou revise a restauração antes de tentar novamente.
- Disco cheio: interrompa novas gravações, libere espaço externo ao armazenamento, faça backup e só então considere manutenção. Nunca apague objetos raw manualmente.
- Sem mudanças aparentes: confira compartilhamento, pasta local, credencial fora da pasta e saída do comando. Links e arquivos especiais são recusados.

## Limites

Linux inicial; Windows, interface gráfica e Android são trabalho posterior. O piloto básico em dois computadores foi confirmado pelo usuário; testes físicos ampliados de queda de energia, disco cheio e medições entre máquinas ainda precisam ocorrer. Há relatórios sintéticos locais, sem promessa de desempenho para seus documentos.

Servidor serializado para rede privada. Sem quotas totais, compactação, retomada parcial ou limpeza automática das cópias locais. Manifestos e histórico por caminho têm limite de 16 MiB; manifestos aceitam até cem mil entradas. Raiz e arquivos internos não devem ser editados externamente. Não há lançamento estável anunciado.

## Licenca

A licença de distribuição aguarda definição do mantenedor. O repositório é público; a automação mantém a publicação de release condicionada à presença de uma licença. Consulte o repositório para o estado atualizado.

## Contribuir

[GitHub](https://github.com/goncalvesl/crewsync) reúne código, issues e plano. Leia `AGENTS.md` e `docs/WORKFLOW.md`, use dados sintéticos e execute `scripts/check-pilot.sh` antes de enviar alterações. Nunca inclua tokens, chaves ou documentos reais.
