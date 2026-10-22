#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-only
# Instalação local, sem sudo e sem modificar configuração/dados existentes.
set -eu
case "${1:---all}" in
    --client) install_client=yes; install_server=no ;;
    --server) install_client=no; install_server=yes ;;
    --all) install_client=yes; install_server=yes ;;
    --help)
        printf 'Uso: ./install.sh [--client | --server | --all]\nSem opção, instala cliente e servidor. PREFIX define o destino (padrão: ~/.local).\n'
        exit 0 ;;
    *) echo 'Opção inválida. Use ./install.sh --help.' >&2; exit 2 ;;
esac
if [ "$#" -gt 1 ]; then
    echo 'Informe apenas uma opção. Use ./install.sh --help.' >&2
    exit 2
fi
package_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
install_prefix=${PREFIX:-"$HOME/.local"}
case "$install_prefix" in /*) ;; *) echo 'PREFIX deve ser absoluto' >&2; exit 2 ;; esac
# Validar o pacote antes de iniciar a instalação.
for required in LICENSE docs/MANUAL.md; do
    test -f "$package_dir/$required" || { echo 'Pacote incompleto; extraia novamente.' >&2; exit 1; }
done
if [ "$install_client" = yes ]; then
    test -f "$package_dir/bin/crewsync-client" || { echo 'Cliente ausente no pacote.' >&2; exit 1; }
fi
if [ "$install_server" = yes ]; then
    for required in bin/crewsync-server config/server.example.toml scripts/pilot-cert.sh; do
        test -f "$package_dir/$required" || { echo 'Servidor incompleto no pacote.' >&2; exit 1; }
    done
fi
share_dir=$install_prefix/share/crewsync
mkdir -p "$install_prefix/bin" "$share_dir"
install -m 644 "$package_dir/LICENSE" "$share_dir/LICENSE"
install -m 644 "$package_dir/docs/MANUAL.md" "$share_dir/MANUAL.md"
install_binary() {
    temporary=$(mktemp "$install_prefix/bin/.crewsync-install.XXXXXX")
    if install -m 755 "$1" "$temporary" && mv -f "$temporary" "$2"; then :; else
        rm -f "$temporary"
        return 1
    fi
}
if [ "$install_client" = yes ]; then
    install_binary "$package_dir/bin/crewsync-client" "$install_prefix/bin/crewsync-client"
fi
if [ "$install_server" = yes ]; then
    install_binary "$package_dir/bin/crewsync-server" "$install_prefix/bin/crewsync-server"
    install -m 644 "$package_dir/config/server.example.toml" "$share_dir/server.example.toml"
    install -m 755 "$package_dir/scripts/pilot-cert.sh" "$share_dir/pilot-cert.sh"
fi
printf 'CrewSync instalado. Manual: %s/MANUAL.md\n' "$share_dir"
if [ "$install_client" = yes ]; then
    printf '\nCliente: peça ao administrador o endereço, o certificado e um convite.\nComandos disponíveis:\n  "%s/bin/crewsync-client" --help\n' "$install_prefix"
fi
if [ "$install_server" = yes ]; then
    printf '\nServidor: configuração de exemplo em %s/server.example.toml\nGerador de certificado: %s/pilot-cert.sh\n' "$share_dir" "$share_dir"
fi
