# Piloto Linux em dois computadores

O código agora oferece servidor HTTPS, ativação por dispositivo, sincronização
manual e reconciliação periódica. Use dois computadores Linux com dados
sintéticos em pastas novas. Este é um piloto em rede privada; ainda não é uma
versão estável ou uma substituição de backup.

A máquina 1 pode executar servidor e cliente; a máquina 2 executa o segundo
cliente. A sincronização passa sempre pelo servidor. Windows e Android não estão
incluídos nesta entrega.

## 1. Compilar e validar

Pré-requisitos nas duas máquinas: Linux com kernel >= 5.6 e `/proc` montado,
`/usr/bin/python3` >= 3.11 com `ssl` e `tomllib`, GNU coreutils com
`/usr/bin/sha256sum`. Para compilar, Rust stable com rustfmt/Clippy. OpenSSL CLI
é necessário na máquina 1 para criar os certificados e para os testes TLS.
Sem pacotes Python externos. A ponte HTTPS está embarcada nos binários.

Copie o código para as máquinas ou use um checkout do mesmo estado. Na raiz:

```bash
cargo build --release --workspace --locked
mkdir -p "$HOME/.local/bin"
install -m 755 target/release/crewsync-server target/release/crewsync-client "$HOME/.local/bin/"
```

Os exemplos usam esses binários pelo caminho completo, sem depender do PATH.
Compile em cada máquina se as distribuições/versões de glibc forem diferentes.
Não há download publicado, tag de release ou licença atribuída nesta entrega.

Antes do piloto, execute no terminal normal de uma máquina com sockets liberados:

```bash
scripts/check-pilot.sh
```

Este comando inclui o teste HTTPS com servidor e dois clientes em processos
separados. Inicialmente, o sandbox da sessão de implementação proibiu criar qualquer socket
(`Operation not permitted`), inclusive local. Por isso os dois testes de rede
estão marcados `ignored` por padrão e são executados explicitamente pelo script
e pela CI. Após atualizar as permissões da sessão, os dois testes HTTPS passaram.
Também passaram os testes de disco, autenticação, worker em processo separado,
reconciliação e TLS em memória. Execute a sequência nas máquinas do piloto antes
de usar dados reais.

## 2. Configurar a máquina 1

```bash
mkdir -p "$HOME/crewsync-pilot"
cp config/server.example.toml "$HOME/crewsync-pilot/server.toml"
```

Edite `storage_root` para um caminho absoluto, por exemplo
`/home/SEU_USUARIO/crewsync-pilot/store`. O pai precisa existir; o servidor criará
a raiz privada 0700. Não use a pasta dos clientes como armazenamento interno.
O exemplo habilita `ana` com escrita e `bruno` com leitura em `Obras`.

```bash
"$HOME/.local/bin/crewsync-server" check-config "$HOME/crewsync-pilot/server.toml"
```

Descubra o IPv4 da máquina 1 na rede local. Nos próximos comandos,
**substitua `192.168.1.10` pelo endereço real e mantenha-o fixo durante o piloto**.
O certificado precisa incluir esse mesmo endereço.

```bash
scripts/pilot-cert.sh "$HOME/crewsync-pilot/tls" 192.168.1.10
```

O script cria uma CA e um certificado de servidor. Copie **somente `ca.crt`**
para os clientes por um meio confiável; mantenha `ca.key` e `server.key` privados
na máquina 1. Os certificados de servidor duram 90 dias. Não existe opção de
ignorar validação TLS; endereço incorreto ou CA não confiável causam erro.

Inicie o servidor num terminal dedicado:

```bash
"$HOME/.local/bin/crewsync-server" serve "$HOME/crewsync-pilot/server.toml" 0.0.0.0 8443 "$HOME/crewsync-pilot/tls/server.crt" "$HOME/crewsync-pilot/tls/server.key"
```

Aguarde `CrewSync HTTPS ready 8443`. `0.0.0.0` escuta nas interfaces IPv4;
clientes usam o IP real, nunca `0.0.0.0`. Para parar em primeiro plano, Ctrl-C.
O servidor atende uma operação por vez, com buffers de 64 KiB e timeout de socket
de 30 segundos. Não exponha a porta na Internet durante este piloto.

Se o servidor usa UFW ativo e a rede é `192.168.1.0/24`, permita apenas a rede local:

```bash
sudo ufw allow from 192.168.1.0/24 to any port 8443 proto tcp
```

