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
    printf 'Conexão: 1 = HTTPS direto; 2 = HTTPS por proxy; 3 = HTTP para testes.\n' >&2
    ask 'Modo de conexão' 1; mode=$answer
    case "$mode" in 1|2|3) ;; *) fail 'Modo inválido; escolha 1, 2 ou 3.' ;; esac
    printf 'Informe o endereço usado pelos clientes: domínio (sync.empresa.com), IPv4 público ou IP de VPN.\n' >&2
    ask 'Domínio ou IPv4 de acesso (sem https:// e sem porta)' "$( [ "$mode" = 3 ] && printf 127.0.0.1 || true )"; server_host=$answer
    sh "$package/scripts/pilot-cert.sh" --check-address "$server_host"
    if [ "$mode" = 1 ]; then default_port=8443; else default_port=8080; fi
    ask 'Porta do CrewSync (interna no modo proxy)' "$default_port"; port=$answer
    case "$port" in ''|*[!0-9]*) fail 'Porta inválida.' ;; esac
    [ "${#port}" -le 5 ] && [ "$port" -ge 1024 ] && [ "$port" -le 65535 ] || fail 'Use uma porta entre 1024 e 65535.'
    bind=127.0.0.1
    if [ "$mode" = 1 ]; then
        bind=0.0.0.0
        ask 'Certificado HTTPS existente (caminho absoluto) ou gerar CA privada' gerar; cert=$answer
        if [ "$cert" = gerar ]; then
            command -v openssl >/dev/null || fail 'Instale OpenSSL para gerar certificados, ou forneça um certificado existente.'
        else
            path "$cert"; [ -f "$cert" ] || fail 'Certificado não encontrado.'
            ask 'Chave privada do certificado (caminho absoluto)' ''; key=$answer
            path "$key"; [ -f "$key" ] || fail 'Chave privada não encontrada.'
        fi
    elif [ "$mode" = 3 ]; then
        ask 'IPv4 de escuta (127.0.0.1 para testes locais)' 127.0.0.1; bind=$answer
        sh "$package/scripts/pilot-cert.sh" --check-address "$bind"
        case "$bind" in *[!0-9.]*) fail 'Use um IPv4 para a escuta HTTP.' ;; esac
        printf 'HTTP envia convites, credenciais e arquivos sem criptografia.\n' >&2
        ask 'Para habilitar HTTP sem criptografia, digite HTTP' nao
        [ "$answer" = HTTP ] || fail 'HTTP não habilitado.'
    fi
else
    ask 'Raiz local para as pastas compartilhadas' "$HOME/CrewSync"; [ ! -L "$answer" ] || fail 'A pasta local não pode ser um link simbólico.'; data=$(canonical "$answer"); separate "$config" "$data"
    [ ! -L "$data" ] || fail 'A pasta local não pode ser um link simbólico.'
    ask 'Endereço do servidor (https:// ou http:// explícito para testes)' ''; url=$answer
    case "$url" in
        https://*)
            ask 'CA privada (caminho absoluto) ou system para certificado público' system; ca=$answer
            if [ "$ca" != system ]; then
                path "$ca"
                [ -f "$ca" ] || fail 'Certificado não encontrado. Copie ca.crt por um meio confiável.'
            fi
            ;;
        http://*)
            printf 'HTTP envia convites, credenciais e arquivos sem criptografia.\n' >&2
            ask 'Para habilitar HTTP sem criptografia, digite HTTP' nao
            [ "$answer" = HTTP ] || fail 'HTTP não habilitado.'
            ca=insecure-http
            ;;
        *) fail 'O endereço deve começar com https:// ou http://.' ;;
    esac
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
CONFIG
    "$prefix/bin/crewsync-server" check-config "$config/server.toml"
    if [ "$mode" = 1 ]; then
        if [ "$cert" = gerar ]; then
            sh "$package/scripts/pilot-cert.sh" "$config/tls" "$server_host"
            cert=$config/tls/server.crt; key=$config/tls/server.key
            generated_ca=yes
        else
            generated_ca=no
        fi
        set -- "$prefix/bin/crewsync-server" serve "$config/server.toml" "$bind" "$port" "$cert" "$key"
        public_url=https://$server_host:$port
    else
        set -- "$prefix/bin/crewsync-server" serve-http "$config/server.toml" "$bind" "$port"
        if [ "$mode" = 2 ]; then
            public_url=https://$server_host
            printf '%s {\n    reverse_proxy 127.0.0.1:%s\n}\n' "$server_host" "$port" > "$config/Caddyfile"
        else
            public_url=http://$server_host:$port
        fi
    fi
