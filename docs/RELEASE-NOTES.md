# CrewSync 0.1.0-alpha.9

Corrige a permissão da raiz de armazenamento do servidor Linux. O daemon exige
que esse diretório seja privado (`0700`); a alpha.8 o criava como `0750`, então
o serviço podia falhar durante a configuração inicial. O assistente agora cria
a raiz como `0700`, e as atualizações Debian, Fedora, Arch e tar.gz também
corrigem instalações existentes sem alterar os arquivos sincronizados.

Se a configuração alpha.8 já gravou `server.toml` antes de falhar, não execute o
assistente novamente. O manual traz os comandos para ajustar a permissão,
reiniciar o serviço e gerar a chave administrativa e o convite inicial.

A distribuição continua em quatro formas: pacote Arch x86_64 via `makepkg`,
pacotes Debian/Fedora x86_64 e ARM64, e tar.gz x86_64 e ARM64. Esta é uma versão
alpha para testes; instalações e ciclos de atualização/reboot em VMs Arch,
Debian e Fedora continuam pendentes antes de uma versão estável.
