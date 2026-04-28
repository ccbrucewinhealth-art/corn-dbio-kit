#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 保證 cargo 呼叫到 rustup toolchain 內的 rustc（避免誤用 /usr/local/bin/rustc 1.82）
RUSTUP_BIN="$(command -v rustup)"
CARGO_BIN="$(dirname "$RUSTUP_BIN")/cargo"
export RUSTUP_TOOLCHAIN="stable-x86_64-unknown-linux-gnu"
export RUSTC="$($RUSTUP_BIN which --toolchain stable rustc)"


"$RUSTUP_BIN" target add x86_64-unknown-linux-musl

echo "[corn-dbio-kit] all-in-one compile start (standalone in current project)"

# 使用各資料庫 native Rust connector；SQLite 採 bundled 靜態函式庫。
# 不再連結 unixODBC/libodbc，因此 musl 靜態編譯不需因 ODBC 原生庫回退。
if "$RUSTUP_BIN" run stable "$CARGO_BIN" build --release --manifest-path "$SCRIPT_DIR/Cargo.toml" --target x86_64-unknown-linux-musl; then
  cp "$SCRIPT_DIR/target/x86_64-unknown-linux-musl/release/corn-dbio-kit" "$SCRIPT_DIR/corn-dbio-kit"
  echo "[corn-dbio-kit] built with target=x86_64-unknown-linux-musl"
else
  echo "[corn-dbio-kit] musl build failed, fallback to gnu target"
  "$RUSTUP_BIN" run stable "$CARGO_BIN" build --release --manifest-path "$SCRIPT_DIR/Cargo.toml" --target x86_64-unknown-linux-gnu
  cp "$SCRIPT_DIR/target/x86_64-unknown-linux-gnu/release/corn-dbio-kit" "$SCRIPT_DIR/corn-dbio-kit"
  echo "[corn-dbio-kit] built with target=x86_64-unknown-linux-gnu (fallback)"
fi

chmod +x "$SCRIPT_DIR/corn-dbio-kit"
echo "[corn-dbio-kit] all-in-one compile done => $SCRIPT_DIR/corn-dbio-kit"
