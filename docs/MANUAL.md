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

A alpha.9 mantém os três modos de conexão, o painel administrativo e os pacotes de sistema para o servidor Linux. Atualize programas anteriores antes de configurar HTTP ou proxy.

O procedimento de Docker/Coolify e o bootstrap vazio descritos abaixo estão no
código atual do repositório. Os pacotes alpha.9 já publicados ainda criam o primeiro
usuário e compartilhamento durante o setup; a mudança chegará aos pacotes numa
próxima release. Enquanto isso, `setup-online.sh` continua usando o assistente
incluído na alpha.9; use o Compose do repositório para instalar pelo Coolify.

O assistente baixa a última release, identifica Intel/AMD ou ARM64 e pergunta onde instalar. No servidor, pergunta o modo de conexão e o domínio ou IPv4, prepara a raiz de dados, inicia o serviço e mostra a chave administrativa. No painel, crie usuários e compartilhamentos, configure permissões e gere convites para os dispositivos. No modo direto pode gerar CA privada ou usar certificado existente; no modo proxy gera um exemplo Caddy. No cliente, ativa o computador e inicia a sincronização.

Tenha `curl`, `tar` e `sha256sum` instalados; no servidor, também OpenSSL. Não precisa de Rust, Cargo ou Python. Os comandos funcionam no terminal de uma sessão Linux, sem sudo. Os diretórios sugeridos podem ser alterados. Escolha diretórios novos para a configuração e os dados internos do servidor; instalações existentes são preservadas.

