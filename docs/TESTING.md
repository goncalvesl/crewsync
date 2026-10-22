# Política de testes automatizados

## Regra de conclusão

Toda funcionalidade deve incluir testes automatizados do comportamento e dos erros relevantes. Toda correção de bug deve incluir teste de regressão. Não considerar uma etapa pronta com testes ausentes, ignorados sem justificativa ou falhando. Verificações manuais complementam os testes, sem substituí-los.

Não há meta percentual de cobertura definida. Priorizar invariantes de integridade, autorização e recuperação; testes de funções isoladas não substituem integração e ponta a ponta.

## Camadas

- Unitários próximos dos módulos Rust: validação, permissões, revisões e regras de retenção.
- Integração em `crates/<crate>/tests/`: persistência, protocolo e comportamento entre componentes.
- Ponta a ponta em `tests/e2e/`: servidor e dois clientes em processos separados, com diretórios temporários e isolamento.
- Site e documentação: links, navegação, renderização em telas distintas, exemplos executáveis e pacotes de instalação, conforme forem implementados.

## Matriz mínima por funcionalidade

Cobrir hash incorreto, caminhos maliciosos, compartilhamentos isolados, leitura sem escrita, tokens inválidos/revogados, convites expirados/reutilizados, conflitos, exclusões após reconexão, diretórios vazios, arquivos mudando durante envio e ausência de ciclos.

Para persistência e operação, cobrir reinício após interrupção, transferências incompletas, configuração inválida, retenção, restauração, coleta de objetos ainda referenciados e disco cheio. Usar abstrações ou ambientes controlados para falhas determinísticas.

Testes não devem acessar dados reais, portas fixas ou serviços externos obrigatórios. Usar fixtures sintéticas, limites de tempo e espera por condições; evitar sleeps arbitrários. Testes devem funcionar em CI Linux desde a fundação do código.

## Comandos do workspace

Executar na raiz:

```bash
cargo build --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

O workflow `.github/workflows/ci.yml` executa build, fmt, Clippy e testes com
`--locked`. Também verifica dependências do sistema e roda os testes TLS e de rede.

Na implementação local há 82 testes Rust locais (incluindo integração de autenticação,
worker em processo separado, cliente+armazenamento e falhas simuladas), além de
cinco testes HTTPS e um ensaio ENOSPC marcados `ignored` por exigirem
recursos externos (TCP e tmpfs isolado). O motivo é explícito:
o ambiente inicial proibia sockets; a sessão atual permite rede e os testes são
executados explicitamente. Não suprimir esses testes na validação real/CI:

```bash
cargo build --workspace --locked
cargo test -p crewsync-server --test pilot --locked -- --ignored
```

O runner `tests/e2e/pilot.rs` está ligado ao Cargo do servidor por `[[test]]`.
Ele executa servidor e dois clientes em processos separados, cria certificados
sintéticos, usa porta efêmera e verifica convergência, conflitos, exclusões,
rename/deduplicação, reinício, permissões, revogação, symlinks e recusa de TLS
incorreto. O terceiro cenário cobre diagnósticos de inicialização: TLS ausente ou
inválido, porta/armazenamento em uso, raiz 0755 e configuração inválida sem ecoar
seus valores. Os binários de ambos os pacotes precisam estar compilados no mesmo
perfil/target; por isso o build do workspace antecede o teste. A CI limita a etapa
a três minutos. `scripts/check-pilot.sh` executa a sequência completa, com timeout
na etapa de rede.

Os testes `reconciliation.rs` usam o reconciliador real e o armazenamento real
com transporte determinístico sem sockets, cobrindo também resposta perdida,
editar durante envio, corrupção de download, mudanças arquivo/diretório e
conflitos entre exclusão de diretório e edição concorrente de filhos. Não são
substitutos do teste HTTPS. `worker.rs` exercita framing e autenticação em um
processo servidor separado, inclusive EOF prematuro sem publicação.

TLS real pode ser testado sem sockets, via OpenSSL MemoryBIO:

```bash
/usr/bin/python3 -I tests/e2e/tls_memory.py
```

São seis testes, para IPv4 e domínio DNS: CA confiável/hostname válido,
CA não confiável e hostname incorreto. O setup usa o gerador real `scripts/pilot-cert.sh`, validando também a
cadeia e permissões da chave. Esta verificação não testa TCP, firewall ou HTTP.

A ativação tem regressões para confirmar com Enter mantendo stdin aberto, aceitar
EOF em pipes/CRLF, limitar a entrada e não imprimir o convite.

Runtime: Linux >=5.6 com /proc e filesystem local; binários nativos sem Python.
Testes auxiliares/build do site/pacotes: Python padrão e OpenSSL CLI. O quinto
cenário HTTPS interrompe upload, confirma recuperação do worker e ausência de
conteúdo parcial publicado. Retenção/coleta têm relógio controlado, simulação,
prazo de graça, lixeira, dados atuais preservados e falhas de publicação. Backup
cobre locks, instalação em destino novo, preservação de tokens e marcador ausente.
notify tem teste real de eventos e exclusão da área interna. HTTP recusa headers
ambíguos, duplicados, chunked e cabeçalhos acima do limite.

Site: `python3 scripts/build-site.py` e `python3 tests/site/check.py target/site`.
Navegador: tests/site/browser.cjs, 390/768/1440px, console, overflow, links e âncoras.
Instalador: tests/packaging/test_install.py cobre cliente/servidor/ambos, caminhos
com espaços, reinstalação, configuração preservada e pacote incompleto recusado.
Pacotes: scripts/check-package.py valida GPL e metadados, checksum, extração segura, instalação em
prefixo temporário e comandos básicos. Nenhum teste acessa dados reais.
Pacotes Debian/Fedora: `scripts/package_system.py` constrói os dois formatos e
`scripts/check-system-packages.py` confere versão, arquitetura, arquivos e checksum.
O `.pkg.tar.zst` Arch é gerado por `makepkg` dentro de Arch Linux e validado por
`scripts/check-arch-package.py`; a release oferece essa arquitetura somente para
x86_64. A primeira configuração do serviço de sistema ainda precisa de validação
de instalação/upgrade/reboot em VMs Arch, Debian e Fedora.

Benchmark: `python3 scripts/benchmark.py --report target/benchmark.json`.
Relatório contém wall/CPU/ru_maxrss por processo cliente e bytes de conteúdo, não
tráfego no fio. ru_maxrss inclui o pico inicial herdado do harness no fork;
comparar só ensaios com a mesma metodologia. Não representa latência entre máquinas.
ENOSPC real passou em tmpfs Docker limitado a 1 MiB, preservando arquivo confirmado
e limpando o upload incompleto. Execute `scripts/check-disk-full.sh DEPS_MUSL`.
Falhas físicas de energia e matriz ampliada em duas máquinas permanecem
pendentes, conforme docs/reports/physical-acceptance.json.

## Evidências

Registrar no status ou relatório da etapa os comandos executados, resultados, ambiente e limitações. Não afirmar que testes passaram quando não foram executados. Benchmarks devem indicar hardware, volume e tipos de arquivo, sem tratar tempo exato como teste determinístico.

## Publicação

`python3 -m unittest discover -s tests/release` cobre sincronização de versões e
links, arquiteturas, fonte, README/manual/site obsoletos, tag/licença, bloqueio de
estável sem aceite físico e Cargo.lock desatualizado. prepare-release.py --check
roda na CI e no build de Pages. package-source/check-source verificam checksum,
extração e build offline com CARGO_HOME vazio; a release só publica após fontes e
ambas as arquiteturas passarem. Assets públicos são baixados e verificados antes
de expor os novos links no site. Procedimento recorrente em RELEASING.md.

Assistente: `python3 tests/packaging/test_setup.py` usa diretórios sintéticos,
porta efêmera, HTTPS real, dois clientes watch e ambos os modos de inicialização.
Cobre caminhos com espaços/aspas/%/$, preservação de configuração, entradas
inválidas, convite enviado imediatamente em terminal real sem eco e bootstrap via terminal com download simulado de alpha e hash incorreto.
Não instala nem inicia serviços na conta real. Na matriz de pacotes usa
CREWSYNC_BIN_DIR para executar os binários musl nativos de cada arquitetura.

Certificados remotos: cert_input.py cobre domínio/IPv4 e entradas maliciosas
recusadas sem gravar arquivos; tls_memory.py faz seis ensaios TLS (IP e domínio,
CA correta/incorreta e nome incorreto). test_setup.py inclui instalação com domínio
não resolvível, verificação SAN/hostname e HTTPS/SNI com curl --resolve; clientes
Rust do cenário de sincronização usam URL DNS localhost. Não alterar DNS público
ou /etc/hosts nos testes e não afirmar validação externa a partir de loopback.

## Raiz local e processos do assistente

Duas regressões Rust exercitam múltiplos compartilhamentos em subpastas, isolamento,
perda de acesso com cópia local preservada, arquivos avulsos, links e layout legado.
O teste real do assistente usa dois clientes e adiciona um compartilhamento durante
watch, verificando descoberta sem reconfiguração e ausência da pasta sem acesso.

O assistente cobre HUP enviado antes de nohup executar e
limpeza que encerra serviço e workers, aguardando pidfds antes de apagar o fixture.
A regressão HUP foi conferida contra o script anterior (falha esperada) e o script
corrigido. Os testes auxiliares exigem Linux com /proc e suporte a pidfd; isso não
adiciona dependências aos binários distribuídos.

## Três modos de transporte (implementação local)

Os testes do assistente agora têm onze cenários: os anteriores, modo HTTP/proxy,
recusa de HTTP antes de gravar, bind IPv4 específico e certificado existente/confiança do sistema sem
ca.crt no cliente. Os testes Rust verificam opt-in de HTTP, URLs sem credenciais
ou caminhos adicionais e leitura de sessões antigas sem mudar a confiança.

Proxy real:

```bash
CREWSYNC_CADDY_BIN=/caminho/caddy python3 tests/e2e/proxy.py
```

O teste usa Caddy real, TLS com CA sintética e HTTP no backend, dois clientes,
transferência de 1 MiB, rejeição de confiança incorreta e revogação. SSL_CERT_FILE
e SSL_CERT_DIR são alterados somente nos subprocessos isolados; não instala CAs
na conta real, não usa ACME e não depende de DNS externo/portas fixas. check-pilot
executa esse teste quando CREWSYNC_CADDY_BIN está definido; a CI instala Caddy e
o executa explicitamente. O teste não comprova emissão pública Let's Encrypt.

## Painel administrativo

`python3 tests/e2e/admin.py` cobre HTTP e HTTPS direto com cliente Rust real:
login, usuários, compartilhamentos/ACL, convite, ativação, revogação, bloqueio,
origem cruzada e formulário desatualizado. Com `PLAYWRIGHT_MODULE` apontando para
o módulo Playwright, executa também o fluxo Chromium completo e verifica telas
390/768/1440, console e overflow. Capturas em `target/admin-*.png`.

Seis testes Rust do módulo admin cobrem hash/permissões/rotação da chave, sessão,
CSRF, origem/expiração, escape HTML, limites, TOML inline/comentários, revisão e
publicação privada. O teste Caddy também verifica login e cadastro pelo proxy,
com cookie Secure derivado da origem pública HTTPS.

## Docker Compose

`sh scripts/test-docker-compose.sh` constrói a imagem, confere que a porta não é
publicada no host e que `/var/lib/crewsync` é bind mount, inicia como UID 10001
com configuração vazia, cria a chave administrativa, simula cadastro de usuário
e compartilhamento, emite convite, reinicia e verifica que o estado persiste. A
CI executa esse smoke test com Docker Compose.