Adapte à rede e ao firewall reais. Essa regra não remove restrições de um
sandbox de desenvolvimento. Não é necessário abrir porta nos clientes.

### Diagnóstico de inicialização

Se o servidor encerrar antes de mostrar `CrewSync HTTPS ready`, o diagnóstico
indica certificado/chave ausentes ou inválidos, porta em uso ou falha do worker
(configuração, raiz privada, estado de autenticação ou armazenamento).
Não imprime conteúdo de configuração, tokens ou chaves.

`check-config` valida o esquema, mas não as permissões da raiz. Uma pasta já
existente usada como `storage_root` deve ser privada. Confira o caminho configurado
e aplique `chmod 700 /caminho/da/raiz-interna` somente nessa raiz do serviço, sem
alterar recursivamente pastas compartilhadas. O servidor não muda permissões
existentes automaticamente. Se o armazenamento estiver em uso, não apague o
arquivo `lock`: use a instância existente ou encerre-a antes de reiniciar.

## 3. Ativar cada cliente

Em cada máquina, prepare a pasta privada de configuração e coloque a cópia
confiável da CA em `$HOME/.config/crewsync/ca.crt`:

```bash
mkdir -p "$HOME/.config/crewsync"
chmod 700 "$HOME/.config/crewsync"
mkdir -p "$HOME/CrewSync-Obras"
```

A pasta de sincronização deve ser nova/vazia para o primeiro ensaio.
A credencial deve ficar **fora** dela; o cliente recusa credencial dentro da
pasta que seria enviada ao servidor.

Em outro terminal da máquina 1, gere um convite para o primeiro dispositivo:

```bash
"$HOME/.local/bin/crewsync-server" invite "$HOME/crewsync-pilot/server.toml" ana 600
```

O comando mostra um código secreto válido por 600 segundos e por uma ativação.
Ele é a única saída deliberada desse segredo: não o coloque em scripts, logs ou
argumentos de comandos. Na máquina do cliente, execute:

```bash
"$HOME/.local/bin/crewsync-client" activate https://192.168.1.10:8443 "$HOME/.config/crewsync/ca.crt" "$HOME/.config/crewsync/session.json" pc1
```

Cole o código de convite na entrada do programa e pressione Enter. O cliente
mostra `Ativando dispositivo no servidor HTTPS...` enquanto aguarda a resposta.
Não é necessário Ctrl-D. Em binários antigos que ainda pedem Ctrl-D, finalize a
entrada com Ctrl-D ou atualize o cliente.
A credencial é salva em arquivo 0600 e não é impressa. Para a máquina 2, gere
**outro convite** e repita a ativação, usando `pc2` como nome do dispositivo.
Não copie `session.json` entre computadores. Se a ativação perder a resposta,
pode ser necessário emitir outro convite; convites consumidos não são reutilizados.

Confira o acesso em cada cliente:

```bash
"$HOME/.local/bin/crewsync-client" shares "$HOME/.config/crewsync/session.json"
```

## 4. Sincronizar e testar

Na máquina 1, crie apenas arquivos sintéticos na pasta de teste e execute:

```bash
printf 'teste inicial\n' > "$HOME/CrewSync-Obras/piloto.txt"
mkdir "$HOME/CrewSync-Obras/vazia"
"$HOME/.local/bin/crewsync-client" sync "$HOME/.config/crewsync/session.json" Obras "$HOME/CrewSync-Obras"
```

Na máquina 2, execute o mesmo comando `sync`. Confirme que `piloto.txt` e o
diretório vazio chegaram. Edite um arquivo na máquina 2, sincronize-a e depois
sincronize a máquina 1. Repita sem mudanças: o relatório deve indicar zero envios
e zero recebimentos. Arquivos permanecem raw, sem compactação.

Teste também, com monitoramento parado:

1. **Conflito:** após ambas sincronizarem, edite `piloto.txt` de formas diferentes
   nas duas máquinas. Sincronize a 1, depois a 2. A versão remota fica no caminho
   original; a versão concorrente da 2 fica em `.crewsync/conflicts/`, junto de
   um JSON que registra o caminho original. O relatório informa o conflito.
2. **Desconexão/exclusão:** deixe a 2 sem executar o cliente, remova somente um
   arquivo sintético na 1 e sincronize a 1. Reconecte/sincronize a 2: o arquivo
   não deve ressurgir. Se a 2 o editou offline, sua versão fica em `conflicts`.
