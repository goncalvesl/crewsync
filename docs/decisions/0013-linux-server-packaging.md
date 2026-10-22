# 0013 — Servidor Linux e formatos de instalação

## Contexto

O servidor atende como serviço contínuo e guarda a configuração, autenticação e
conteúdo sincronizado. Instalá-lo sob o HOME do administrador associa o daemon à
conta/sessão e dificulta uma operação padrão de servidor. O mantenedor definiu o
servidor somente para Linux. Clientes permanecem multiplataforma no roadmap.

## Direção

Publicar pacote Arch (.pkg.tar.zst) para x86_64, pacotes Debian (.deb) e Fedora
(RPM) para x86_64/ARM64 e um tar.gz upstream para ambas arquiteturas. O pacote Arch
é produzido por `makepkg`; Arch Linux ARM fica fora da primeira publicação. Pacotes
de sistema devem obedecer às convenções do gerenciador
em cada distro e instalar executáveis em paths de sistema, configuração em /etc,
estado persistente em /var/lib, e unidade systemd de sistema sob usuário de
serviço dedicado sem shell/login. O daemon não deve operar como root.

Layout candidato comum para pacotes da distro: /usr/bin/crewsync-server,
/etc/crewsync/server.toml, certificados/configuração TLS em /etc/crewsync/tls,
/var/lib/crewsync para estado e armazenamento, e unidade do pacote em diretório
systemd da distro. Diretório de estado pertence ao usuário do serviço. A
configuração continua root-owned, mas o diretório permite escrita ao grupo do
serviço para que o painel publique TOML atomicamente; TLS fica em subdiretório
root-owned sem escrita para o daemon. Binários são administrados pelo gerenciador.
A remoção comum deve
preservar config e dados; remoção total precisa ser ação explícita.

Para distribuição independente upstream, /opt/crewsync é adequado: FHS reserva
/opt/<package> para objetos estáticos de software adicional, /etc/opt para
configuração específica do host e /var/opt para dados variáveis. Um tar.gz pode
ser extraído ali e acompanhado de unidade de exemplo, ou permitir prefixo escolhido
pelo operador. Não escrever automaticamente em /usr/local nem exigir que tar.gz
seja instalado como root.

## Implementação

O assistente `crewsync-server-setup` cria a configuração inicial, certificados e
convite sem sobrescrever configuração existente. Pacotes instalam o serviço
desabilitado e executam o daemon como usuário dedicado; a configuração inicial
habilita o serviço. O tar.gz traz `install-system.sh`, usa `/opt/crewsync`,
`/etc/opt/crewsync` e `/var/opt/crewsync`. O instalador por usuário segue disponível
para clientes e pilotos existentes. A remoção de pacotes preserva configuração e
dados. Ainda falta validar upgrade, downgrade, remoção/reinstalação e reboot em
VMs reais das três famílias antes de estabilizar esse fluxo.

## Referências

- [Filesystem Hierarchy Standard 3.0 — /opt, /etc/opt e /var/opt](https://refspecs.linuxfoundation.org/FHS_3.0/fhs-3.0.pdf)
- [Debian Policy — hierarquia de arquivos e serviços](https://www.debian.org/doc/debian-policy/ch-opersys.html)
- [Arch Linux — criação de pacotes com makepkg](https://wiki.archlinux.org/title/Creating_packages)
- [Arch package guidelines](https://wiki.archlinux.org/title/Arch_package_guidelines)
