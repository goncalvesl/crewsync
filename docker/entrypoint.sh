#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-only
set -eu
umask 077

config_dir=/etc/crewsync
config_file=$config_dir/server.toml
storage_root=/var/lib/crewsync
mkdir -p "$config_dir"
[ -d "$storage_root" ] || { printf '%s\n' "A raiz de dados $storage_root deve ser um bind mount já criado no host." >&2; exit 1; }
chmod 0700 "$config_dir" "$storage_root"

if [ ! -e "$config_file" ]; then
    temporary=$config_file.tmp.$$
    trap 'rm -f "$temporary"' EXIT HUP INT TERM
    cat > "$temporary" <<EOF
version = 1

[server]
storage_root = "$storage_root"
EOF
    chmod 0600 "$temporary"
    mv "$temporary" "$config_file"
    trap - EXIT HUP INT TERM
fi

if [ ! -f "$config_file" ] || [ -L "$config_file" ]; then
    printf '%s\n' 'A configuração precisa ser um arquivo regular, não um link.' >&2
    exit 1
fi
chmod 0600 "$config_file"
/usr/local/bin/crewsync-server check-config "$config_file"
/usr/local/bin/crewsync-server bootstrap-admin "$config_file" "$CREWSYNC_PUBLIC_URL"
exec /usr/local/bin/crewsync-server "$@"
