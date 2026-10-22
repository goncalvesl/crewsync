#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-only
set -eu
[ "$(id -u)" -eq 0 ] || { echo 'Execute com sudo.' >&2; exit 1; }
source_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
updating=no
if [ -e /etc/opt/crewsync/server.toml ]; then
    systemctl stop crewsync-server.service
    updating=yes
fi
install -d -m755 /opt/crewsync
for item in bin docs config scripts LICENSE README.md THIRD-PARTY-NOTICES.txt; do
    [ ! -e "$source_dir/$item" ] || cp -a "$source_dir/$item" /opt/crewsync/
done
install -m755 "$source_dir/scripts/server-system-setup.sh" /opt/crewsync/server-system-setup.sh
install -d -m755 /opt/crewsync/share
install -m755 "$source_dir/scripts/pilot-cert.sh" /opt/crewsync/share/pilot-cert.sh
install -d -m755 /etc/systemd/system
cat > /etc/systemd/system/crewsync-server.service <<'UNIT'
[Unit]
Description=CrewSync server (upstream tar.gz)
After=network-online.target
Wants=network-online.target
StartLimitIntervalSec=300
StartLimitBurst=3
[Service]
Type=simple
User=crewsync
Group=crewsync
ExecStart=/opt/crewsync/bin/crewsync-server serve /etc/opt/crewsync/server.toml 0.0.0.0 8443 /etc/opt/crewsync/tls/server.crt /etc/opt/crewsync/tls/server.key
Restart=on-failure
RestartSec=5
UMask=0077
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/opt/crewsync /etc/opt/crewsync
[Install]
WantedBy=multi-user.target
UNIT
if [ "$updating" = yes ]; then
    systemctl daemon-reload
    systemctl restart crewsync-server.service
    printf 'CrewSync atualizado; configuração e dados foram preservados.\n'
else
    CREWSYNC_ETC_DIR=/etc/opt/crewsync CREWSYNC_VAR_DIR=/var/opt/crewsync CREWSYNC_SHARE_DIR=/opt/crewsync/share PATH=/opt/crewsync/bin:$PATH sh /opt/crewsync/server-system-setup.sh
fi
