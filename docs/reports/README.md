# Relatórios

- synthetic-local.json: benchmark real, sintético e localhost. Registra hardware,
  kernel, bytes de conteúdo e wall/CPU/ru_maxrss de processos cliente. Nenhum
  documento real ou credencial. ru_maxrss pode incluir pico herdado do harness
  no fork. Não há medição de tráfego no fio nem resultados entre computadores.
- physical-acceptance.json: piloto básico confirmado pelo usuário; matriz ampliada
  permanece falsa até registrar o ensaio real. Não editar para destravar release
  sem cumprir os critérios de PLAN.md.
- synthetic-musl.json: mesmo ensaio com pacote estático, quando presente.

Reproduzir: `python3 scripts/benchmark.py --report target/benchmark.json`.
Não tratar tempos exatos como asserções determinísticas ou promessas de desempenho.
