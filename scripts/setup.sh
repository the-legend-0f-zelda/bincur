#!/usr/bin/env bash
set -euo pipefail

# === Paths ===
GROUP="input"
RULE_FILE="/etc/udev/rules.d/99-bincur-uinput.rules"
CONF_DIR="${BINCUR_CONF_HOME:-$HOME/.config/bincur}"
SERVICE_DIR="$HOME/.config/systemd/user"
SERVICE_FILE="$SERVICE_DIR/bincur.service"
BIN_DEST="$HOME/.local/bin/bincur"

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN_SRC="$REPO_ROOT/target/release/bincur"

# === Args ===
ACTION="install"
case "${1:-}" in
    "")            ACTION="install" ;;
    --uninstall)   ACTION="uninstall" ;;
    -h|--help)
        cat <<EOF
Usage: $0 [--uninstall|--help]

  (no args)     Full setup:
                  1. Add user to '$GROUP' group
                  2. Install udev rule for /dev/uinput
                  3. Install binary to $BIN_DEST
                  4. Write default configs to $CONF_DIR (skipped if present)
                  5. Install systemd user service ($SERVICE_FILE)

  --uninstall   Reverse the setup. Existing user configs in $CONF_DIR are kept.
                Group membership in '$GROUP' is also kept since other tools may rely on it.
EOF
        exit 0
        ;;
    *)
        echo "Unknown option: $1" >&2
        echo "Run '$0 --help' for usage." >&2
        exit 1
        ;;
esac

if [[ $EUID -eq 0 ]]; then
    echo "Don't run this script as root. It will sudo individual steps as needed." >&2
    exit 1
fi

# Cache sudo creds upfront so the user isn't prompted multiple times.
sudo -v

# ============================================================
# INSTALL
# ============================================================
if [[ "$ACTION" == "install" ]]; then

    # --- 1. input group ---
    RELOGIN_NEEDED=0
    if id -nG | tr ' ' '\n' | grep -qx "$GROUP"; then
        echo "[=] Already a member of '$GROUP'"
    else
        echo "[+] Adding $USER to '$GROUP'"
        sudo usermod -aG "$GROUP" "$USER"
        RELOGIN_NEEDED=1
    fi

    # --- 2. udev rule for /dev/uinput ---
    echo "[+] Writing $RULE_FILE"
    sudo tee "$RULE_FILE" >/dev/null <<EOF
KERNEL=="uinput", GROUP="$GROUP", MODE="0660", OPTIONS+="static_node=uinput"
EOF
    sudo udevadm control --reload-rules
    if [[ -e /dev/uinput ]]; then
        sudo chgrp "$GROUP" /dev/uinput
        sudo chmod 0660 /dev/uinput
    fi

    # --- 3. binary ---
    if [[ ! -f "$BIN_SRC" ]]; then
        echo "[!] Binary not found at $BIN_SRC" >&2
        echo "    Build first: cargo build --release" >&2
        exit 1
    fi
    mkdir -p "$(dirname "$BIN_DEST")"
    install -m 0755 "$BIN_SRC" "$BIN_DEST"
    echo "[+] Installed binary: $BIN_DEST"

    # --- 4. default config files (don't overwrite) ---
    mkdir -p "$CONF_DIR"
    if [[ ! -f "$CONF_DIR/vmouse.conf" ]]; then
        cat > "$CONF_DIR/vmouse.conf" <<'EOF'
# Grab mode trigger keys
# When TRUE, key-combo input events bound to that mode are consumed and not propagated.
grab_linear : FALSE
grab_logarithmic : TRUE
grab_scroll : TRUE

# Cursor step size
step_size_x : 256
step_size_y : 256

# Wheel scroll distance
scroll_dist_x : 2
scroll_dist_y : 2
EOF
        echo "[+] Wrote default $CONF_DIR/vmouse.conf"
    else
        echo "[=] $CONF_DIR/vmouse.conf already exists; skipping"
    fi

    if [[ ! -f "$CONF_DIR/keymap.conf" ]]; then
        cat > "$CONF_DIR/keymap.conf" <<'EOF'
# Terminate process
leftctrl+q : EXIT

# Trigger virtual mouse mode
leftalt : LINEAR_MODE
leftalt+leftshift : LOGARITHMIC_MODE
leftalt+c : SCROLL_MODE

# Move cursor or scroll
i : MOVE_UP
k : MOVE_DOWN
j : MOVE_LEFT
l : MOVE_RIGHT

# Mouse click
space : CLICK_LEFT
semicolon : CLICK_RIGHT
EOF
        echo "[+] Wrote default $CONF_DIR/keymap.conf"
    else
        echo "[=] $CONF_DIR/keymap.conf already exists; skipping"
    fi

    # --- 5. systemd user service ---
    mkdir -p "$SERVICE_DIR"
    cat > "$SERVICE_FILE" <<EOF
[Unit]
Description=Binary Cursor - keyboard-controlled mouse emulator

[Service]
Type=simple
ExecStart=$BIN_DEST
Restart=on-failure
EOF
    echo "[+] Installed user service: $SERVICE_FILE"

    systemctl --user daemon-reload

    echo
    echo "Setup complete."
    if [[ "$RELOGIN_NEEDED" == "1" ]]; then
        echo "Note: log out and back in for the '$GROUP' group change to take effect."
    fi

# ============================================================
# UNINSTALL
# ============================================================
else

    echo "[+] Stopping and disabling bincur.service"
    systemctl --user disable --now bincur.service 2>/dev/null || true
    rm -f "$SERVICE_FILE"
    systemctl --user daemon-reload

    echo "[+] Removing binary $BIN_DEST"
    rm -f "$BIN_DEST"

    echo "[+] Removing $RULE_FILE"
    sudo rm -f "$RULE_FILE"
    sudo udevadm control --reload-rules

    echo
    echo "Uninstall complete."
    echo "Note: configs in $CONF_DIR were kept (in case of customizations)."
    echo "      '$GROUP' group membership was kept (other tools may rely on it)."
fi
