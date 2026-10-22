# CrewSync 0.1.0-alpha.2

Instalação guiada de servidor e cliente Linux, com um comando e sem compilar.

## Instalar

Servidor:

```sh
curl -fsSL https://raw.githubusercontent.com/goncalvesl/crewsync/main/scripts/setup-online.sh | sh -s -- server
```

Cliente:

```sh
curl -fsSL https://raw.githubusercontent.com/goncalvesl/crewsync/main/scripts/setup-online.sh | sh -s -- client
```

O assistente detecta x86_64/ARM64, baixa a última release (incluindo alpha),
confere o SHA-256 e pergunta os diretórios. No servidor, configura usuário e pasta,
cria certificados HTTPS, inicia o serviço e fornece um convite. No cliente, pede
endereço, ca.crt e convite, ativa o dispositivo e inicia a sincronização.

Requer curl, tar e ferramentas comuns do shell/coreutils; OpenSSL apenas na
preparação do servidor. Não precisa de Rust, Cargo ou Python. O pacote também
inclui `setup.sh server|client` para executar o assistente sem o bootstrap online.
Os downloads agora usam nomes linux-x86_64 e linux-arm64; `unknown` nos nomes
anteriores era o campo de fornecedor do target Rust, sem relação com a versão.

## Serviços e instalações existentes

Usa systemd de usuário quando disponível. Para funcionar sem login, configure
`sudo loginctl enable-linger SEU_USUARIO`. Sem systemd, inicia em segundo plano e
informa log, PID e comando de reinício; nesse modo não configura início no boot.
O firewall deve permitir a porta TCP escolhida na rede local. Transfira apenas o
ca.crt para os clientes por um meio confiável; certificados não são ignorados.

O assistente não sobrescreve configurações, certificados ou sessões existentes.
Para atualizar a alpha.1, pare os serviços, faça backup e use `install.sh --server`
ou `--client` do novo pacote no mesmo PREFIX; depois reinicie. A troca de binários
agora é atômica. Não rebaixar após histórico v2. GPL-3.0-only preservada.

## Validação e limites

Testes do assistente: servidor HTTPS e dois clientes reais, sincronização watch,
preservação de configuração, entradas inválidas, caminhos com caracteres especiais,
unidades de serviço e bootstrap via pipe/terminal, incluindo recusa de hash errado.
A CI repete os testes com os binários musl em x86_64 e ARM64, além da suíte Rust,
HTTPS, ENOSPC isolado, pacotes e build offline do fonte vendorizado.

Manual: https://goncalvesl.github.io/crewsync/manual.html.
Linux kernel >=5.6 com /proc e filesystem local, rede privada, servidor serializado.
Windows/GUI/Android são futuros. Matriz física ampliada continua pendente; alpha
para testes, sem substituir backup independente. Certificado gerado vale 90 dias;
o manual explica renovação e operação.
