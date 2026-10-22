#!/bin/sh
# Executar fora de sandbox que proíba sockets TCP.
set -eu
cd "$(dirname "$0")/.."
cargo build --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
/usr/bin/python3 -I tests/e2e/tls_memory.py
/usr/bin/python3 -I tests/e2e/cert_input.py
timeout 180s cargo test -p crewsync-server --test pilot --locked -- --ignored

python3 scripts/build-site.py
python3 tests/site/check.py target/site
python3 tests/site/test_build.py

python3 tests/packaging/test_install.py

python3 -m unittest discover -s tests/release
python3 scripts/prepare-release.py --check
