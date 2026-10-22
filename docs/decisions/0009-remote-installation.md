# 0009 — Instalação para servidor remoto

## Contexto

O mantenedor quer instalar em servidor remoto, usando domínio ou IPv4 público,
com o mesmo assistente de um comando. Pedir apenas o IP local impedia criar o
certificado para o endereço efetivamente usado pelos clientes.

## Decisão

O assistente pede o domínio DNS ou IPv4 de acesso, sem esquema, porta ou caminho.
O gerador valida a entrada antes de gravar qualquer arquivo e inclui o endereço
como DNS ou IP no subjectAltName. Mantém localhost/127.0.0.1 para o teste de
inicialização local; não exige DNS público funcionando nem acesso ao IP público
por dentro da VPS para iniciar o serviço. O bind continua 0.0.0.0 e a porta é
perguntada separadamente. URLs, curingas, IPv6 e nomes DNS inválidos são recusados
com instrução específica, sem interpolar configuração OpenSSL não validada.

Mantém CA privada gerada automaticamente e validação HTTPS explícita nos clientes.
O administrador entrega ca.crt por meio confiável. Não há ACME/Let's Encrypt,
proxy reverso ou emissão por CA pública automática nesta etapa. O nome do arquivo
pilot-cert.sh permanece compatível com instalações e pacotes existentes.

## Operação e limites

DNS deve apontar ao servidor; firewall do sistema e regras do provedor devem
permitir a porta TCP para os clientes. Serviços systemd de usuário precisam de
linger para sobreviver sem login; o assistente informa o comando. No modo nohup,
início após reboot continua manual. Certificado de servidor vale 90 dias.

O escopo passa a incluir servidor remoto HTTPS, além de rede local/VPN. Isso não
transforma os testes locais em validação de exposição pública: carga, abuso da
Internet e matriz física ampliada continuam pendentes. Permanecem autenticação,
autorização por operação, limites existentes e os critérios de release alpha.
