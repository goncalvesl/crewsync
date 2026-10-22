#!/bin/sh
# Instalador do pacote: sem sudo, sem alterar dados/configuração existentes.
set -eu
package_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
install_prefix=${PREFIX:-"$HOME/.local"}
case "$install_prefix" in /*) ;; *) echo 'PREFIX deve ser absoluto' >&2; exit 2 ;; esac
mkdir -p "$install_prefix/bin"
install -m 755 "$package_dir/bin/crewsync-client" "$package_dir/bin/crewsync-server" "$install_prefix/bin/"
printf 'Instalado em %s/bin. Consulte docs/MANUAL.md e mantenha configuração e dados fora deste pacote.\n' "$install_prefix"
