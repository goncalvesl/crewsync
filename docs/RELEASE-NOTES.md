# CrewSync 0.1.0-alpha.4

Instalação simples para servidor remoto por domínio ou IPv4 público/VPN.

## Instalar

Servidor:

```sh
curl -fsSL https://raw.githubusercontent.com/goncalvesl/crewsync/main/scripts/setup-online.sh | sh -s -- server
```

Cliente:

```sh
curl -fsSL https://raw.githubusercontent.com/goncalvesl/crewsync/main/scripts/setup-online.sh | sh -s -- client
```

No servidor, informe o endereço que os clientes usarão, como sync.empresa.com,
sem https:// ou porta. A porta é perguntada separadamente (padrão 8443). O
assistente valida o nome/IP antes de instalar, cria certificado com SAN DNS/IP,
inicia o serviço e mostra URL, pasta, ca.crt e convite. A verificação de início
é local e não depende de DNS público já configurado. No cliente, informe a URL
HTTPS completa e o ca.crt recebido por meio confiável, além do convite.

Aponte o DNS A para o IPv4 público e permita a porta TCP no firewall do Linux e
no painel do provedor para os clientes. Para systemd funcionar sem login SSH,
execute sudo loginctl enable-linger SEU_USUARIO. Sem systemd, o modo nohup
continua disponível; reinício após reboot é manual.

## Certificados e atualização

CA privada automática, HTTPS validado e certificado de servidor com validade de
90 dias. Não exige ACME/proxy e não instala certificado de CA pública/Let's Encrypt.
Domínio DNS ASCII/Punycode ou IPv4 aceitos; URLs, porta no nome, curingas, IPv6,
nomes inválidos e injeções são recusados sem criar certificados.

Instalações existentes são preservadas. Para atualizar, pare o serviço, faça
backup, instale o novo pacote com install.sh --server/--client no mesmo PREFIX e
reinicie. Para mudar o endereço de certificado já existente, gere conjunto novo
em outro diretório e ajuste o serviço, seguindo a seção técnica do manual;
redistribua o novo ca.crt. A troca de endereço não apaga dados ou dispositivos.

## Testes e limites

Validação de entradas sem gravar arquivos, TLS real por domínio/IP, nomes e CAs
incorretos recusados, instalação com domínio sem DNS público, verificação SAN/SNI,
ativação em terminal com convite oculto e dois clientes Rust sincronizando por
URL DNS. Suíte Rust/HTTPS, pacotes x86_64/ARM64, ENOSPC isolado, fonte offline e
site continuam na CI. Não confundir testes loopback com teste de acesso externo.

Manual: https://goncalvesl.github.io/crewsync/manual.html.
GPL-3.0-only. Alpha para testes Linux >=5.6, /proc e filesystem local. Servidor
serializado e sem quotas totais/retomada parcial. Aceite físico ampliado continua
pendente; não é versão estável nem substituto de backup. Windows/GUI/Android futuros.
