# CrewSync 0.1.0-alpha.8

Esta alpha acrescenta quatro formas de distribuição do servidor Linux: pacote
Arch (`.pkg.tar.zst`, x86_64), Debian (`.deb`) e Fedora (`.rpm`) para x86_64 e
ARM64, além do tar.gz independente para ambas as arquiteturas. O pacote Arch é
gerado com `makepkg`; Arch Linux ARM não está incluído. Os pacotes nativos usam os caminhos do sistema, criam uma
conta de serviço sem login e instalam um serviço systemd que só é habilitado
depois da configuração inicial. O painel pode atualizar a configuração com o
serviço em execução; TLS permanece sem escrita para o daemon. O tar.gz pode ser instalado em `/opt/crewsync`,
com configuração em `/etc/opt/crewsync` e dados em `/var/opt/crewsync`.

Para pacotes nativos, instale o arquivo da sua distribuição e execute
`sudo crewsync-server-setup`. No tar.gz, extraia e execute
`sudo ./install-system.sh`. O assistente configura HTTPS direto com CA privada ou
backend HTTP local para proxy HTTPS, inicia o serviço e mostra a chave do painel e
o convite inicial. O instalador por usuário continua disponível para pilotos e
clientes existentes; esta release não migra automaticamente instalações antigas.

Configuração e dados sobrevivem à atualização e remoção normal do pacote. Ainda
não publicamos repositórios apt/pacman/dnf. Antes da versão estável, falta ensaiar
instalação, atualização, remoção/reinstalação e reboot em VMs Arch, Debian e Fedora.
O piloto físico ampliado também continua pendente; esta é uma versão alpha para
testes, não uma declaração de aceite operacional.

O fluxo de release compila e testa os binários musl x86_64 e ARM64, testa HTTPS,
ENOSPC e instalação e gera os quatro formatos com checksum SHA-256. A fonte da
release inclui dependências vendorizadas e é compilada offline pela CI.
