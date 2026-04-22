#!/usr/bin/env bash
# Thin wrapper: delegates to the Rust binary. Called by Claude Code /
# Codex hooks (settings.json).
#
# Why this file exists even though `tmux-agent-sidebar setup` can emit
# absolute binary paths:
#
# 1. Late binding. settings.json only needs to know where `hook.sh`
#    lives. The actual binary is resolved fresh on every hook fire, so
#    the user can move or rebuild the binary (bin/ ↔ target/release/,
#    relocate the plugin dir, swap install methods) without having to
#    regenerate their agent config. Without this indirection, any
#    setup-generated path becomes a stale snapshot the moment the
#    binary moves.
#
# 2. Graceful absence. If the binary is missing — during a rebuild,
#    mid-uninstall, or on a fresh clone before `cargo build` — this
#    script exits 0 silently, so the agent session never sees a hook
#    failure. A direct binary invocation would surface "no such file"
#    errors into the user's workflow.
#
# Keep this wrapper small and side-effect-free. Any logic that needs to
# know event semantics belongs in the Rust `hook` subcommand.
PLUGIN_DIR="$(cd "$(dirname "$0")" && pwd -P)"
# Fallback location used when this script is executed from a Claude Code
# plugin install (e.g. `${CLAUDE_PLUGIN_ROOT}/hook.sh`). The plugin cache
# never contains the binary, so hop over to the tmux plugin directory
# where TPM placed it.
TPM_DIR="$HOME/.tmux/plugins/tmux-agent-sidebar"

pick_local_binary() {
  local root="$1"
  local bin_path="$root/bin/tmux-agent-sidebar"
  local release_path="$root/target/release/tmux-agent-sidebar"

  if [ -x "$bin_path" ] && [ -x "$release_path" ]; then
    if [ "$release_path" -nt "$bin_path" ]; then
      printf '%s\n' "$release_path"
    else
      printf '%s\n' "$bin_path"
    fi
    return 0
  fi

  if [ -x "$release_path" ]; then
    printf '%s\n' "$release_path"
    return 0
  fi

  if [ -x "$bin_path" ]; then
    printf '%s\n' "$bin_path"
    return 0
  fi

  return 1
}

# Prefer the newer local artifact so `cargo build --release` in a linked
# working copy wins over an older downloaded binary in `bin/`.
BIN="$(pick_local_binary "$PLUGIN_DIR" || pick_local_binary "$TPM_DIR" || true)"
if [ -z "$BIN" ] && command -v tmux-agent-sidebar &>/dev/null; then
  BIN="tmux-agent-sidebar"
fi
if [ -z "$BIN" ]; then
  exit 0
fi
exec "$BIN" hook "$@"
