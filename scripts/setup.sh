#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-only
# Assistente local, também incluído em cada pacote de release.
set -eu
umask 077
fail() { printf '%s\n' "$*" >&2; exit 1; }
case "${1:-}" in server|client) role=$1 ;; *) fail 'Uso: ./setup.sh server|client' ;; esac
[ "$#" -eq 1 ] || fail 'Informe somente server ou client.'
package=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
for command in install sed mkdir chmod nohup realpath grep awk uname; do command -v "$command" >/dev/null || fail "Instale $command e tente novamente."; done
if [ "$role" = server ]; then
    command -v openssl >/dev/null || fail 'Instale OpenSSL e tente novamente (Arch: sudo pacman -S openssl; Debian/Ubuntu: sudo apt install openssl).'
    command -v curl >/dev/null || fail 'Instale curl e tente novamente.'
fi
ask() {
    printf '%s [%s]: ' "$1" "$2" >&2
    IFS= read -r answer || fail 'Entrada encerrada; execute em um terminal.'
    [ -n "$answer" ] || answer=$2
}
path() {
    case "$1" in /*) ;; *) fail 'Use caminhos absolutos, começando com /.' ;; esac
    if printf '%s' "$1" | LC_ALL=C grep -q '[[:cntrl:]]'; then fail 'Caminho contém caracteres de controle.'; fi
    case "$1" in /|*'/../'*|*/..|*'/./'*|*/.|*'
'*) fail 'Caminho inválido.' ;; esac
}
canonical() { path "$1"; realpath -m -- "$1"; }
new_path() { path "$1"; [ ! -e "$1" ] && [ ! -L "$1" ] || fail "Já existe: $1. Nada foi sobrescrito; use outro diretório ou siga Atualizar no manual."; }
# Escapes independentes para TOML e systemd (sem eval; argumentos literais).
toml() { printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'; }
unit_arg() { printf '"'; printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g; s/%/%%/g; s/\$/$$/g'; printf '" '; }
separate() { case "$1/" in "$2/"*) fail 'Diretórios de configuração e dados devem ser separados.' ;; esac; case "$2/" in "$1/"*) fail 'Diretórios de configuração e dados devem ser separados.' ;; esac; }
printf '\nInstalação do CrewSync — %s\nPressione Enter para aceitar cada sugestão. Sem sudo.\n' "$role"
ask 'Diretório de instalação dos programas' "$HOME/.local"; prefix=$(canonical "$answer")
ask 'Diretório de configuração' "$HOME/.config/crewsync-$role"; config=$(canonical "$answer"); new_path "$config"
service=crewsync-$role.service
unit_dir=${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user
[ ! -e "$unit_dir/$service" ] && [ ! -L "$unit_dir/$service" ] || fail "Já existe $unit_dir/$service; siga Atualizar no manual."
manager=background
if command -v systemctl >/dev/null && systemctl --user show-environment >/dev/null 2>&1; then manager=systemd; fi
if [ "$role" = server ]; then
    ask 'Diretório dos dados do servidor (novo)' "$HOME/.local/share/crewsync-server"; data=$(canonical "$answer"); new_path "$data"; separate "$config" "$data"
    printf 'Informe o endereço usado pelos clientes: domínio (sync.empresa.com), IPv4 público ou IP de VPN.\n' >&2
    ask 'Domínio ou IPv4 de acesso (sem https:// e sem porta)' ''; server_host=$answer
    sh "$package/scripts/pilot-cert.sh" --check-address "$server_host"
    ask 'Porta HTTPS' 8443; port=$answer
    case "$port" in ''|*[!0-9]*) fail 'Porta inválida.' ;; esac
    [ "${#port}" -le 5 ] && [ "$port" -ge 1024 ] && [ "$port" -le 65535 ] || fail 'Use uma porta entre 1024 e 65535.'
    ask 'Primeiro usuário (letras, números, _ ou -)' usuario; user=$answer
    printf '%s\n' "$user" | LC_ALL=C grep -Eq '^[a-zA-Z0-9_-]{1,64}$' || fail 'Nome de usuário inválido.'
    ask 'Nome da pasta compartilhada (letras, números, _ ou -)' Obras; share=$answer
    printf '%s\n' "$share" | LC_ALL=C grep -Eq '^[a-zA-Z0-9_-]{1,64}$' || fail 'Nome da pasta inválido.'
else
    ask 'Raiz local para as pastas compartilhadas' "$HOME/CrewSync"; [ ! -L "$answer" ] || fail 'A pasta local não pode ser um link simbólico.'; data=$(canonical "$answer"); separate "$config" "$data"
    [ ! -L "$data" ] || fail 'A pasta local não pode ser um link simbólico.'
    ask 'Endereço HTTPS do servidor (ex.: https://sync.empresa.com:8443)' ''; url=$answer
    case "$url" in https://*) ;; *) fail 'O endereço deve começar com https://.' ;; esac
    ask 'Arquivo ca.crt recebido do administrador' "$HOME/Downloads/ca.crt"; ca=$answer; path "$ca"
    [ -f "$ca" ] || fail 'Certificado não encontrado. Copie ca.crt do servidor por um meio confiável.'
    ask 'Nome deste computador' "$(uname -n)"; device=$answer
