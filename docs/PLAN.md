# Plano de desenvolvimento

O usuário autorizou executar sequencialmente todas as etapas restantes. Implementação
Linux, operação, site e automação avançaram; aceites externos continuam explícitos.
Veja STATUS.md para evidências atuais. Não considerar release pública concluída
sem aceite físico ampliado. Licença GPL-3.0-only escolhida pelo mantenedor.

## 1. Fundação e decisões

Workspace, arquitetura, testes e CI implementados. Distribuição definida para
Linux musl x86_64/ARM64. **Licença GPL-3.0-only definida pelo mantenedor**.
Objetivo: simples de instalar, manter e usar, além de rápido. Instalação mínima e
manual do usuário direto, com detalhes técnicos apenas na seção final. Sem promessa de MSRV; CI usa stable.

## 2. Núcleo de armazenamento

Implementado: objetos raw SHA-256, manifestos formato 1, diretórios vazios,
revisões/tombstones, deduplicação, streaming, temporários e publicação durável.
Caminhos validados (incluindo profundidade), links/arquivos especiais recusados.
Falhas determinísticas e ENOSPC real em tmpfs isolado preservam dados confirmados.
Perda física de energia continua ensaio externo, não simulado como aceite.

## 3. Autenticação e autorização

Implementado: TOML nativo, ACLs read/write, negação padrão, recarga que mantém última
válida, convites de uso único, token por dispositivo, hashes no servidor, revogação
e HTTPS rustls com CA/hostname. Testes positivos/negativos e ponta a ponta.

## 4. Sincronização Linux

Implementado: sync manual, watch por eventos com reconciliação periódica,
reconexão/backoff limitado, streaming, snapshots, baseline por revisão, preservação
de conflitos, diretórios vazios, exclusões e rename como exclusão/criação.
Testes com dois clientes e servidor, incluindo upload interrompido e recuperação.

## 5. Histórico e operação

Implementado: consulta/restauração individual, lixeira no servidor e retenção
configurável, coleta segura offline por lock exclusivo, simulação e contabilidade
de objetos/bytes, testes de falhas, logs sem segredos, unidades systemd, backup e
restauração verificáveis em destino novo. Manual de atualização e diagnóstico.
Não há limpeza automática das cópias locais; não faz parte da coleta do servidor.

## 6. Piloto nos dois computadores

**Piloto básico confirmado pelo usuário.** Benchmark sintético local registrado
em docs/reports, com CPU/memória/tempo por processo e conteúdo trafegado; tráfego
no fio não medido. ENOSPC real testado em tmpfs Docker, além das falhas simuladas.

Pendente externo: matriz ampliada e medições entre máquinas físicas (interrupções
de rede/processo, conflitos, exclusões, revogação, disco cheio e energia), com
hardware, kernel, volume e resultados. Não usar localhost como substituto. Registrar
aceite em docs/reports/physical-acceptance.json somente após executar. Release
0.1.0-alpha permanece condicionada a esses critérios.

## 7. Manual e site

Manual completo, instalação, início, permissões/dispositivos, conflitos, histórico,
retenção, backup/restauração, atualização, diagnóstico e contribuição em MANUAL.md.
Site estático responsivo em site/, manual gerado sem bibliotecas de terceiros,
links relativos para GitHub Pages sob /crewsync/. Verificados links/âncoras e
navegação em 390/768/1440px, sem overflow ou erros de console. Saída CLI real de
ensaio sintético; sem clientes, depoimentos, métricas ou downloads fictícios.
Workflow Pages publica o site. Estado real da publicação em STATUS.md.

## 8. Distribuição pública

Automação Packages implementada: musl x86_64/ARM64, testes, HTTPS, ENOSPC isolado,
pacote permitido, checksum, avisos de dependências e smoke de instalação. Execução
manual gera artefatos para ensaio. Tag correspondente publica prerelease somente
com LICENSE e aceite físico ampliado. Pacote x86_64 validado localmente sem Python
no container mínimo; ARM64 passou pela matriz CI, incluindo execução nativa de testes e instalação.

Pendente de lançamento: aceite externo, número publicável e tag. Nenhuma
versão estável anunciada. Disponibilidade dos artefatos/CI em STATUS.md.

## Evolução posterior (fora da versão Linux atual)

1. Windows: caminhos, instalador, serviço e testes da plataforma.
2. Otimizações após medições: transporte comprimido/retomada; armazenamento raw.
3. Interface gráfica para status, erros, conflitos e pastas.
4. Android online: listar, baixar e enviar, sem espelhamento offline obrigatório.
