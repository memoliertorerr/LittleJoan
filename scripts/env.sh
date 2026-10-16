# Source this before any npm, node, cargo, rustup, or tauri command:
#
#   source scripts/env.sh
#
# It keeps every toolchain, cache, temp file, and the dev app's data inside this repo's
# .cache/ folder (gitignored), so building Little Joan never writes to the rest of the machine.
# Works from bash and zsh. CI sources it too, so local and CI builds use the same layout.

LJ_ROOT_POSIX="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.." && pwd)"
LJ_ROOT="$LJ_ROOT_POSIX"
# Git Bash on Windows: native tools (cargo, node) need D:/a/... paths, but PATH needs /d/a/...
case "$(uname -s)" in
  MINGW* | MSYS* | CYGWIN*) LJ_ROOT="$(cygpath -m "$LJ_ROOT_POSIX")" ;;
esac
export LJ_ROOT
export LJ_CACHE="$LJ_ROOT/.cache"

# Node / npm
export npm_config_cache="$LJ_CACHE/npm"
export npm_config_update_notifier=false
export npm_config_fund=false
export npm_config_audit=false

# Temp files (npm, cargo build scripts, the linker, xcrun) and the clang module cache.
export TMPDIR="$LJ_CACHE/tmp"
export CLANG_MODULE_CACHE_PATH="$LJ_CACHE/clang-modules"

# Rust: toolchain, registry, and build output all stay in .cache/.
export RUSTUP_HOME="$LJ_CACHE/rustup"
export CARGO_HOME="$LJ_CACHE/cargo"
export CARGO_TARGET_DIR="$LJ_CACHE/target"
# Incremental artifacts are large and only speed up rebuilds of our own small crate.
export CARGO_INCREMENTAL=0

# The app reads this to keep its settings (and, where the webview allows it, its browsing data)
# out of the user's real profile folders while developing. Unset in shipped builds.
export LITTLEJOAN_DATA_DIR="$LJ_CACHE/dev-data"

# gh: no update-check state files outside the repo.
export GH_NO_UPDATE_NOTIFIER=1

case ":$PATH:" in
  *":$LJ_ROOT_POSIX/.cache/cargo/bin:"*) ;;
  *) export PATH="$LJ_ROOT_POSIX/.cache/cargo/bin:$LJ_ROOT_POSIX/node_modules/.bin:$PATH" ;;
esac

mkdir -p "$TMPDIR" "$CLANG_MODULE_CACHE_PATH" "$npm_config_cache" "$LITTLEJOAN_DATA_DIR"