fi
mkdir -p "$(dirname -- "$config")"
mkdir -m 700 "$config"
PREFIX=$prefix sh "$package/install.sh" "--$role"
if [ "$role" = server ]; then
    cat > "$config/server.toml" <<CONFIG
version = 1
[server]
storage_root = "$(toml "$data")"
[users.$user]
enabled = true
[shares.$share.permissions]
$user = "write"
CONFIG
    "$prefix/bin/crewsync-server" check-config "$config/server.toml"
    sh "$package/scripts/pilot-cert.sh" "$config/tls" "$server_host"
    set -- "$prefix/bin/crewsync-server" serve "$config/server.toml" 0.0.0.0 "$port" "$config/tls/server.crt" "$config/tls/server.key"
else
    install -m 600 "$ca" "$config/ca.crt"
    saved_tty=
    if [ -t 0 ]; then saved_tty=$(stty -g); stty -echo; trap 'stty "$saved_tty"' EXIT HUP INT TERM; fi
    printf 'Convite de ativação (entrada oculta): ' >&2
    IFS= read -r invite || fail 'Convite não informado.'
    if [ -n "$saved_tty" ]; then stty "$saved_tty"; trap - EXIT HUP INT TERM; fi
    printf '\n' >&2
    printf '%s\n' "$invite" | "$prefix/bin/crewsync-client" activate "$url" "$config/ca.crt" "$config/session.json" "$device"
    unset invite
    mkdir -p "$data"
    "$prefix/bin/crewsync-client" sync-root "$config/session.json" "$data"
    set -- "$prefix/bin/crewsync-client" watch-root "$config/session.json" "$data" 5
fi
# Arquivo privado com argumentos literais, usado pelos dois modos de serviço.
{
    printf '#!/bin/sh\nexec '
    for argument do printf "'"; printf '%s' "$argument" | sed "s/'/'\\\\''/g"; printf "' "; done
    printf '\n'
} > "$config/start.sh"
chmod 700 "$config/start.sh"
if [ "$manager" = systemd ]; then
    set -- /bin/sh "$config/start.sh"
    mkdir -p "$unit_dir"
    {
        printf '[Unit]\nDescription=CrewSync %s\nStartLimitIntervalSec=300\nStartLimitBurst=3\n[Service]\nType=simple\nExecStart=' "$role"
        for argument do unit_arg "$argument"; done
        printf '\nRestart=on-failure\nRestartSec=5\nUMask=0077\nNoNewPrivileges=true\n[Install]\nWantedBy=default.target\n'
    } > "$unit_dir/$service"
    systemctl --user daemon-reload
    systemctl --user enable --now "$service"
else
    nohup "$config/start.sh" > "$config/service.log" 2>&1 </dev/null &
    echo "$!" > "$config/service.pid"
fi
if [ "$role" = server ]; then
    ready=no
    for attempt in 1 2 3 4 5 6 7 8 9 10; do
        if curl --noproxy '*' --cacert "$config/tls/ca.crt" -sS --connect-timeout 1 --max-time 2 "https://127.0.0.1:$port/" -o /dev/null 2>/dev/null; then ready=yes; break; fi
        sleep 1
    done
    [ "$ready" = yes ] || fail "O servidor não iniciou. Consulte $( [ "$manager" = systemd ] && printf 'journalctl --user -u %s' "$service" || printf '%s/service.log' "$config" )."
    printf '\nServidor pronto: https://%s:%s\nCopie apenas este certificado para os clientes: %s/tls/ca.crt\nConvite para o primeiro cliente (válido por 1 hora):\n' "$server_host" "$port" "$config"
    printf 'Pasta compartilhada: %s\n' "$share"
    "$prefix/bin/crewsync-server" invite "$config/server.toml" "$user" 3600
    printf 'Outro convite: "%s/bin/crewsync-server" invite "%s/server.toml" "%s" 3600\n' "$prefix" "$config" "$user"
    printf 'Se houver firewall, libere TCP %s no servidor e no painel do provedor para os clientes.\n' "$port"
else
    printf '\nCliente ativado e sincronizando: %s\n' "$data"
fi
if [ "$manager" = systemd ]; then
    systemctl --user is-active --quiet "$service" || fail 'O serviço parou; consulte journalctl --user.'
    printf 'Serviço: systemctl --user status %s\nInício automático durante sua sessão de usuário; para funcionar sem login: sudo loginctl enable-linger %s\n' "$service" "$(id -un)"
else
    kill -0 "$(cat "$config/service.pid")" 2>/dev/null || fail "O processo parou; consulte $config/service.log."
    printf 'Processo em segundo plano. Log: %s/service.log\nApós reiniciar o computador, execute: "%s/start.sh"\n' "$config" "$config"
fi
