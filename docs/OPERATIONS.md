# Operação

Referência de comandos e procedimentos em [MANUAL.md](MANUAL.md), seções Manutenção,
Backup, Serviços e atualização, Diagnóstico. Decisão [0006](decisions/0006-operation-native-distribution.md).

## Rotina sugerida

1. Acompanhar logs do serviço e espaço livre. Logs HTTP registram status e bytes;
   o relatório maintain contabiliza objetos, não uso total do filesystem.
2. Reservar janela para parar servidor e obter backup em outra mídia.
3. Simular `crewsync-server maintain CONFIG`; conferir política e relatório.
4. Aplicar com `--apply`, se desejado, e reiniciar servidor.
5. Ensaiar recuperação em destino novo, sem clientes reais conectados.

Não há agendamento automático de coleta: exige serviço parado e revisão pelo
operador. Não limpar manualmente objetos ou tombstones. A política do servidor
não remove conflitos/lixeira locais. Manter backups independentes das versões.

Backup preserva hashes dos tokens: proteger a cópia. Certificados/chaves TLS e
credenciais locais não são incluídos pelo comando, pois ficam fora da raiz.
A restauração entrega `DESTINO/store`; editar cópia da configuração para esse
caminho e validar antes de servir. Destinos incompletos ficam para diagnóstico;
ausência de backup.json/restored.json indica que a operação não terminou.
