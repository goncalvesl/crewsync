# Plano de desenvolvimento

Estado inicial: somente planejamento e diretórios. Cada etapa exige implementação, testes automatizados pertinentes e documentação atualizada antes de ser considerada concluída. Não iniciar etapas seguintes automaticamente sem solicitação do usuário.

## 1. Fundação e decisões

- Definir nome, licença, plataformas Linux e decisões técnicas iniciais.
- Criar workspace Cargo com server, client e protocol.
- Configurar formatação, Clippy, testes e CI Linux desde o primeiro código.
- Documentar arquitetura, comandos e exemplos válidos de configuração.

Aceite: workspace compila, verificações passam e instalação de desenvolvimento está documentada.

## 2. Núcleo de armazenamento

- Implementar objetos raw por hash, manifestos, diretórios vazios, revisões e exclusões persistentes.
- Definir regras de nomes e caminhos; rejeitar links e arquivos especiais inicialmente.
- Implementar temporários, gravação durável, publicação atômica e recuperação após falhas.

Aceite: testes demonstram integridade, deduplicação, rejeição de caminhos inválidos e recuperação consistente.

## 3. Autenticação e autorização

- Implementar configuração TOML, usuários e compartilhamentos read/write.
- Implementar convites temporários, tokens por dispositivo, listagem, revogação e desativação.
- Adicionar HTTPS e validação de autorização em todas as operações.
- Rejeitar recarga inválida mantendo a última configuração válida.

Aceite: testes positivos e negativos cobrem acesso, isolamento, expiração, uso único e revogação.

## 4. Sincronização Linux

- Começar por sincronização manual e depois monitoramento contínuo.
- Cobrir sincronização inicial, criação, alteração, exclusão e movimentação.
- Implementar reconexão, reconciliação periódica, streaming e limites de recursos.
- Detectar alterações durante leitura, impedir ciclos e preservar conflitos por revisão-base.
- Inicialmente permitir movimentação como exclusão/criação, reutilizando conteúdo idêntico.

Aceite: testes de ponta a ponta com servidor e dois clientes preservam conteúdo e convergem, inclusive após desconexão.

## 5. Histórico e operação

- Implementar consulta e restauração de versões, lixeira e retenção configurável.
- Implementar coleta segura de objetos sem referências, contabilização de espaço e falhas por disco cheio.
- Adicionar logs sem segredos, unidades systemd e procedimentos de backup, restauração e atualização.

Aceite: testes comprovam restauração, retenção, limpeza concorrente segura e preservação de dados confirmados.

## 6. Piloto nos dois computadores

- Executar primeiro com dados sintéticos: arquivos grandes, muitos arquivos pequenos, interrupções de rede/processo, conflitos, exclusões, revogação e disco cheio.
- Automatizar cenários reproduzíveis; registrar validação real nas duas máquinas como complemento.
- Medir CPU, memória, tráfego, sincronização inicial e latência de propagação.
- Publicar 0.1.0-alpha somente após os critérios anteriores, documentando limitações.

Aceite: relatório de testes e medições; nenhuma perda silenciosa nos cenários definidos. Não prometer números não medidos.

## 7. Manual e site

- Manter o manual durante as etapas anteriores; finalizar instalação, início rápido, permissões, dispositivos, histórico, backup, atualização e diagnóstico.
- Construir site estático responsivo, com acabamento profissional, navegação acessível e capturas reais.
- Incluir apresentação, vantagens, funcionamento, downloads, manual, licença, contribuição e roadmap.
- Distinguir funcionalidades disponíveis de planejadas; deixar explícitos open-source, auto-hospedagem e suporte Linux inicial.
- Testar navegação, links, caminhos sob /REPOSITORIO/, páginas principais e instruções reproduzíveis.

Aceite: instalação possível usando somente a documentação; site verificado em desktop e celular, sem downloads ou recursos fictícios.

## 8. Distribuição pública

- Automatizar compilação, testes, pacotes Linux, checksums e publicação por tags.
- Usar GitHub Releases para binários e GitHub Pages para site, sem domínio próprio.
- Documentar dependências de execução, arquiteturas, notas da versão e compatibilidade.
- Validar os próprios pacotes antes de disponibilizar; lançamento estável depende do piloto e correções.

Aceite: site publicado, artefatos testados e downloads correspondentes à versão divulgada.

## Evolução posterior

1. Windows: caminhos, instalação, serviço e testes na plataforma.
2. Otimizações medidas: eventual compactação de transporte e retomada de transferências; armazenamento continua raw.
3. Interface gráfica: status, erros, conflitos e acesso à pasta.
4. Android online: listar, baixar e enviar, sem exigir espelhamento offline.

## Referências de hospedagem

- GitHub Pages: https://docs.github.com/en/pages/getting-started-with-github-pages/what-is-github-pages
- GitHub Releases: https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases
