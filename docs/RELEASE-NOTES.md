# CrewSync — candidato de desenvolvimento

Linux, HTTPS nativo em Rust, ativação por dispositivo, ACLs, sincronização manual
ou por eventos com reconciliação periódica, histórico e restauração individual.
Retenção configurável e coleta administrativa offline, backup e restauração em
destino novo. Pacotes musl estáticos e manual no GitHub Pages.

Configuração e sessões do piloto continuam válidas. Faça backup com servidor
parado antes de atualizar. Histórico formato 2 adiciona datas de arquivamento;
legado recebe prazo de graça. Não rebaixar servidor após a atualização.

Limitações: rede privada, servidor serial, Linux kernel >=5.6 com `/proc`,
filesystem local, sem quotas totais/retomada parcial/limpeza das cópias locais.
Windows/Android/interface gráfica são posteriores. O aceite dos ensaios físicos ampliados
ainda é condição de lançamento; esta nota não declara uma release
publicada. Licença GPL-3.0-only definida. Instalação por função com --client/--server,
manual simples e aprofundamento técnico na seção final. Consulte STATUS.md e os relatórios.
