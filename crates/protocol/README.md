# protocol

Contratos compartilhados de piloto: modelos de manifestos/entradas, validação de
caminhos, RPC versão 1, framing limitado, utilitários de persistência e ponte HTTPS.
Não depende das crates de aplicação. Usa serde/serde_json e rustix.

O HTTP/TLS temporário usa Python padrão embarcado no binário. Requer Python >= 3.11,
com `ssl`; usa validação de certificado e hostname sem modo inseguro. Veja
[a decisão do piloto](../../docs/decisions/0004-pilot.md) e [o roteiro](../../docs/PILOT.md).
Contratos ainda não constituem API estável de uma release pública.
