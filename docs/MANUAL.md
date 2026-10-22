# Manual do CrewSync

O CrewSync mantém uma pasta atualizada entre seus computadores. Seu objetivo é ser simples de instalar, manter e usar, além de rápido.

Para começar, siga Instalar e Sincronizar. A preparação do servidor é feita uma vez pelo administrador. A explicação técnica fica na última seção.

Esta é uma versão de teste para Linux, com servidor remoto ou local. Use primeiro arquivos de teste e mantenha backup separado.

## Instalar

No servidor, execute este comando e responda às perguntas:

```bash
curl -fsSL https://raw.githubusercontent.com/goncalvesl/crewsync/main/scripts/setup-online.sh | sh -s -- server
```

Nos outros computadores, execute:

```bash
curl -fsSL https://raw.githubusercontent.com/goncalvesl/crewsync/main/scripts/setup-online.sh | sh -s -- client
```

O assistente baixa a última release, identifica Intel/AMD ou ARM64 e pergunta onde instalar. No servidor, pergunta o domínio ou IPv4 de acesso, cria o certificado, configura o primeiro usuário e compartilhamento, inicia o serviço e mostra um convite. No cliente, ativa o computador e inicia a sincronização.

Tenha `curl`, `tar` e `sha256sum` instalados; no servidor, também OpenSSL. Não precisa de Rust, Cargo ou Python. Os comandos funcionam no terminal de uma sessão Linux, sem sudo. Os diretórios sugeridos podem ser alterados. Escolha diretórios novos para a configuração e os dados internos do servidor; instalações existentes são preservadas.

Se preferir baixar e conferir o pacote manualmente:

<!-- downloads:start -->
Versão atual: 0.1.0-alpha.4 (alpha, para testes).

- [Linux Intel/AMD (x86_64) — baixar pacote](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.4/crewsync-0.1.0-alpha.4-linux-x86_64.tar.gz) · [SHA-256](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.4/crewsync-0.1.0-alpha.4-linux-x86_64.tar.gz.sha256)
- [Linux ARM64 (aarch64) — baixar pacote](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.4/crewsync-0.1.0-alpha.4-linux-arm64.tar.gz) · [SHA-256](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.4/crewsync-0.1.0-alpha.4-linux-arm64.tar.gz.sha256)

