#!/bin/sh
# Teste destrutivo apenas dentro de tmpfs descartável de 1 MiB, nunca no host.
set -eu
if [ "$#" -ne 1 ]; then
    echo 'Uso: check-disk-full.sh DIRETORIO_DEPS_MUSL' >&2
    exit 2
fi
deps_dir=$(CDPATH= cd -- "$1" && pwd)
test_binary=$(ls "$deps_dir"/disk_full-* | awk '!/\.d$/ {print; exit}')
test -x "$test_binary"
docker run --rm --network none --read-only --tmpfs /full:size=1m,mode=0700 \
    -e CREWSYNC_FULL_DISK_DIR=/full -v "$deps_dir:/tests:ro" \
    busybox "/tests/$(basename "$test_binary")" --ignored
