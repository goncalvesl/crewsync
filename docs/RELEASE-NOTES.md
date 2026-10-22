# CrewSync 0.1.0-alpha.6

O cliente pergunta somente a raiz local e sincroniza automaticamente todas as
pastas às quais o usuário tem acesso. Por exemplo, ao escolher `/home/voce/Terna`,
o compartilhamento Obras aparece em `/home/voce/Terna/Obras`.

## Instalar

Cliente novo:

```sh
curl -fsSL https://raw.githubusercontent.com/goncalvesl/crewsync/main/scripts/setup-online.sh | sh -s -- client
```

Informe URL HTTPS, ca.crt recebido por meio confiável, convite e raiz local.
O assistente ativa o dispositivo e inicia o serviço. A lista de compartilhamentos
é consultada a cada reconciliação. Arquivos avulsos na raiz não são enviados;
pastas que perderem acesso permanecem no disco, sem novas sincronizações.

## Compatibilidade e atualização

O protocolo não mudou; o cliente novo usa a operação Shares já existente no
servidor alpha.4. Não é necessário reinstalar o servidor para usar a raiz local.
Os comandos individuais sync/watch continuam disponíveis. Novos comandos:
`sync-root SESSION RAIZ` e `watch-root SESSION RAIZ [SEGUNDOS]`.

Instalações existentes exigem migração explícita: pare o serviço, faça backup,
atualize o binário e mova a pasta individual inteira (incluindo .crewsync) para
RAIZ/NOME_DO_COMPARTILHAMENTO. Ajuste o serviço para watch-root. Não execute os dois
modos sobre a mesma árvore. O assistente preserva configurações existentes e
recusa reinstalação sobre elas. Veja o procedimento técnico no manual.

## Inicialização sem systemd

O fallback nohup herda a proteção contra SIGHUP antes de iniciar o processo.
Fechar o terminal imediatamente após o assistente não encerra mais o cliente
nessa janela. Uma regressão força o sinal antes de nohup executar.

## Limpeza dos testes

Os testes de instalação encerram e aguardam o serviço e seus workers antes de
remover os diretórios temporários. Uma regressão verifica a espera pelos filhos.
Isso corrige a corrida encontrada na validação ARM64 da tag alpha.5, que não
chegou a publicar uma release. A tag permanece preservada.

## Validação e limites

Validação local passou: 74 testes Rust, build, fmt, Clippy, cinco cenários HTTPS,
seis verificações TLS reais sem sockets, dois testes de entrada de certificado,
instalação, assistente em terminal, site e gates de release. O assistente testa
dois clientes em HTTPS, criação das subpastas e descoberta de um novo compartilhamento
sem reconfiguração. Pacotes musl x86_64/ARM64, fonte offline e ENOSPC são validados
pela CI antes da publicação.

Alpha para testes Linux >=5.6, /proc e disco local. Sincronização serial, sem
quotas totais, retomada parcial ou compactação. Erros permanentes encerram o serviço;
falhas transitórias têm atraso limitado. Aceite físico ampliado continua pendente;
não é versão estável nem substitui backup. Windows/GUI/Android ficam para o futuro.
GPL-3.0-only. Manual: https://goncalvesl.github.io/crewsync/manual.html.
