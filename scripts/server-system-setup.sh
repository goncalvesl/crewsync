#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-only
# Provisiona o servidor de sistema instalado por um pacote Linux ou tar.gz.
set -eu
umask 077
fail() { printf '%s\n' "$*" >&2; exit 1; }
[ "$(id -u)" -eq 0 ] || fail 'Execute com sudo: crewsync-server-setup.'
command -v crewsync-server >/dev/null || fail 'crewsync-server não está no PATH.'
command -v openssl >/dev/null || fail 'Instale OpenSSL para gerar o certificado.'
command -v systemctl >/dev/null || fail 'Esta instalação exige systemd.'
command -v runuser >/dev/null || fail 'Instale util-linux para executar as operações administrativas como a conta de serviço.'
etc_dir=${CREWSYNC_ETC_DIR:-/etc/crewsync}
var_dir=${CREWSYNC_VAR_DIR:-/var/lib/crewsync}
share_dir=${CREWSYNC_SHARE_DIR:-/usr/share/crewsync}
server_bin=$(command -v crewsync-server)
if ! getent passwd crewsync >/dev/null; then
    nologin_shell=$(command -v nologin || printf /usr/sbin/nologin)
    if command -v useradd >/dev/null; then useradd --system --user-group --home-dir "$var_dir" --shell "$nologin_shell" crewsync
    elif command -v adduser >/dev/null; then adduser --system --home "$var_dir" --no-create-home --group --disabled-login crewsync
    else fail 'Não encontrei useradd/adduser para criar a conta de serviço.'; fi
fi
ask() { printf '%s [%s]: ' "$1" "$2" >&2; IFS= read -r answer || fail 'Entrada encerrada.'; [ -n "$answer" ] || answer=$2; }
printf 'Configuração inicial do servidor CrewSync (Linux/systemd).\n'
[ ! -e "$etc_dir/server.toml" ] || fail "Já existe $etc_dir/server.toml; preservado. Edite-o e reinicie o serviço."
printf 'Conexão: 1 = HTTPS direto (CA privada gerada); 2 = HTTPS por proxy reverso.\n'
ask 'Modo' 1; mode=$answer; case "$mode" in 1|2) ;; *) fail 'Escolha 1 ou 2.' ;; esac
ask 'Domínio ou IPv4 usado pelos clientes (sem esquema/porta)' sync.example.com; host=$answer
printf '%s\n' "$host" | grep -Eq '^[A-Za-z0-9.-]+$' || fail 'Endereço inválido.'
sh "$share_dir/pilot-cert.sh" --check-address "$host"
mkdir -p "$etc_dir/tls" "$var_dir"
chmod 2770 "$etc_dir"
chmod 750 "$etc_dir/tls"
chmod 700 "$var_dir"
chown root:crewsync "$etc_dir" "$etc_dir/tls"
chown crewsync:crewsync "$var_dir"
cat > "$etc_dir/server.toml" <<EOF
version = 1
[server]
storage_root = "$var_dir"
EOF
chmod 660 "$etc_dir/server.toml"; chown root:crewsync "$etc_dir/server.toml"
"$server_bin" check-config "$etc_dir/server.toml"
if [ "$mode" = 1 ]; then
    sh "$share_dir/pilot-cert.sh" "$etc_dir/tls" "$host"
    chown root:crewsync "$etc_dir/tls/server.key"
    chmod 640 "$etc_dir/tls/server.key"
    command_line="ExecStart=$server_bin serve $etc_dir/server.toml 0.0.0.0 8443 $etc_dir/tls/server.crt $etc_dir/tls/server.key"
    public_url="https://$host:8443"
else
    command_line="ExecStart=$server_bin serve-http $etc_dir/server.toml 127.0.0.1 8080"
    public_url="https://$host"
    printf '%s {\n    reverse_proxy 127.0.0.1:8080\n}\n' "$host" > "$etc_dir/Caddyfile"
    chmod 640 "$etc_dir/Caddyfile"; chown root:crewsync "$etc_dir/Caddyfile"
fi
if [ "$mode" = 2 ]; then
    mkdir -p /etc/systemd/system/crewsync-server.service.d
    cat > /etc/systemd/system/crewsync-server.service.d/20-crewsync-proxy.conf <<UNIT
[Service]
ExecStart=
$command_line
UNIT
fi
systemctl daemon-reload
systemctl enable --now crewsync-server.service
admin_key=$(runuser -u crewsync -- "$server_bin" admin-key "$etc_dir/server.toml" "$public_url")
printf '\nServidor iniciado: %s\nPainel: %s/\nChave administrativa (guarde em local seguro):\n%s\n' "$public_url" "$public_url" "$admin_key"
unset admin_key
printf '\nAcesse o painel para criar usuários, compartilhamentos e permissões; depois gere um convite para cada dispositivo.\nConfiguração: %s/server.toml\nRaiz de dados: %s\nLogs: journalctl -u crewsync-server\n' "$etc_dir" "$var_dir"
if [ "$mode" = 2 ]; then printf 'Exemplo Caddy: %s/Caddyfile. O proxy deve encaminhar HTTPS para 127.0.0.1:8080.\n' "$etc_dir"; else printf 'CA para os clientes: %s/tls/ca.crt. Libere TCP 8443 no firewall e na nuvem.\n' "$etc_dir"; fi
