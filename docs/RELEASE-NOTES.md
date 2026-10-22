# CrewSync 0.1.0-alpha.7

Esta versão de teste adiciona o painel administrativo do servidor e três modos de conexão: HTTP explícito para testes, HTTPS atrás de proxy e HTTPS direto. O painel fica no mesmo endereço da sincronização.

## Painel do servidor

O instalador mostra uma chave administrativa aleatória. Use o painel para cadastrar e bloquear usuários, criar pastas compartilhadas, conceder leitura ou escrita, criar convites por computador e consultar ou revogar dispositivos. A chave não é um convite e não substitui os tokens dos clientes. Para recuperar acesso, execute localmente `crewsync-server admin-key CONFIG URL_PUBLICA`; isso troca a chave e encerra sessões administrativas anteriores.

O painel altera o TOML validando a configuração toda e grava por publicação atômica. Atualizações concorrentes são recusadas para revisão. Compartilhamentos são nomes virtuais geridos pelo CrewSync; o painel não importa caminhos arbitrários do disco nem remove conteúdo do armazenamento. Arquivos que já foram sincronizados continuam no computador após bloqueio ou revogação.

## Conexão e proxy

O HTTPS direto continua disponível, com CA própria ou certificado existente. HTTPS por proxy permite reutilizar Caddy, Coolify ou outro proxy. O backend HTTP deve ficar restrito ao proxy, normalmente em loopback ou numa rede privada de containers. O HTTP sem TLS é apenas para teste isolado. O cliente aceita certificados do sistema ou CA privada e não reduz o nível de segurança automaticamente.

O manual descreve as opções e traz uma configuração mínima de Caddy. O painel usa o mesmo domínio/porta e a mesma configuração do proxy.

## Instalação e compatibilidade

Instale com o assistente publicado ou baixe o pacote para Linux x86_64 ou ARM64. A atualização substitui os binários; mantenha a configuração, o armazenamento e certificados atuais. Não execute o assistente sobre diretórios de configuração existentes sem revisar as perguntas. Sessões administrativas são encerradas ao reiniciar o servidor. Clientes já ativados continuam usando os seus tokens individuais.

A alpha.7 é uma versão de testes Linux. Não é uma versão estável e não representa o aceite físico ampliado. Faça backup independente antes de uso com dados importantes.

## Verificações

Passaram build, fmt, Clippy, 82 testes Rust, cinco cenários HTTPS explícitos, seis ensaios TLS MemoryBIO, dois testes de entrada do certificado, onze cenários do instalador, testes do painel HTTP/HTTPS com clientes reais, fluxo Chromium em 390/768/1440 pixels, teste ponta a ponta via Caddy e verificações do site e dos gates de release. O ensaio ENOSPC exige Docker/tmpfs e é executado separadamente na matriz de pacote. A CI de release também compila os pacotes musl x86_64 e ARM64 e valida fontes offline.
