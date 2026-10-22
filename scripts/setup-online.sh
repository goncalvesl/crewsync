#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-only
# Bootstrap sem Python, Cargo ou jq. As perguntas usam o terminal, mesmo via pipe.
set -eu
case "${1:-}" in server|client) role=$1 ;; *) echo 'Uso: sh setup-online.sh server|client' >&2; exit 2 ;; esac
[ "$#" -eq 1 ] || exit 2
for command in curl tar sha256sum mktemp uname sed head grep; do
    command -v "$command" >/dev/null || { echo "Instale $command e execute novamente." >&2; exit 1; }
done
[ "$(uname -s)" = Linux ] || { echo 'Esta versão requer Linux.' >&2; exit 1; }
case "$(uname -m)" in x86_64) arch=linux-x86_64 ;; aarch64|arm64) arch=linux-arm64 ;; *) echo 'Arquitetura não suportada (use x86_64 ou ARM64).' >&2; exit 1 ;; esac
# A API inclui prereleases; /releases/latest não inclui as versões alpha.
release=$(curl --proto '=https' --proto-redir '=https' --tlsv1.2 -fsSL --connect-timeout 15 --max-time 60 'https://api.github.com/repos/goncalvesl/crewsync/releases?per_page=1')
tag=$(printf '%s\n' "$release" | sed -n 's/^[[:space:]]*"tag_name": "\([^"]*\)",*[[:space:]]*$/\1/p' | head -n 1)
printf '%s\n' "$tag" | LC_ALL=C grep -Eq '^v[0-9]+\.[0-9]+\.[0-9]+(-alpha\.[0-9]+)?$' || { echo 'Não foi possível identificar a última release.' >&2; exit 1; }
version=${tag#v}
name=crewsync-$version-$arch
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT HUP INT TERM
url=https://github.com/goncalvesl/crewsync/releases/download/$tag/$name.tar.gz
printf 'Baixando CrewSync %s (%s)…\n' "$version" "$arch"
curl --proto '=https' --proto-redir '=https' --tlsv1.2 -fsSL --connect-timeout 15 --max-time 600 "$url" -o "$work/package.tar.gz"
curl --proto '=https' --proto-redir '=https' --tlsv1.2 -fsSL --connect-timeout 15 --max-time 60 "$url.sha256" -o "$work/checksum"
expected=$(sed -n '1s/ .*//p' "$work/checksum")
printf '%s\n' "$expected" | LC_ALL=C grep -Eq '^[0-9a-f]{64}$' || { echo 'Checksum inválido.' >&2; exit 1; }
printf '%s  %s\n' "$expected" "$work/package.tar.gz" | sha256sum -c -
tar -xzf "$work/package.tar.gz" -C "$work"
# Não consumir o restante do script como resposta quando executado com curl | sh.
sh "$work/$name/setup.sh" "$role" </dev/tty