else
    case "$ca" in
        system|insecure-http) activation_ca=$ca ;;
        *) install -m 600 "$ca" "$config/ca.crt"; activation_ca=$config/ca.crt ;;
    esac
    saved_tty=
    if [ -t 0 ]; then saved_tty=$(stty -g); stty -echo; trap 'stty "$saved_tty"' EXIT HUP INT TERM; fi
    printf 'Convite de ativação (entrada oculta): ' >&2
    IFS= read -r invite || fail 'Convite não informado.'
    if [ -n "$saved_tty" ]; then stty "$saved_tty"; trap - EXIT HUP INT TERM; fi
    printf '\n' >&2
    printf '%s\n' "$invite" | "$prefix/bin/crewsync-client" activate "$url" "$activation_ca" "$config/session.json" "$device"
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
    # Herdar HUP ignorado antes de iniciar: o terminal pode fechar antes de nohup executar.
    trap '' HUP
    nohup "$config/start.sh" > "$config/service.log" 2>&1 </dev/null &
    echo "$!" > "$config/service.pid"
    trap - HUP
fi
if [ "$role" = server ]; then
    ready=no
    for attempt in 1 2 3 4 5 6 7 8 9 10; do
        if [ "$mode" = 1 ]; then
            if [ "$generated_ca" = yes ]; then readiness_ca=$config/tls/ca.crt; else readiness_ca=$cert; fi
            # Certificado privado ou público existente: confiar na cadeia fornecida
            # e verificar o nome de acesso, roteando somente este teste para loopback.
            if curl --noproxy '*' --cacert "$readiness_ca" --resolve "$server_host:$port:127.0.0.1" -sS --connect-timeout 1 --max-time 2 "$public_url/" -o /dev/null 2>/dev/null; then ready=yes; break; fi
        else
            probe_host=$bind
            [ "$probe_host" != 0.0.0.0 ] || probe_host=127.0.0.1
            if curl --noproxy '*' -sS --connect-timeout 1 --max-time 2 "http://$probe_host:$port/" -o /dev/null 2>/dev/null; then ready=yes; break; fi
        fi
        sleep 1
    done
    [ "$ready" = yes ] || fail "O servidor não iniciou. Consulte $( [ "$manager" = systemd ] && printf 'journalctl --user -u %s' "$service" || printf '%s/service.log' "$config" )."
    if [ "$mode" = 2 ]; then
        printf '\nBackend pronto: http://127.0.0.1:%s\nURL pública após configurar o proxy: %s\nExemplo Caddy: %s/Caddyfile\n' "$port" "$public_url" "$config"
    else
        printf '\nServidor pronto: %s\n' "$public_url"
    fi
    if [ "$mode" = 1 ] && [ "$generated_ca" = yes ]; then
        printf 'Copie apenas este certificado para os clientes: %s/tls/ca.crt\n' "$config"
    fi
    admin_key=$("$prefix/bin/crewsync-server" admin-key "$config/server.toml" "$public_url")
    printf 'Painel de administração: %s/\nChave administrativa (guarde em local seguro):\n%s\n' "$public_url" "$admin_key"
    unset admin_key
    printf 'No painel, crie os usuários e compartilhamentos, defina as permissões e gere um convite para cada dispositivo.\n'
    if [ "$mode" = 2 ]; then
        printf 'Configure o proxy HTTPS; libere somente TCP 80/443 para ele. Mantenha a porta interna privada.\n'
    elif [ "$mode" = 1 ] || [ "$bind" != 127.0.0.1 ]; then
        printf 'Se houver firewall, libere TCP %s somente para os clientes desejados.\n' "$port"
    fi
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
