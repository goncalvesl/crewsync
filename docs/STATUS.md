# Estado e retomada

## Preparação da 0.1.0-alpha.3

A alpha.2 foi publicada na tag 49e32d7, com Packages 39822819605 e Rust 39822819807
bem-sucedidos, incluindo ARM64 nativo, fonte offline, ENOSPC, pacote e setup.
Os quatro hashes públicos foram conferidos. No teste adicional do bootstrap
publicado em terminal real, uma resposta imediata podia aparecer na tela porque
o prompt vinha antes de stty -echo. A alpha.3 corrige essa ordem e inclui regressão
PTY com resposta imediata; não mover ou sobrescrever a tag/arquivos da alpha.2.

Instaladores guiados em scripts/setup-online.sh e scripts/setup.sh. Bootstrap
identifica arquitetura, inclui alpha e verifica checksum. Servidor pergunta
pastas/IP/porta/usuário/compartilhamento, cria HTTPS e inicia. Cliente pede CA,
endereço e convite, ativa e inicia watch. systemd de usuário ou nohup; configurações
existentes preservadas. Pacotes linux-x86_64/linux-arm64. install.sh troca binários
atomicamente. Runtime sem Cargo/Python; OpenSSL só na preparação de certificados.

## Validação

Suíte local completa passou na preparação da alpha.2: build/fmt/Clippy, 72 Rust,
5 HTTPS, 3 TLS, certificado, site e gates. Quatro testes de setup com HTTPS real,
dois clientes, caminhos especiais, systemd-analyze, fallback nohup, configuração
preservada e bootstrap alpha simulado/checksum errado. Fonte compilou offline com
CARGO_HOME vazio. QA Chromium 390/768/1440px passou. Alpha.3 adiciona terminal real
para a ativação; os quatro testes e scripts/check-pilot.sh passaram novamente.
Publicar e verificar assets antes de atualizar o site conforme RELEASING.md.
Nenhum serviço ou dado real foi alterado.

## Publicar novamente

Tarefa recorrente em AGENTS.md e RELEASING.md: incrementar, testar, publicar e
conferir release/downloads por arquitetura, manual/README/site. Não encerrar no
commit/tag. prepare-release.py mantém versão/URLs sincronizados. Enviar tag antes
de main; verificar os assets antes de expor seus links em Pages.

## Pendências externas

Piloto básico em duas máquinas confirmado pelo mantenedor. Matriz física ampliada
e medições continuam pendentes; extended_matrix_accepted=false. Alpha autorizada
para testes; estável depende de aceite físico real. Linux >=5.6 com /proc e disco
local, rede privada e servidor serializado. Windows/GUI/Android futuros. Sem quotas
totais, retomada parcial, compactação ou limpeza de cópias locais. Não rebaixar após
histórico v2; backup independente necessário. Benchmark local não equivale a ensaio
físico nem promessa de desempenho.
