#!/usr/bin/env bash
set -euo pipefail

#要先執行安裝下以列套件，才能成功編譯：
#sudo mv /etc/apt/sources.list.d/archive_uri-http_apt_kubernetes_io_-noble.list /etc/apt/sources.list.d/archive_uri-http_apt_kubernetes_io_-noble.list.disabled && sudo apt-get update && sudo apt-get install -y musl-tools musl-dev
#sudo apt-get update && sudo apt-get install -y musl-tools musl-dev

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 保證 cargo 呼叫到 rustup toolchain 內的 rustc（避免誤用 /usr/local/bin/rustc 1.82）
RUSTUP_BIN="$(command -v rustup)"
CARGO_BIN="$(dirname "$RUSTUP_BIN")/cargo"
export RUSTUP_TOOLCHAIN="stable-x86_64-unknown-linux-gnu"
export RUSTC="$($RUSTUP_BIN which --toolchain stable rustc)"

echo "[corn-dbio-kit] compile release binary"
# 強制使用 rustup 管理的 stable toolchain，避免抓到 /usr/local/bin/cargo
"$RUSTUP_BIN" run stable "$CARGO_BIN" build --release --manifest-path "$SCRIPT_DIR/Cargo.toml"
cp "$SCRIPT_DIR/target/release/corn-dbio-kit" "$SCRIPT_DIR/corn-dbio-kit"
chmod +x "$SCRIPT_DIR/corn-dbio-kit"
echo "[corn-dbio-kit] output => $SCRIPT_DIR/corn-dbio-kit"
