#!/usr/bin/env bash

PLUGIN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

pick_local_binary() {
    local root="$1"
    local bin_path="$root/bin/tmux-agent-sidebar"
    local release_path="$root/target/release/tmux-agent-sidebar"

    if [[ -x "$bin_path" && -x "$release_path" ]]; then
        if [[ "$release_path" -nt "$bin_path" ]]; then
            printf '%s\n' "$release_path"
        else
            printf '%s\n' "$bin_path"
        fi
        return 0
    fi

    if [[ -x "$release_path" ]]; then
        printf '%s\n' "$release_path"
        return 0
    fi

    if [[ -x "$bin_path" ]]; then
        printf '%s\n' "$bin_path"
        return 0
    fi

    return 1
}

# Prefer the newer local artifact so `cargo build --release` in a linked
# working copy wins over an older downloaded binary in `bin/`.
SIDEBAR_BINARY="$(pick_local_binary "$PLUGIN_DIR" || true)"
if [[ -z "$SIDEBAR_BINARY" ]] && command -v "tmux-agent-sidebar" &>/dev/null; then
    SIDEBAR_BINARY="tmux-agent-sidebar"
fi

if [[ -z "$SIDEBAR_BINARY" ]]; then
    tmux run-shell -b "bash '$PLUGIN_DIR/install-wizard.sh'"
    exit 0
fi

INSTALLED_VERSION="$("$SIDEBAR_BINARY" version 2>/dev/null)"
EXPECTED_VERSION="$(sed -n 's/^version *= *"\(.*\)"/\1/p' "$PLUGIN_DIR/Cargo.toml")"

if [[ -n "$EXPECTED_VERSION" && "$INSTALLED_VERSION" != "$EXPECTED_VERSION" ]]; then
    tmux run-shell -b "SIDEBAR_UPDATE=1 bash '$PLUGIN_DIR/install-wizard.sh'"
    exit 0
fi

tmux set -g @agent_sidebar_bin "$SIDEBAR_BINARY"

tmux source-file "$PLUGIN_DIR/agent-sidebar.conf"
