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

## Comandos previstos

Após criação do workspace:

```bash
cargo build --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Esses comandos ainda não funcionam: não existe Cargo.toml. Quando o runner de ponta a ponta e o site forem criados, registrar seus comandos aqui e executá-los na CI. Diretórios de testes fora das crates não são descobertos automaticamente pelo Cargo; conectar explicitamente o runner.

## Evidências

Registrar no status ou relatório da etapa os comandos executados, resultados, ambiente e limitações. Não afirmar que testes passaram quando não foram executados. Benchmarks devem indicar hardware, volume e tipos de arquivo, sem tratar tempo exato como teste determinístico.
