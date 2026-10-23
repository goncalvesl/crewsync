#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-only
set -eu
cd "$(dirname "$0")/.."

project=crewsync-docker-smoke-$$
compose_file=compose.yaml
data_dir=$(mktemp -d "${TMPDIR:-/tmp}/crewsync-compose-data.XXXXXX")
export CREWSYNC_DATA_DIR=$data_dir
export CREWSYNC_ADMIN_PASSWORD=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
export CREWSYNC_PUBLIC_URL=https://sync.test.example
host_uid=$(id -u)
host_gid=$(id -g)
cleanup() {
    docker compose -p "$project" -f "$compose_file" down --volumes --remove-orphans >/dev/null 2>&1 || true
    docker run --rm --user 0 -v "$data_dir:/data" debian:bookworm-slim \
        sh -c "chown -R $host_uid:$host_gid /data && chmod 0700 /data" >/dev/null 2>&1 || true
    rm -rf "$data_dir"
}
trap cleanup EXIT HUP INT TERM
docker run --rm --user 0 -v "$data_dir:/data" debian:bookworm-slim \
    sh -c 'chown 10001:10001 /data && chmod 0700 /data'

docker compose -p "$project" -f "$compose_file" config --format json | python3 -c '
import json, sys
service = json.load(sys.stdin)["services"]["server"]
assert "ports" not in service, "Compose must not publish the backend on a host port"
assert service["expose"] == ["8080"]
assert service["read_only"] is True
assert len(service["volumes"]) == 2
data = next(item for item in service["volumes"] if item["target"] == "/var/lib/crewsync")
assert data["type"] == "bind", "Server data must live on the host filesystem"
'

docker compose -p "$project" -f "$compose_file" up --build --detach
container=$(docker compose -p "$project" -f "$compose_file" ps -q server)

attempt=0
while [ "$attempt" -lt 180 ]; do
    state=$(docker inspect --format '{{.State.Health.Status}}' "$container" 2>/dev/null || true)
    [ "$state" = healthy ] && break
    [ "$state" != unhealthy ] || { docker compose -p "$project" -f "$compose_file" logs; exit 1; }
    attempt=$((attempt + 1))
    sleep 1
done
[ "$state" = healthy ] || { docker compose -p "$project" -f "$compose_file" logs; exit 1; }

docker compose -p "$project" -f "$compose_file" exec -T server sh -eu -c '
    test "$(id -u)" = 10001
    test "$(stat -c %a /etc/crewsync)" = 700
    test "$(stat -c %a /etc/crewsync/server.toml)" = 600
    test "$(stat -c %a /var/lib/crewsync)" = 700
    ! grep -q "^\\[users\\." /etc/crewsync/server.toml
    ! grep -q "^\\[shares\\." /etc/crewsync/server.toml
    /usr/local/bin/crewsync-server check-config /etc/crewsync/server.toml
'
docker compose -p "$project" -f "$compose_file" exec -T server sh -eu -c '
    grep -q '"key_hash"' /var/lib/crewsync/auth/admin.json
    ! grep -q "$CREWSYNC_ADMIN_PASSWORD" /var/lib/crewsync/auth/admin.json
'
docker compose -p "$project" -f "$compose_file" exec -T server sh -eu -c '
    cat >> /etc/crewsync/server.toml <<EOF
[users.docker_smoke]
enabled = true
[shares.SmokeShare.permissions]
docker_smoke = "write"
EOF
    /usr/local/bin/crewsync-server check-config /etc/crewsync/server.toml
'
invite=$(docker compose -p "$project" -f "$compose_file" exec -T server \
    /usr/local/bin/crewsync-server invite /etc/crewsync/server.toml docker_smoke 3600)
[ -n "$invite" ]

credentials_before=$(docker compose -p "$project" -f "$compose_file" exec -T server \
    sha256sum /var/lib/crewsync/auth/admin.json | cut -d ' ' -f 1)
docker compose -p "$project" -f "$compose_file" restart server
attempt=0
while [ "$attempt" -lt 30 ]; do
    state=$(docker inspect --format '{{.State.Health.Status}}' "$container" 2>/dev/null || true)
    [ "$state" = healthy ] && break
    attempt=$((attempt + 1))
    sleep 1
done
[ "$state" = healthy ] || { docker compose -p "$project" -f "$compose_file" logs; exit 1; }
docker compose -p "$project" -f "$compose_file" exec -T server \
    test -s /var/lib/crewsync/auth/admin.json
credentials_after=$(docker compose -p "$project" -f "$compose_file" exec -T server \
    sha256sum /var/lib/crewsync/auth/admin.json | cut -d ' ' -f 1)
[ "$credentials_before" = "$credentials_after" ]
printf '%s\n' 'Docker Compose: private ports, persistent state, setup, health and restart OK.'
