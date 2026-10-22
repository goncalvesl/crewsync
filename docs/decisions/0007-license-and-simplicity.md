# 0007 — GPL-3.0 e simplicidade como requisito de produto

O mantenedor autorizou expressamente GPL-3 e definiu o objetivo: ser simples de
instalar, manter e usar, além de rápido. Adotamos GPL-3.0-only, sem presumir licença
para versões futuras. Texto integral oficial em LICENSE; todos os pacotes Cargo
herdam o identificador. Avisos das dependências continuam sob suas próprias licenças.
Fonte do texto: https://www.gnu.org/licenses/gpl-3.0.txt.

Instalação deve reduzir passos, dependências e decisões. O pacote já fornece
executáveis nativos; o instalador agora permite cliente/servidor, mantém instalação
conjunta sem opções e copia manual/licença/exemplos para share/crewsync. Mostra
caminhos completos para dispensar ajustes de PATH. Não altera configurações reais,
não pede sudo nem inicia serviços. Certificados e acesso continuam preparação
única do administrador; não enfraquecer HTTPS para encurtar o procedimento.

O manual começa pelo cliente e uso diário, segue com preparação/manutenção do
servidor e termina com detalhes técnicos opcionais. Compilação, arquitetura,
protocolos, persistência, limites e desenvolvimento deixam o caminho inicial.
A documentação explica o que fazer antes de explicar os mecanismos internos.

Velocidade é objetivo a medir, sem prometer resultados universais. Simplicidade
não elimina integridade, proteção de concorrência, autenticação nem backup.
O aceite físico ampliado continua pendente; escolher licença não autoriza inventá-lo.