Se usa Coolify, siga o [deploy Docker com HTTPS e dados no host](DISTRIBUTION.md#docker-e-coolify). Ele compila a versão do repositório e não exige copiar certificados para os clientes.

Se preferir baixar e conferir o pacote manualmente:

<!-- downloads:start -->
Versão atual: 0.1.0-alpha.9 (alpha, para testes).

- **Arch Linux:** [x86_64](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.9/crewsync-0.1.0_alpha.9-1-x86_64.pkg.tar.zst) · [SHA-256](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.9/crewsync-0.1.0_alpha.9-1-x86_64.pkg.tar.zst.sha256)
- **Debian/Ubuntu:** [amd64](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.9/crewsync_0.1.0.alpha.9-1_amd64.deb) · [SHA-256](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.9/crewsync_0.1.0.alpha.9-1_amd64.deb.sha256) · [ARM64](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.9/crewsync_0.1.0.alpha.9-1_arm64.deb) · [SHA-256](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.9/crewsync_0.1.0.alpha.9-1_arm64.deb.sha256)
- **Fedora:** [x86_64](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.9/crewsync-0.1.0.alpha.9-1.x86_64.rpm) · [SHA-256](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.9/crewsync-0.1.0.alpha.9-1.x86_64.rpm.sha256) · [ARM64](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.9/crewsync-0.1.0.alpha.9-1.aarch64.rpm) · [SHA-256](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.9/crewsync-0.1.0.alpha.9-1.aarch64.rpm.sha256)
- **Tar.gz independente:** [x86_64](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.9/crewsync-0.1.0-alpha.9-linux-x86_64.tar.gz) · [SHA-256](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.9/crewsync-0.1.0-alpha.9-linux-x86_64.tar.gz.sha256) · [ARM64](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.9/crewsync-0.1.0-alpha.9-linux-arm64.tar.gz) · [SHA-256](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.9/crewsync-0.1.0-alpha.9-linux-arm64.tar.gz.sha256)

[Notas da versão](https://github.com/goncalvesl/crewsync/releases/tag/v0.1.0-alpha.9) · [Código-fonte e dependências](https://github.com/goncalvesl/crewsync/releases/download/v0.1.0-alpha.9/crewsync-0.1.0-alpha.9-source.tar.gz)
<!-- downloads:end -->

Extraia o pacote e execute `./setup.sh server` ou `./setup.sh client` na pasta extraída. O resultado é o mesmo. `./install.sh --client` e `./install.sh --server` continuam disponíveis para instalar somente os programas, sem configurar ou iniciar serviços.

### Servidor instalado pelo sistema

Para operar o servidor como serviço Linux, instale o `.pkg.tar.zst` no Arch, o
`.deb` no Debian/Ubuntu ou o `.rpm` no Fedora, e execute `sudo
crewsync-server-setup`. O assistente cria uma conta de serviço sem login,
configura a raiz de armazenamento, instala o serviço systemd e mostra a chave
administrativa. Cadastre usuários e compartilhamentos no painel. Os arquivos ficam em `/usr/bin`, `/etc/crewsync` e
`/var/lib/crewsync`; a instalação do pacote não inicia o servidor antes desse
passo.

O estado interno do servidor deve permanecer privado (`0700`). Se uma instalação
alpha.8 parar com `diretório interno deve ser privado, regular e 0700`, corrija a
permissão e reinicie o serviço antes de concluir a configuração:

```sh
sudo chown crewsync:crewsync /var/lib/crewsync
sudo chmod 0700 /var/lib/crewsync
sudo systemctl restart crewsync-server
sudo -u crewsync /usr/bin/crewsync-server admin-key /etc/crewsync/server.toml https://sync.empresa.com:8443
sudo -u crewsync /usr/bin/crewsync-server invite /etc/crewsync/server.toml usuario 3600
```

Guarde a chave administrativa exibida e use a URL pública escolhida na
instalação. Para tar.gz, use
`/var/opt/crewsync`, `/etc/opt/crewsync/server.toml` e
`/opt/crewsync/bin/crewsync-server`. A alpha.9 aplica `0700` automaticamente
durante instalação e atualização.

O tar.gz independente é a quarta opção: extraia-o e execute `sudo
./install-system.sh` na pasta extraída. Ele instala o programa em `/opt/crewsync`,
a configuração em `/etc/opt/crewsync` e os dados em `/var/opt/crewsync`. Arch está
disponível para x86_64; os pacotes Debian/Fedora e o tar.gz estão disponíveis para
x86_64 e ARM64 na seção de downloads. A remoção do programa não apaga os dados;
mantenha backup antes de atualizar. Atualizações dos pacotes reiniciam o serviço;
o tar.gz o interrompe durante a troca dos binários e o inicia novamente.

## Painel do servidor

**Alpha.9.** Para atualizar uma instalação alpha.8, atualize o pacote ou substitua os programas mantendo a configuração e o armazenamento. Abra no navegador o endereço do
servidor, por exemplo `https://sync.empresa.com/`. O instalador mostra a chave
administrativa ao terminar; guarde-a em um gerenciador de senhas. Ela é separada
dos convites e dos tokens dos dispositivos.

No painel você pode:

- Adicionar usuários e ativar ou bloquear seu acesso.
- Criar pastas compartilhadas e dar acesso de leitura ou leitura e escrita por usuário.
- Gerar convites de uso único para ativar cada computador.
- Consultar dispositivos por usuário e revogar individualmente o acesso.

As pastas são compartilhamentos gerenciados pelo CrewSync. O painel não importa
um diretório arbitrário do disco. Clientes com raiz automática criam as pastas
autorizadas dentro da raiz escolhida. Bloquear um usuário ou revogar um dispositivo
interrompe seu acesso ao servidor; os arquivos já baixados permanecem no computador.

Em instalações existentes, ou se perder a chave, execute no servidor com a conta
do serviço e o caminho real da configuração:

```sh
crewsync-server admin-key /caminho/server.toml https://sync.empresa.com
```

No pacote de sistema, use: `sudo -u crewsync /usr/bin/crewsync-server admin-key
/etc/crewsync/server.toml https://sync.empresa.com`.

O comando mostra uma nova chave e encerra as sessões administrativas anteriores.
Não altera tokens dos dispositivos. Informe a URL pública exata, incluindo porta
se houver: ela protege os formulários e define os cookies HTTPS mesmo atrás do
proxy. Não compartilhe a chave com usuários comuns. Sair encerra a sessão atual;
reiniciar o serviço também encerra sessões do painel.

O exemplo de Caddy da seção de conexão já publica o painel junto da sincronização.
Em HTTP de teste, chave e dados trafegam sem criptografia. Para uso real, escolha
HTTPS direto ou pelo proxy. Nenhum passo adicional de frontend é necessário.

O TOML continua disponível como alternativa. Se outra pessoa alterar a configuração
enquanto um formulário estiver aberto, o painel recusa a gravação: recarregue a
página e confira os novos valores antes de tentar novamente.

## Dispositivos

Antes de instalar cada cliente, peça ao administrador o endereço do servidor e um convite próprio para este computador. Com HTTPS público, escolha `system` no campo de CA e não copie certificados. Com CA privada, peça também o arquivo `ca.crt`. HTTP de teste exige confirmar a palavra `HTTP` no assistente. Escolha uma raiz local; o cliente descobre as pastas autorizadas automaticamente. Transfira `ca.crt` por um meio confiável. Nunca copie as chaves `.key` nem a sessão de outro computador.

### Um convite por dispositivo

Cada computador precisa de um convite novo, mesmo sendo do mesmo usuário. O convite é temporário e de uso único; depois da ativação, o dispositivo recebe sua própria credencial e herda as permissões do usuário. Não precisa de outro convite para reiniciar ou receber acesso a uma nova pasta.

No servidor, gere um convite válido por uma hora, substituindo `ana` pelo usuário cadastrado:

```bash
"$HOME/.local/bin/crewsync-server" invite "$HOME/.config/crewsync-server/server.toml" ana 3600
```

Cole o convite quando o assistente do cliente solicitar; ele não aparece enquanto você digita. Se perder o convite, se ele expirar ou já tiver sido usado, gere outro. Se a resposta de uma ativação for perdida, gere outro convite para tentar novamente.

### Conferir ou bloquear um dispositivo

No servidor, liste os dispositivos e use o ID correspondente para revogar somente aquele computador:

```bash
"$HOME/.local/bin/crewsync-server" devices "$HOME/.config/crewsync-server/server.toml"
"$HOME/.local/bin/crewsync-server" revoke "$HOME/.config/crewsync-server/server.toml" ID_DO_DISPOSITIVO
```

Revogar um dispositivo preserva o acesso dos demais. Desativar o usuário com `enabled = false` bloqueia todos os seus dispositivos. As cópias locais não são apagadas ao perder o acesso.

## Sincronizar

Depois do assistente, a sincronização já fica ativa. Escolha somente a raiz local (por exemplo, `/home/voce/Terna`): o cliente cria `Terna/Obras` e uma subpasta para cada compartilhamento autorizado. Crie um arquivo de teste dentro de uma dessas subpastas e confira no outro computador. Arquivos soltos na raiz não são enviados. A lista é atualizada a cada reconciliação; se o acesso for removido, a cópia local permanece sem sincronizar.

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

Os exemplos desta seção usam a instalação por usuário: ela guarda a configuração
em `$HOME/.config/crewsync-server/server.toml` e os arquivos internos em
`$HOME/.local/share/crewsync-server`. Não edite o armazenamento interno.

Em servidor instalado por pacote do sistema, a configuração é
`/etc/crewsync/server.toml`, os dados ficam em `/var/lib/crewsync` e os comandos
que alteram convites ou dispositivos devem rodar como a conta de serviço. Por
exemplo: `sudo -u crewsync crewsync-server invite /etc/crewsync/server.toml ana 3600`.
Prefira o painel administrativo para usuários, pastas e dispositivos.

### Adicionar um usuário

No servidor, faça backup antes de editar a configuração:

```bash
cp -a "$HOME/.config/crewsync-server/server.toml" "$HOME/.config/crewsync-server/server.toml.bak"
```

Abra `server.toml` no editor de sua preferência. Para cadastrar uma nova pessoa, acrescente um bloco com um nome único:

```toml
[users.ana]
enabled = true
```

Se o usuário já existe, mantenha o bloco existente, sem duplicá-lo. O cadastro sozinho não concede acesso a nenhuma pasta.

### Adicionar pastas e permissões

Acrescente ao mesmo arquivo os compartilhamentos desejados. Exemplo para a usuária `ana`, já cadastrada:

```toml
[shares.Financeiro.permissions]
ana = "write"

[shares.Projetos.permissions]
ana = "write"
```

Use `read` para leitura e `write` para leitura, criação, alteração e exclusão. Sem permissão, o usuário não vê a pasta. Para conceder acesso a uma pasta já configurada, acrescente o usuário ao bloco de permissões existente, sem repetir o cabeçalho.

Confira a configuração depois de editar:

```bash
"$HOME/.local/bin/crewsync-server" check-config "$HOME/.config/crewsync-server/server.toml"
```

Com configuração válida, não precisa reiniciar o servidor: as permissões são recarregadas na próxima operação. Se houver erro, corrija o arquivo; o servidor mantém a última configuração válida.

Nos clientes alpha.6 com raiz automática, as pastas aparecem na próxima reconciliação. Ao escolher `/home/voce/Terna`, por exemplo, serão criadas `/home/voce/Terna/Financeiro` e `/home/voce/Terna/Projetos`. Todos os dispositivos ativados do usuário recebem o mesmo acesso, sem novos convites.

Coloque os arquivos nessas pastas pelo cliente. Não crie ou copie diretórios dentro do armazenamento interno do servidor: ele mantém objetos e índices próprios. Subpastas herdam a permissão do compartilhamento; não precisam de outro bloco no TOML.

Se o servidor foi instalado como `root`, `$HOME` nos comandos acima corresponde a `/root`. Execute os comandos de configuração e convite na conta usada para instalar o servidor. A configuração de exemplo instalada em `$HOME/.local/share/crewsync/server.example.toml` mostra outros casos.

## Conexão e rede

No assistente do servidor, escolha um dos três modos:

- `1` — HTTPS direto (padrão): usa certificado/chave existentes ou gera uma CA privada. Sem proxy; porta padrão 8443. Com CA privada, entregue somente `tls/ca.crt` aos clientes. Nunca compartilhe as chaves `.key`.
- `2` — HTTPS por proxy: CrewSync atende HTTP em `127.0.0.1:8080` e gera um `Caddyfile`. O proxy fornece HTTPS público e cuida dos certificados; o cliente usa `https://sync.empresa.com` e `system`.
- `3` — HTTP para testes: não gera certificado. Escuta em `127.0.0.1:8080` por padrão e exige confirmar `HTTP`. Convites, credenciais e arquivos trafegam sem criptografia; use dados sintéticos e ambiente isolado. Para duas máquinas de teste, escolha conscientemente um IPv4 de escuta acessível e restrinja a porta aos clientes.

O cliente nunca troca HTTPS por HTTP automaticamente e não segue redirecionamentos. Informe a URL final da raiz do serviço, sem caminho adicional. Certificado inválido, nome incorreto ou CA não confiável causam erro. Para HTTPS público, o Linux precisa das CAs do sistema, normalmente fornecidas por `ca-certificates`. Para CA privada, informe o caminho absoluto de `ca.crt`.

### Adicionar um proxy com Caddy

Se já usa Coolify ou outro proxy nas portas 80/443, aproveite esse proxy conforme a seção seguinte. Para uma máquina sem proxy, o caminho simples com Caddy é:

- Instale o servidor CrewSync escolhendo `2`, informe `sync.empresa.com` e aceite a porta interna 8080. Ele mostra o caminho do exemplo `Caddyfile` e informa que a URL externa depende do proxy.
- Aponte o DNS A do domínio ao IPv4 público. Se houver registro AAAA, ele também precisa alcançar o proxy. Libere TCP 80 e 443 no Linux e no provedor. Mantenha a porta interna 8080 privada.
- Instale Caddy. Em Debian/Ubuntu com o pacote disponível, use os comandos abaixo; para outras distribuições ou o repositório oficial, consulte a [instalação do Caddy](https://caddyserver.com/docs/install).

```bash
sudo apt update
sudo apt install caddy
```

Edite `/etc/caddy/Caddyfile` e adicione o bloco abaixo, substituindo o domínio e a porta pelos escolhidos. Preserve os outros sites existentes e não duplique um bloco para o mesmo domínio. O assistente também deixa esse exemplo em `$HOME/.config/crewsync-server/Caddyfile`.

```bash
sudo nano /etc/caddy/Caddyfile
```

```text
sync.empresa.com {
    reverse_proxy 127.0.0.1:8080
}
```

Valide e recarregue:

```bash
sudo caddy validate --config /etc/caddy/Caddyfile --adapter caddyfile
sudo systemctl reload caddy
```

O Caddy emite e renova o certificado automaticamente por ACME, usando autoridades como Let's Encrypt. DNS e conectividade pública precisam funcionar para isso. No cliente, use `https://sync.empresa.com` e aceite `system` para a CA. Não precisa copiar `ca.crt` nem abrir a porta do backend na Internet. Confira os logs do proxy se a emissão falhar: `sudo journalctl -u caddy -n 50 --no-pager`. Veja [HTTPS automático do Caddy](https://caddyserver.com/docs/automatic-https).

### Usar o proxy existente ou Coolify

Configure um domínio HTTPS dedicado no proxy existente e encaminhe todas as requisições, incluindo `/v1/rpc`, ao backend HTTP do CrewSync. Preserve método, corpo e `Content-Length`; desative compressão/buffering que altere o framing e ajuste limites de tamanho/tempo para os arquivos. O exemplo Caddy foi testado com transferência em streaming. Outros proxies precisam de validação equivalente.

Se o proxy está no mesmo host, `127.0.0.1:8080` funciona. Se está em container, o localhost dele é outro: use uma rede privada ou endereço do host acessível apenas ao proxy e ajuste o bind do `serve-http` no script de início. Não exponha esse backend à Internet. Reinicie o serviço depois de alterar o bind. O domínio público deve apontar ao proxy, e o cliente continua usando HTTPS com `system`.

### HTTPS direto e certificados existentes

No modo `1`, informe os caminhos absolutos do certificado completo (fullchain) e da chave, ou aceite `gerar` para CA privada. Em certificado público, o cliente usa `system`; em CA privada, usa `ca.crt`. Emissão e renovação de certificados externos são responsabilidade do operador. Após renovar, reinicie o serviço CrewSync para carregar os novos arquivos. A CA privada gerada continua com certificado de servidor de 90 dias; a renovação está na seção técnica.

Para acesso remoto direto, libere somente a porta TCP escolhida e aponte o DNS ao servidor. Em LAN/VPN com UFW, por exemplo:

```bash
sudo ufw allow from 192.168.1.0/24 to any port 8443 proto tcp
```

Não é necessário abrir portas nos clientes. Eles usam o endereço real do servidor/proxy, nunca `0.0.0.0`. Com systemd de usuário, `sudo loginctl enable-linger SEU_USUARIO` permite funcionar sem login SSH. O instalador não instala proxy nem altera firewall automaticamente.

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

Para uma instalação sem o assistente, copie e edite o exemplo TOML, gere os certificados e execute `crewsync-server serve CONFIG 0.0.0.0 8443 CERT KEY` para HTTPS direto ou `crewsync-server serve-http CONFIG 127.0.0.1 8080` para proxy/testes. No cliente, crie a pasta de configuração privada (0700) e a pasta local, copie `ca.crt`, execute `crewsync-client activate URL CA SESSION NOME` e cole o convite. Use o caminho de CA privada, `system` para HTTPS com confiança do sistema ou `insecure-http` para habilitar HTTP explicitamente. Use `crewsync-client sync-root SESSION RAIZ` uma vez ou `crewsync-client watch-root SESSION RAIZ 5` continuamente. Os comandos individuais `sync SESSION PASTA DIRETORIO` e `watch SESSION PASTA DIRETORIO 5` continuam disponíveis. Não rode `sync` junto ao serviço watch na mesma pasta.

### Arquitetura e sincronização

O servidor central ordena as revisões de cada compartilhamento. Os clientes comparam o conteúdo local, o estado remoto e a última revisão confirmada. Alterações concorrentes são preservadas como conflitos; registros de exclusão impedem que clientes antigos recriem arquivos apagados. Mover um arquivo é tratado como exclusão e criação, com reaproveitamento de conteúdo igual.

`watch` reage a eventos locais, agrupa gravações em uma janela de 200 ms e faz uma reconciliação completa pelo menos a cada intervalo configurado. A verificação periódica cobre eventos perdidos e alterações remotas. Falhas transitórias usam atraso crescente limitado; erros permanentes encerram o processo.

### Integridade e armazenamento

Arquivos são identificados por SHA-256, recebidos em streaming e armazenados sem compactação. Conteúdo igual compartilha um objeto, inclusive entre compartilhamentos. Publicação usa temporários, verificação de hash, renomeação e sincronização em disco antes da confirmação.

A raiz interna exige permissão 0700. Ela contém `objects`, `manifests`, `history`, `auth` e `tmp`. Não a edite externamente. `state.json` no cliente é o registro das revisões confirmadas, vinculado à identidade do servidor e ao compartilhamento; não o remova para tentar resolver um erro.

Linux kernel 5.6 ou superior, `/proc` montado e filesystem local com suporte a `fsync` e renomeação atômica são necessários. Não usar NFS como raiz interna. Links simbólicos e arquivos especiais são recusados. Caminhos têm até 4096 bytes e 128 componentes. Manifestos e histórico por caminho têm limite de 16 MiB; manifestos aceitam até cem mil entradas. Exceder os limites recusa a alteração.

### Segurança e configuração

HTTPS usa rustls com TLS 1.2/1.3, CA privada ou confiança nativa do sistema e validação do endereço; não existe opção de ignorar o certificado. O servidor atende uma operação por vez, com timeout de conexão de 30 segundos, com acesso por domínio ou IPv4. Tokens são individuais por dispositivo e somente seus hashes ficam no servidor.

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

### Migrar uma pasta individual para a raiz local

Pare o serviço do cliente antes de mudar o layout. Uma pasta anteriormente usada
como o próprio compartilhamento não pode virar raiz automaticamente. Por exemplo,
renomeie `Terna` para `Terna-anterior`, crie uma nova `Terna` e mova
`Terna-anterior` inteira para `Terna/Obras`, incluindo `.crewsync` e os arquivos
ocultos. Use o nome real do compartilhamento e preserve um backup. Altere o serviço
para `watch-root SESSION RAIZ 5`, recarregue o systemd e inicie novamente. Não rode
os modos individual e raiz ao mesmo tempo. O cliente recusa mistura de layouts,
links e raízes vinculadas a outro servidor.

### Trocar o modo de uma instalação existente

Pare o serviço e faça backup da configuração, estado e certificados. Atualize os
programas para a versão com os três modos, sem executar setup sobre configuração
existente. No servidor, ajuste `start.sh` para `serve` com certificado/chave ou
`serve-http` com bind privado; mantenha o mesmo server.toml e armazenamento.
Configure o proxy antes de conectar os clientes e reinicie o serviço.

Se a URL pública mudar, preserve as pastas antigas e use uma nova sessão e raiz
local com novo convite; os estados existentes são vinculados à URL. Não edite
state.json ou session.json para contornar essa proteção. Se a URL/CA privada
continuar igual, as sessões antigas seguem válidas. Remova a exposição da antiga
porta pública no firewall depois de verificar o novo endpoint.

O painel usa o mesmo endereço público e o mesmo proxy da sincronização, sem
uma segunda porta. Se mudar o endereço público do painel, gere uma nova chave
administrativa com a URL correta, como explicado na seção do painel.
