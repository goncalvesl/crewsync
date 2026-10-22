# 0013 — Servidor Linux e formatos de instalação

## Contexto

O servidor atende como serviço contínuo e guarda a configuração, autenticação e
conteúdo sincronizado. Instalá-lo sob o HOME do administrador associa o daemon à
conta/sessão e dificulta uma operação padrão de servidor. O mantenedor definiu o
servidor somente para Linux. Clientes permanecem multiplataforma no roadmap.

## Direção

Planejar pacotes nativos para Arch Linux (PKGBUILD), Debian (.deb) e Fedora (RPM),
e um tar.gz upstream. Pacotes nativos devem obedecer às convenções do gerenciador
em cada distro e instalar executáveis em paths de sistema, configuração em /etc,
estado persistente em /var/lib, e unidade systemd de sistema sob usuário de
serviço dedicado sem shell/login. O daemon não deve operar como root.

Layout candidato comum para pacotes da distro: /usr/bin/crewsync-server,
/etc/crewsync/server.toml, certificados/configuração TLS em /etc/crewsync/tls,
/var/lib/crewsync para estado e armazenamento, e unidade do pacote em diretório
systemd da distro. Diretório de estado deve pertencer ao usuário do serviço; config
e binários permanecem administrados pelo root/gerenciador. A remoção comum deve
preservar config e dados; remoção total precisa ser ação explícita.

Para distribuição independente upstream, /opt/crewsync é adequado: FHS reserva
/opt/<package> para objetos estáticos de software adicional, /etc/opt para
configuração específica do host e /var/opt para dados variáveis. Um tar.gz pode
ser extraído ali e acompanhado de unidade de exemplo, ou permitir prefixo escolhido
pelo operador. Não escrever automaticamente em /usr/local nem exigir que tar.gz
seja instalado como root.

## Antes dos pacotes

Ainda precisamos definir instalação interativa inicial (URL pública, modo TLS,
usuário e compartilhamento), obtenção/rotação da chave administrativa, ownership
de certificados, ciclo de atualização do serviço, migração sem cópia/perda do
storage e compatibilidade das instalações atuais por usuário. Testes devem cobrir
upgrade, downgrade, remoção/reinstalação preservando estado, início após reboot e
recuperação de configuração em VMs Debian, Fedora e Arch. A alpha.7 mantém o
assistente por usuário existente; esta decisão orienta o desenho futuro e não altera
seu comportamento.

## Referências

- [Filesystem Hierarchy Standard 3.0 — /opt, /etc/opt e /var/opt](https://refspecs.linuxfoundation.org/FHS_3.0/fhs-3.0.pdf)
- [Debian Policy — hierarquia de arquivos e serviços](https://www.debian.org/doc/debian-policy/ch-opersys.html)
- [Arch package guidelines](https://wiki.archlinux.org/title/Arch_package_guidelines)