3. **Movimentação e pastas:** renomeie um arquivo/pasta, crie diretórios vazios e
   sincronize ambas. Movimentação é exclusão/criação com deduplicação do conteúdo.
4. **Somente leitura:** ative outro dispositivo com convite para `bruno`.
   Downloads funcionam; editar/enviar/excluir é recusado, conservando o conteúdo
   local. Copie a edição para fora da árvore ou ajuste o acesso antes de continuar.
5. **Interrupção:** interrompa um envio de arquivo sintético grande; reinicie o
   servidor/cliente e reconcilie. Nenhum arquivo parcial pode ser confirmado.
6. **Revogação:** na máquina 1, liste e revogue pelo ID público do dispositivo:

```bash
"$HOME/.local/bin/crewsync-server" devices "$HOME/crewsync-pilot/server.toml"
"$HOME/.local/bin/crewsync-server" revoke "$HOME/crewsync-pilot/server.toml" ID_DO_DISPOSITIVO
```

A próxima operação desse token deve ser recusada; os outros dispositivos
continuam funcionando. Desativar o usuário em TOML bloqueia todos os seus
dispositivos. O worker tenta recarregar a configuração antes de cada requisição;
recarga inválida mantém a última configuração válida.

## 5. Reconciliação periódica

Em cada cliente, depois dos testes manuais:

```bash
"$HOME/.local/bin/crewsync-client" watch "$HOME/.config/crewsync/session.json" Obras "$HOME/CrewSync-Obras" 5
```

O cliente reconcilia a cada 5 segundos. Erros transitórios usam atraso crescente
limitado a 60 segundos; credencial revogada, certificado inválido e erros de
configuração interrompem o processo. Ctrl-C encerra; temporários são recuperados
na próxima execução. Há varredura periódica, sem observador de eventos ainda.
Não execute duas instâncias para a mesma pasta.

Exemplos opcionais de unidades de usuário estão em
[config/systemd](../config/systemd/). Adapte os caminhos, copie as unidades para
`$HOME/.config/systemd/user/` e use `systemctl --user daemon-reload` e
`systemctl --user enable --now crewsync-server.service` (ou `crewsync-client.service`).
As unidades limitam reinícios e encerram também os processos da ponte. Linger
e execução antes do login dependem da administração do computador; não são
configurados automaticamente.

## Recuperação, backup e limites

- `.crewsync/state.json` é o baseline, vinculado ao endereço do servidor e ao
  compartilhamento. Não edite ou remova durante operações e não altere a URL da
  credencial para outro servidor. Recuo/perda de revisões remotas causa erro.
- `.crewsync/trash/` guarda arquivos e diretórios locais substituídos/excluídos;
  `.crewsync/conflicts/` guarda as versões concorrentes. Os JSONs ao lado de cada
  cópia registram seu caminho original. Inspecione e copie o conteúdo desejado de
  volta com outro nome para sincronizar uma resolução. Não sincronizar os JSONs.
- Não há limpeza automática dessas cópias nem de objetos órfãos do servidor:
  acompanhe espaço livre. Disco cheio deve produzir erro; faça backup antes de
  qualquer limpeza manual. Não apagar objetos raw individualmente.
- Para backup consistente inicial, pare o servidor e os clientes, copie a raiz
  interna completa do servidor, configuração e certificados para armazenamento
  separado; preserve permissões. Copie também `.crewsync/conflicts` e `trash` dos
  clientes. Restaure com processos parados; use cópias novas para ensaiar restauração,
  sem conectar clientes antigos contra um servidor que perdeu revisões.
- Links simbólicos e arquivos especiais abortam a reconciliação. Nomes precisam
  seguir as regras em [STORAGE.md](STORAGE.md); `.crewsync` no primeiro componente
  é reservado e a profundidade máxima é 128 componentes.
- Não há histórico consultável/restauração pelo servidor, retenção configurável,
  coleta de objetos, quotas totais, métricas ou retomada parcial de transferências.
  A etapa 5 e o relatório real de desempenho/piloto continuam abertos.
- Não foi validado aqui comportamento real em disco cheio, queda de energia,
  duas máquinas físicas ou rede entre máquinas. Valide primeiro os cenários acima com dados
  descartáveis e registre distribuição, kernel, volume de arquivos e resultados.