[Notas da versão](https://github.com/goncalvesl/crewsync/releases/tag/v0.1.0-alpha.4) · [Código-fonte e dependências](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.4/crewsync-0.1.0-alpha.4-source.tar.gz)
<!-- downloads:end -->

Extraia o pacote e execute `./setup.sh server` ou `./setup.sh client` na pasta extraída. O resultado é o mesmo. `./install.sh --client` e `./install.sh --server` continuam disponíveis para instalar somente os programas, sem configurar ou iniciar serviços.

## Dispositivos

Antes de instalar o cliente, peça ao administrador o endereço HTTPS, o arquivo `ca.crt`, o nome da pasta compartilhada e um convite próprio para este computador. Transfira `ca.crt` por um meio confiável. Nunca copie as chaves `.key` nem a sessão de outro computador.

Informe esses dados ao assistente. O convite não aparece enquanto você digita e só pode ser usado uma vez. Para outro computador, gere outro convite.

## Sincronizar

Depois do assistente, a sincronização já fica ativa. Crie um arquivo de teste na pasta local escolhida e confira no outro computador.

Com systemd, consulte o estado:

```bash
systemctl --user status crewsync-client.service
```

O serviço inicia durante sua sessão. Para continuar sem login, execute `sudo loginctl enable-linger SEU_USUARIO` em cada computador que precisar disso. Sem systemd, o assistente inicia um processo em segundo plano e informa o log e o comando para reiniciar após ligar o computador.

Para conferir quais pastas você pode acessar, usando os diretórios sugeridos:

```bash
"$HOME/.local/bin/crewsync-client" shares "$HOME/.config/crewsync-client/session.json"
```

Não execute dois clientes na mesma pasta. Se escolheu outros diretórios, ajuste os caminhos nos exemplos deste manual.

## Conflitos

Se duas pessoas alterarem o mesmo arquivo, o CrewSync preserva a cópia concorrente em `.crewsync/conflicts/`. Confira as versões e copie o conteúdo que deseja manter de volta à pasta, de preferência com outro nome.

Arquivos locais substituídos ou excluídos podem ser encontrados em `.crewsync/trash/`. Essas cópias não são limpas automaticamente. Não apague ou edite os outros arquivos de controle em `.crewsync/`.

## Historico

Para ver as versões de um arquivo:

```bash
"$HOME/.local/bin/crewsync-client" history "$HOME/.config/crewsync-client/session.json" Obras planta.txt
```

A última linha mostra o estado atual. Para recuperar uma versão, informe o número desejado e o número atual. Exemplo: recuperar a versão 1 quando a atual é 3:

```bash
"$HOME/.local/bin/crewsync-client" restore "$HOME/.config/crewsync-client/session.json" Obras planta.txt 1 3
```

Use os números reais da consulta. Depois execute `sync` nos dois computadores ou aguarde `watch`. É preciso ter permissão de escrita. Se outra pessoa já mudou o arquivo, consulte o histórico novamente antes de repetir.

## Configurar

O assistente do servidor cria `$HOME/.config/crewsync-server/server.toml`. Os arquivos internos ficam, por padrão, em `$HOME/.local/share/crewsync-server`; essa pasta é separada da pasta dos clientes e não deve ser editada manualmente.

Para adicionar usuários e pastas, edite `server.toml`. Use `read` para leitura e `write` para leitura, alteração e exclusão. Usuários com `enabled = false` ficam bloqueados. A configuração de exemplo instalada em `$HOME/.local/share/crewsync/server.example.toml` mostra a sintaxe.

Confira depois de editar:

```bash
"$HOME/.local/bin/crewsync-server" check-config "$HOME/.config/crewsync-server/server.toml"
```

## HTTPS e rede

O servidor já fica iniciado após o assistente. Quando ele perguntar o endereço, informe o domínio usado pelos clientes, como `sync.empresa.com`, ou o IPv4 público/VPN. Digite somente o nome ou IP, sem `https://`, porta ou caminho. A porta é perguntada separadamente; o padrão é 8443.

Em servidor remoto, siga estes passos:

- Se usar domínio, aponte o registro DNS A para o IPv4 público do servidor.
- Libere a porta TCP escolhida no firewall do Linux e nas regras de rede do provedor para os clientes.
- Com systemd, execute `sudo loginctl enable-linger SEU_USUARIO` para manter o serviço funcionando sem uma sessão SSH aberta.
- No cliente, informe `https://sync.empresa.com:8443`, usando o domínio e a porta reais.

Não é necessário ter o DNS pronto para gerar o certificado e iniciar o servidor; ele precisa estar correto quando os clientes conectarem. Entregue aos clientes somente `ca.crt`, localizado na pasta `tls` da configuração. Mantenha os arquivos `.key` privados. O certificado vence em 90 dias; renove antes disso, conforme a seção técnica.

Se usar UFW, permita a porta para os endereços dos clientes ou da VPN. Exemplo para clientes na rede `192.168.1.0/24`:

```bash
sudo ufw allow from 192.168.1.0/24 to any port 8443 proto tcp
```

Não é necessário abrir portas nos clientes. O endereço que eles usam deve ser o domínio ou IP de acesso real, nunca `0.0.0.0`. Se o servidor remoto estiver atrás de roteador/NAT, encaminhe a porta até ele.

Para gerar um convite válido por dez minutos:

```bash
"$HOME/.local/bin/crewsync-server" invite "$HOME/.config/crewsync-server/server.toml" ana 600
```

Para listar ou bloquear um dispositivo:

```bash
"$HOME/.local/bin/crewsync-server" devices "$HOME/.config/crewsync-server/server.toml"
"$HOME/.local/bin/crewsync-server" revoke "$HOME/.config/crewsync-server/server.toml" ID_DO_DISPOSITIVO
```

Cada convite vale para uma ativação. Se perder a resposta da ativação, gere outro. Desativar um usuário na configuração bloqueia todos os seus dispositivos.

## Manutencao

A configuração de exemplo mantém até dez versões anteriores por até trinta dias; o último conteúdo de um arquivo excluído fica recuperável por trinta dias. Você pode mudar esses valores em `[server.retention]`.

Antes de limpar, faça backup e pare o servidor. Primeiro veja o que seria removido:

```bash
"$HOME/.local/bin/crewsync-server" maintain "$HOME/.config/crewsync-server/server.toml"
```

Se concordar com o relatório, aplique a limpeza e inicie o servidor novamente:

```bash
"$HOME/.local/bin/crewsync-server" maintain "$HOME/.config/crewsync-server/server.toml" --apply
```

Sem `--apply`, nada é removido. A limpeza não apaga a versão atual nem as cópias locais em `conflicts` e `trash`. Nunca apague manualmente os arquivos internos do servidor.

## Backup

Com o servidor parado, crie uma cópia em uma pasta nova, preferencialmente em outra mídia:

```bash
"$HOME/.local/bin/crewsync-server" backup "$HOME/.config/crewsync-server/server.toml" /caminho/backup-novo
```

Copie também a pasta de certificados separadamente e proteja as chaves. Para ensaiar a recuperação, use outra pasta nova:

```bash
"$HOME/.local/bin/crewsync-server" restore /caminho/backup-novo /caminho/restauracao-nova
```

A raiz recuperada fica em `/caminho/restauracao-nova/store`. Ajuste `storage_root` em uma cópia da configuração e confira antes de iniciar. Um backup antigo exige cuidado: preserve as pastas dos clientes e use pastas novas e novos convites ao recuperar o serviço. Os detalhes estão no final deste manual.

## Servicos e atualizacao

Para atualizar uma instalação existente, pare os serviços e faça backup. Baixe e extraia o novo pacote para sua arquitetura, execute `./install.sh --server` ou `./install.sh --client` (com o mesmo `PREFIX` se personalizou a instalação) e reinicie os serviços. O assistente de configuração é para instalações novas e não sobrescreve dados existentes. Não volte a versões que desconheçam o formato do histórico atual.

```bash
systemctl --user stop crewsync-server.service
# Faça o backup e instale o novo pacote antes de continuar.
systemctl --user start crewsync-server.service
```

No cliente, use `crewsync-client.service`. Sem systemd, encerre o PID indicado em `service.pid`, confira que parou e use `start.sh` depois da atualização.

## Diagnostico

- Não conecta: confira se o servidor está aberto, o endereço está correto e a porta está liberada.
- Erro de certificado: confira o arquivo `ca.crt`, o domínio/IP, a data dos computadores e a validade do certificado.
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

### Instalação, certificados e execução manual

O bootstrap consulta a API pública de releases do GitHub, incluindo alpha, e valida o SHA-256 do pacote antes de extrair e executar o assistente. Requer ferramentas comuns de shell Linux; não depende do gerenciador de pacotes da distribuição. A API anônima pode limitar requisições; nesse caso, use o download manual. Os downloads agora usam `linux-x86_64` e `linux-arm64`. O antigo `unknown` era o campo de fornecedor no identificador de plataforma Rust, sem relação com a versão.

A inicialização usa systemd de usuário quando disponível; caso contrário, grava `start.sh`, `service.pid` e `service.log` na configuração e inicia com nohup. O modo sem systemd não configura início automático após reboot. O assistente valida domínio/IPv4 antes de instalar e inclui esse endereço no certificado. Verifica HTTPS por 127.0.0.1 antes de anunciar o servidor pronto, sem depender do DNS público. A CA é privada e criada automaticamente; não é um certificado público Let’s Encrypt. Firewall e acesso entre computadores precisam permitir a porta escolhida.

Para renovar, pare o servidor e gere um conjunto novo em outro diretório com o domínio ou IPv4 usado pelos clientes. Preserve o conjunto anterior até conferir o novo. Ajuste os caminhos do serviço e entregue o novo `ca.crt` a cada cliente antes de reiniciar a sincronização:

```bash
"$HOME/.local/share/crewsync/pilot-cert.sh" "$HOME/.config/crewsync-server/tls-novo" sync.empresa.com
```

Para uma instalação sem o assistente, copie e edite o exemplo TOML, gere os certificados e execute `crewsync-server serve CONFIG 0.0.0.0 8443 CERT KEY`. No cliente, crie a pasta de configuração privada (0700) e a pasta local, copie `ca.crt`, execute `crewsync-client activate URL CA SESSION NOME` e cole o convite. Use `crewsync-client sync SESSION PASTA DIRETORIO` uma vez ou `crewsync-client watch SESSION PASTA DIRETORIO 5` continuamente. Não rode `sync` junto ao serviço watch na mesma pasta.

### Arquitetura e sincronização

O servidor central ordena as revisões de cada compartilhamento. Os clientes comparam o conteúdo local, o estado remoto e a última revisão confirmada. Alterações concorrentes são preservadas como conflitos; registros de exclusão impedem que clientes antigos recriem arquivos apagados. Mover um arquivo é tratado como exclusão e criação, com reaproveitamento de conteúdo igual.

`watch` reage a eventos locais, agrupa gravações em uma janela de 200 ms e faz uma reconciliação completa pelo menos a cada intervalo configurado. A verificação periódica cobre eventos perdidos e alterações remotas. Falhas transitórias usam atraso crescente limitado; erros permanentes encerram o processo.

### Integridade e armazenamento

Arquivos são identificados por SHA-256, recebidos em streaming e armazenados sem compactação. Conteúdo igual compartilha um objeto, inclusive entre compartilhamentos. Publicação usa temporários, verificação de hash, renomeação e sincronização em disco antes da confirmação.

A raiz interna exige permissão 0700. Ela contém `objects`, `manifests`, `history`, `auth` e `tmp`. Não a edite externamente. `state.json` no cliente é o registro das revisões confirmadas, vinculado à identidade do servidor e ao compartilhamento; não o remova para tentar resolver um erro.

Linux kernel 5.6 ou superior, `/proc` montado e filesystem local com suporte a `fsync` e renomeação atômica são necessários. Não usar NFS como raiz interna. Links simbólicos e arquivos especiais são recusados. Caminhos têm até 4096 bytes e 128 componentes. Manifestos e histórico por caminho têm limite de 16 MiB; manifestos aceitam até cem mil entradas. Exceder os limites recusa a alteração.

### Segurança e configuração

HTTPS usa rustls com TLS 1.2/1.3, CA explícita e validação do endereço; não existe opção de ignorar o certificado. O servidor atende uma operação por vez, com timeout de conexão de 30 segundos, com acesso por domínio ou IPv4. Tokens são individuais por dispositivo e somente seus hashes ficam no servidor.

Somente pastas de primeiro nível são compartilhamentos; subpastas herdam o acesso. Ausência de permissão nega acesso. A configuração TOML é estrita e recarregada antes de cada requisição; recarga inválida mantém a última válida. Trocar a raiz exige reiniciar. `max_upload_bytes` limita cada envio, não o armazenamento total.

### Histórico, retenção e recuperação

Restaurar exige a revisão desejada e a revisão atual do caminho, criando uma nova revisão. Uma revisão de exclusão não é origem de restauração; escolha uma versão anterior. Para recuperar uma árvore, restaure os diretórios pais antes dos arquivos. Rename mantém versões no caminho antigo. Versões sobrescritas antes da implementação do histórico não podem ser reconstruídas.

A idade de retenção começa quando a versão deixa de ser atual. Histórico legado sem data recebe prazo de graça na primeira manutenção aplicada. Recuo do relógio não acelera exclusões. A manutenção exige o lock exclusivo com o servidor parado, conserva referências de todos os compartilhamentos e publica metadados duráveis antes de remover objetos. Os registros atuais de exclusão permanecem para clientes offline.

Backup bloqueia armazenamento e autenticação, valida a cópia e grava `backup.json` por último; sem esse marcador, a cópia está incompleta. Restauração exige destino novo e só grava `restored.json` após validar. Certificados e cópias locais de conflitos ficam fora desse backup e devem ser copiados separadamente.

Restaurar um backup antigo pode recuar revisões. Nesse caso, pare todos os clientes, preserve suas pastas e metadados para inspeção e configure novas pastas locais e novos dispositivos. Não reconecte automaticamente clientes antigos a um servidor que perdeu revisões.

### Serviços, logs e desinstalação

O assistente cria as unidades com os caminhos escolhidos. Para instalação manual, adapte os exemplos de `config/systemd/` e copie as unidades desejadas para `$HOME/.config/systemd/user/` e execute somente as correspondentes à função desta máquina:

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
