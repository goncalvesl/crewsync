# Manual do CrewSync

O CrewSync mantém uma pasta atualizada entre seus computadores. Seu objetivo é ser simples de instalar, manter e usar, além de rápido.

Para começar como usuário, siga Instalar, Dispositivos e Sincronizar. A preparação do servidor é feita uma vez pelo administrador. A explicação técnica fica na última seção.

Esta é uma versão de teste para Linux em rede privada. Use primeiro arquivos de teste e mantenha backup separado.

## Instalar

Baixe o pacote para seu computador: `x86_64` para Intel/AMD ou `aarch64` para ARM64. Consulte [versões publicadas](https://github.com/goncalvesl/crewsync/releases). Enquanto não houver release, os [pacotes de teste](https://github.com/goncalvesl/crewsync/actions/workflows/release.yml) ficam nos artefatos das execuções concluídas do GitHub.

Extraia o pacote e, dentro da pasta extraída, execute:

```bash
./install.sh --client
```

Pronto: o cliente está em `$HOME/.local/bin`. Não precisa instalar Rust, Cargo ou Python, nem usar sudo. Os comandos deste manual usam o caminho completo; não é necessário configurar o PATH.

No computador que vai servir os arquivos, use `./install.sh --server`. Se quiser os dois programas na mesma máquina, execute apenas `./install.sh`. O instalador também guarda o manual em `$HOME/.local/share/crewsync/MANUAL.md`.

## Dispositivos

Peça ao administrador três coisas: o endereço do servidor, o arquivo `ca.crt` e um convite de ativação próprio para este computador.

Crie a pasta de configuração e uma pasta nova para seus arquivos:

```bash
mkdir -p "$HOME/.config/crewsync" "$HOME/CrewSync-Obras"
chmod 700 "$HOME/.config/crewsync"
```

Salve `ca.crt` em `$HOME/.config/crewsync/`. Ative o computador usando o endereço recebido; abaixo, troque `192.168.1.10` pelo IP correto:

```bash
"$HOME/.local/bin/crewsync-client" activate https://192.168.1.10:8443 "$HOME/.config/crewsync/ca.crt" "$HOME/.config/crewsync/session.json" pc1
```

Cole o convite e pressione Enter. Em outro computador, use outro convite e outro nome, como `pc2`. Não copie `session.json` entre computadores ou para dentro da pasta sincronizada.

## Sincronizar

Para sincronizar uma vez:

```bash
"$HOME/.local/bin/crewsync-client" sync "$HOME/.config/crewsync/session.json" Obras "$HOME/CrewSync-Obras"
```

Para continuar acompanhando as mudanças:

```bash
"$HOME/.local/bin/crewsync-client" watch "$HOME/.config/crewsync/session.json" Obras "$HOME/CrewSync-Obras" 5
```

Deixe o comando aberto; Ctrl-C encerra. Faça isso nos dois computadores. Troque `Obras` e a pasta local se o administrador indicar outros nomes. Não execute dois clientes ao mesmo tempo na mesma pasta.

Para conferir quais pastas você pode acessar:

```bash
"$HOME/.local/bin/crewsync-client" shares "$HOME/.config/crewsync/session.json"
```

## Conflitos

Se duas pessoas alterarem o mesmo arquivo, o CrewSync preserva a cópia concorrente em `.crewsync/conflicts/`. Confira as versões e copie o conteúdo que deseja manter de volta à pasta, de preferência com outro nome.

Arquivos locais substituídos ou excluídos podem ser encontrados em `.crewsync/trash/`. Essas cópias não são limpas automaticamente. Não apague ou edite os outros arquivos de controle em `.crewsync/`.

## Historico

Para ver as versões de um arquivo:

```bash
"$HOME/.local/bin/crewsync-client" history "$HOME/.config/crewsync/session.json" Obras planta.txt
```

A última linha mostra o estado atual. Para recuperar uma versão, informe o número desejado e o número atual. Exemplo: recuperar a versão 1 quando a atual é 3:

```bash
"$HOME/.local/bin/crewsync-client" restore "$HOME/.config/crewsync/session.json" Obras planta.txt 1 3
```

Use os números reais da consulta. Depois execute `sync` nos dois computadores ou aguarde `watch`. É preciso ter permissão de escrita. Se outra pessoa já mudou o arquivo, consulte o histórico novamente antes de repetir.

## Configurar

Esta parte é para o administrador do servidor. Instale com `./install.sh --server` e prepare a configuração:

```bash
mkdir -p "$HOME/crewsync-pilot"
cp "$HOME/.local/share/crewsync/server.example.toml" "$HOME/crewsync-pilot/server.toml"
```

Edite `server.toml`. Em `storage_root`, coloque um caminho absoluto como `/home/SEU_USUARIO/crewsync-pilot/store`. Essa é uma pasta interna do serviço, separada das pastas dos clientes; o servidor a cria quando necessário.

O exemplo permite que `ana` altere a pasta `Obras` e que `bruno` apenas leia. Ajuste os nomes e as permissões da equipe: `read` para leitura, `write` para leitura e alteração. Usuários com `enabled = false` ficam bloqueados.

Confira a configuração:

```bash
"$HOME/.local/bin/crewsync-server" check-config "$HOME/crewsync-pilot/server.toml"
```

## HTTPS e rede

Descubra o IP do servidor na rede e mantenha esse endereço fixo. O comando abaixo prepara os certificados e exige OpenSSL instalado apenas nessa preparação. Troque o IP pelo real:

```bash
"$HOME/.local/share/crewsync/pilot-cert.sh" "$HOME/crewsync-pilot/tls" 192.168.1.10
"$HOME/.local/bin/crewsync-server" serve "$HOME/crewsync-pilot/server.toml" 0.0.0.0 8443 "$HOME/crewsync-pilot/tls/server.crt" "$HOME/crewsync-pilot/tls/server.key"
```

Aguarde `CrewSync HTTPS ready 8443`. Entregue aos clientes somente o arquivo `ca.crt`. Mantenha os arquivos `.key` privados no servidor. O certificado vence em 90 dias; renove antes disso.

Se usar UFW, permita a porta TCP do servidor para a rede local correta. Exemplo para `192.168.1.0/24`:

```bash
sudo ufw allow from 192.168.1.0/24 to any port 8443 proto tcp
```

Não é necessário abrir portas nos clientes. O endereço que eles usam deve ser o IP real, nunca `0.0.0.0`.

Para gerar um convite válido por dez minutos:

```bash
"$HOME/.local/bin/crewsync-server" invite "$HOME/crewsync-pilot/server.toml" ana 600
```

Para listar ou bloquear um dispositivo:

```bash
"$HOME/.local/bin/crewsync-server" devices "$HOME/crewsync-pilot/server.toml"
"$HOME/.local/bin/crewsync-server" revoke "$HOME/crewsync-pilot/server.toml" ID_DO_DISPOSITIVO
```

Cada convite vale para uma ativação. Se perder a resposta da ativação, gere outro. Desativar um usuário na configuração bloqueia todos os seus dispositivos.

## Manutencao

A configuração de exemplo mantém até dez versões anteriores por até trinta dias; o último conteúdo de um arquivo excluído fica recuperável por trinta dias. Você pode mudar esses valores em `[server.retention]`.

Antes de limpar, faça backup e pare o servidor. Primeiro veja o que seria removido:

```bash
"$HOME/.local/bin/crewsync-server" maintain "$HOME/crewsync-pilot/server.toml"
```

Se concordar com o relatório, aplique a limpeza e inicie o servidor novamente:

```bash
"$HOME/.local/bin/crewsync-server" maintain "$HOME/crewsync-pilot/server.toml" --apply
```

Sem `--apply`, nada é removido. A limpeza não apaga a versão atual nem as cópias locais em `conflicts` e `trash`. Nunca apague manualmente os arquivos internos do servidor.

## Backup

Com o servidor parado, crie uma cópia em uma pasta nova, preferencialmente em outra mídia:

```bash
"$HOME/.local/bin/crewsync-server" backup "$HOME/crewsync-pilot/server.toml" /caminho/backup-novo
```

Copie também a pasta de certificados separadamente e proteja as chaves. Para ensaiar a recuperação, use outra pasta nova:

```bash
"$HOME/.local/bin/crewsync-server" restore /caminho/backup-novo /caminho/restauracao-nova
```

A raiz recuperada fica em `/caminho/restauracao-nova/store`. Ajuste `storage_root` em uma cópia da configuração e confira antes de iniciar. Um backup antigo exige cuidado: preserve as pastas dos clientes e use pastas novas e novos convites ao recuperar o serviço. Os detalhes estão no final deste manual.

## Servicos e atualizacao

Para atualizar, pare os programas, faça backup e execute o instalador do novo pacote. Depois inicie o servidor e os clientes. Não apague suas configurações ou pastas. Não volte a uma versão antiga que desconheça o formato do histórico atual.

Para iniciar automaticamente, o pacote inclui exemplos de serviços em `config/systemd/`. A instalação e os comandos estão na seção técnica. O instalador não inicia serviços sozinho.

## Diagnostico

- Não conecta: confira se o servidor está aberto, o endereço está correto e a porta está liberada.
- Erro de certificado: confira o arquivo `ca.crt`, o IP, a data dos computadores e a validade do certificado.
- Acesso negado: peça ao administrador para conferir seu usuário, dispositivo e permissão na pasta.
- Conflito de revisão: sincronize ou consulte o histórico novamente antes de tentar recuperar uma versão.
- Armazenamento em uso: encerre a outra instância. Não apague o arquivo `lock`.
- Disco cheio: pare novas gravações e libere espaço fora da pasta interna. Faça backup antes de limpar versões.

## Limites

O piloto básico em dois computadores foi concluído. A validação física ampliada continua pendente e ainda não há versão estável publicada. Windows, interface gráfica e Android são etapas futuras. O histórico não substitui backup.

## Licenca

CrewSync é software livre sob a GNU GPL versão 3 somente (`GPL-3.0-only`). O texto integral está no arquivo `LICENSE` do pacote e no [repositório](https://github.com/goncalvesl/crewsync/blob/main/LICENSE). Dependências mantêm suas próprias licenças e avisos.

## Detalhes tecnicos

Esta seção é opcional. Ela explica o funcionamento e reúne procedimentos avançados para administração e desenvolvimento.

### Arquitetura e sincronização

O servidor central ordena as revisões de cada compartilhamento. Os clientes comparam o conteúdo local, o estado remoto e a última revisão confirmada. Alterações concorrentes são preservadas como conflitos; registros de exclusão impedem que clientes antigos recriem arquivos apagados. Mover um arquivo é tratado como exclusão e criação, com reaproveitamento de conteúdo igual.

`watch` reage a eventos locais, agrupa gravações em uma janela de 200 ms e faz uma reconciliação completa pelo menos a cada intervalo configurado. A verificação periódica cobre eventos perdidos e alterações remotas. Falhas transitórias usam atraso crescente limitado; erros permanentes encerram o processo.

### Integridade e armazenamento

Arquivos são identificados por SHA-256, recebidos em streaming e armazenados sem compactação. Conteúdo igual compartilha um objeto, inclusive entre compartilhamentos. Publicação usa temporários, verificação de hash, renomeação e sincronização em disco antes da confirmação.

A raiz interna exige permissão 0700. Ela contém `objects`, `manifests`, `history`, `auth` e `tmp`. Não a edite externamente. `state.json` no cliente é o registro das revisões confirmadas, vinculado à identidade do servidor e ao compartilhamento; não o remova para tentar resolver um erro.

Linux kernel 5.6 ou superior, `/proc` montado e filesystem local com suporte a `fsync` e renomeação atômica são necessários. Não usar NFS como raiz interna. Links simbólicos e arquivos especiais são recusados. Caminhos têm até 4096 bytes e 128 componentes. Manifestos e histórico por caminho têm limite de 16 MiB; manifestos aceitam até cem mil entradas. Exceder os limites recusa a alteração.

### Segurança e configuração

HTTPS usa rustls com TLS 1.2/1.3, CA explícita e validação do endereço; não existe opção de ignorar o certificado. O servidor atende uma operação por vez, com timeout de conexão de 30 segundos, para rede privada. Tokens são individuais por dispositivo e somente seus hashes ficam no servidor.

Somente pastas de primeiro nível são compartilhamentos; subpastas herdam o acesso. Ausência de permissão nega acesso. A configuração TOML é estrita e recarregada antes de cada requisição; recarga inválida mantém a última válida. Trocar a raiz exige reiniciar. `max_upload_bytes` limita cada envio, não o armazenamento total.

### Histórico, retenção e recuperação

Restaurar exige a revisão desejada e a revisão atual do caminho, criando uma nova revisão. Uma revisão de exclusão não é origem de restauração; escolha uma versão anterior. Para recuperar uma árvore, restaure os diretórios pais antes dos arquivos. Rename mantém versões no caminho antigo. Versões sobrescritas antes da implementação do histórico não podem ser reconstruídas.

A idade de retenção começa quando a versão deixa de ser atual. Histórico legado sem data recebe prazo de graça na primeira manutenção aplicada. Recuo do relógio não acelera exclusões. A manutenção exige o lock exclusivo com o servidor parado, conserva referências de todos os compartilhamentos e publica metadados duráveis antes de remover objetos. Os registros atuais de exclusão permanecem para clientes offline.

Backup bloqueia armazenamento e autenticação, valida a cópia e grava `backup.json` por último; sem esse marcador, a cópia está incompleta. Restauração exige destino novo e só grava `restored.json` após validar. Certificados e cópias locais de conflitos ficam fora desse backup e devem ser copiados separadamente.

Restaurar um backup antigo pode recuar revisões. Nesse caso, pare todos os clientes, preserve suas pastas e metadados para inspeção e configure novas pastas locais e novos dispositivos. Não reconecte automaticamente clientes antigos a um servidor que perdeu revisões.

### Serviços, logs e desinstalação

Adapte os caminhos nos exemplos de `config/systemd/`, copie as unidades desejadas para `$HOME/.config/systemd/user/` e execute somente as correspondentes à função desta máquina:

```bash
systemctl --user daemon-reload
systemctl --user enable --now crewsync-server.service
systemctl --user enable --now crewsync-client.service
journalctl --user -u crewsync-server.service -f
```

Logs de requisição registram código e tamanho da resposta, sem tokens, convites, usuário, caminho ou conteúdo. Não capture a saída de `invite` em logs públicos. Para desinstalar, pare e desabilite os serviços e remova os binários e arquivos instalados. Preserve os dados e a configuração até conferir o backup.

### Pacotes, compilação e contribuição

Os pacotes Linux musl estáticos dispensam glibc, Cargo, Python e OpenSSL no uso diário. OpenSSL CLI é necessário apenas para gerar certificados pelo script. `PREFIX` permite outro destino de instalação, como `PREFIX=/caminho/local ./install.sh --client`. O instalador não altera o PATH ou configurações existentes.

Para conferir o pacote baixado, coloque seu arquivo `.sha256` na mesma pasta e execute, substituindo o nome:

```bash
sha256sum -c crewsync-VERSAO-ALVO.tar.gz.sha256
```

Para compilar, instale Rust stable e compilador C, clone o repositório e execute:

```bash
git clone https://github.com/goncalvesl/crewsync.git
cd crewsync
cargo build --release --workspace --locked
```

Os executáveis ficam em `target/release/`. Consulte o [repositório](https://github.com/goncalvesl/crewsync) para documentação de desenvolvimento, plano e testes. Leia `AGENTS.md` e `docs/WORKFLOW.md` antes de contribuir. Use dados sintéticos e execute `scripts/check-pilot.sh`; nunca inclua credenciais ou documentos reais.

Rapidez é um objetivo medido por ensaios reproduzíveis, sem promessa universal para os arquivos da equipe. Ainda não há quotas totais, compactação, retomada parcial ou limpeza automática das cópias locais. Resultados localhost não substituem medições físicas entre computadores.
