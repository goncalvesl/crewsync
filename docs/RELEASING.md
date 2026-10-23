# Publicar uma versão e atualizar os downloads

Tarefa recorrente registrada a pedido do mantenedor: quando ele pedir uma
publicação, entregar a release pública e atualizar a última versão no manual,
README e site, com download direto para cada arquitetura, checksum e fontes.
Essa tarefa não termina apenas no commit ou no início do workflow.

## Preparar

1. Ler STATUS/PROJECT/QUALITY/TESTING e inspecionar mudanças locais; preservar
   alterações do usuário e nunca publicar dados privados.
2. Consultar tags e releases existentes. Se o usuário não informar número, escolher
   o próximo da série 0.x. Enquanto o piloto ampliado não estiver aceito, incrementar
   o sufixo alpha (`0.1.0-alpha.1`, `0.1.0-alpha.2`, ...), identificando a versão de
   teste. Uma publicação estável exige aceite físico ampliado verdadeiro; não
   modificar esse aceite por suposição.
3. Executar `python3 scripts/prepare-release.py VERSAO`. Atualiza Cargo.toml,
   dependências internas, Cargo.lock, docs/release.json e os blocos de download no
   manual, README e site. Os URLs são versionados para funcionar também com alpha;
   o redirect GitHub releases/latest não seleciona prereleases.
4. Escrever docs/RELEASE-NOTES.md com mudanças, instalação, compatibilidade, limites
   e testes efetivamente executados. Registrar decisões relevantes e STATUS.
5. Executar scripts/check-pilot.sh, testes de preparação/gates e check de pacotes.
   O workflow Packages repete build/testes/HTTPS/ENOSPC/instalação em x86_64 e ARM64.

## Publicar antes de expor links no site

Os downloads preparados apontam para a nova tag; ainda não são downloads públicos.
Para o site não apontar para assets inexistentes, manter o commit local em main
até a release publicar e enviar primeiro apenas a tag:

```bash
git add ARQUIVOS_REVISADOS
git commit -m 'Prepara release VERSAO'
git tag -a vVERSAO -m 'CrewSync VERSAO'
git push origin vVERSAO
```

O workflow Packages publica a imagem multiarch `ghcr.io/goncalvesl/crewsync-server`
(tags de versão e `alpha` para prereleases, `latest` para estáveis). Depois que
imagem, pacotes e fontes passam, o workflow cria a release. O pacote é ligado ao
repositório público pelo `GITHUB_TOKEN`; confira que o pull anônimo funciona.
A fonte inclui o projeto, Cargo.lock, scripts e dependências em vendor/; o teste
compila offline com CARGO_HOME vazio. Artefatos de release têm checksum e licença.
O aceite físico continua exigido para versões estáveis, mas a autorização do
mantenedor permite alpha pública para teste sem declarar o piloto ampliado concluído.

Acompanhar até o resultado final. Se falhar, diagnosticar/corrigir e usar nova tag
se uma release já tiver sido publicada. Não mover tags públicas ou sobrescrever
assets de versões publicadas. Nunca anunciar publicação a partir de um job verde
sem conferir a release no GitHub.

## Verificar e atualizar a documentação pública

1. Conferir `gh release view vVERSAO --json url,isDraft,isPrerelease,assets`.
   Devem existir os tar.gz musl, `.deb` e `.rpm` para x86_64 e ARM64, o `.pkg.tar.zst`
   Arch x86_64, checksums de cada arquivo, fonte e checksum de fonte, além de install.sh
   e seu checksum. Para imagem, execute `docker buildx imagetools inspect
   ghcr.io/goncalvesl/crewsync-server:VERSAO` e confirme manifestos linux/amd64 e
   linux/arm64; teste pull sem autenticação e confirme tags versionada e `alpha`.
   Os nomes públicos usam linux-x86_64/linux-arm64; os targets Rust permanecem no
   metadado do pacote. Testar também setup.sh e o bootstrap publicado.
2. Baixar os assets publicados, conferir hashes e executar check-package no tar.gz
   x86_64 (ARM64 é executado nativamente na CI). Conferir os metadados dos três
   pacotes de sistema com `scripts/check-system-packages.py` na CI e que `--version`
   corresponde à tag. Não executar binário ARM64 diretamente no host x86_64.
3. Conferir URLs reais de cada arquitetura, checksum e fonte. A última versão do
   projeto é docs/release.json; `prepare-release.py --check` deve passar.
4. Só então `git push origin main`, publicando os links no site via Pages.
   Verificar publicação e conteúdo do manual no URL público. Registrar STATUS,
   versão, URL da release e resultados; commit/push do registro final se necessário.
5. Informar ao usuário a versão, página da release e manual atualizado. Explicitar
   pendências externas reais, sem inventar aprovações ou medições.

Se Pages aguardar uma proteção de ambiente, inspecionar a regra antes de agir.
Não remover proteções nem aprovar pela identidade de outra pessoa. Continuar as
verificações e reportar precisamente qualquer limitação de hospedagem.

## Arquivos que precisam continuar sincronizados

- docs/release.json: versão, tag, canal e arquiteturas.
- Cargo.toml, Cargo.lock e dependências internas.
- Blocos `downloads` de README.md, docs/MANUAL.md e site/index.html.
- docs/RELEASE-NOTES.md, docs/STATUS.md e procedimentos afetados.

A CI verifica divergências. Não editar manualmente URLs isolados: usar o comando
de preparação, que também serve para as próximas publicações solicitadas.
