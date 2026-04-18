#!/usr/bin/env bash
set -euo pipefail

GROUP="bincur"
RULE_FILE="/etc/udev/rules.d/99-bincur.rules"
TARGET_USER="${SUDO_USER:-$USER}"

if [[ $EUID -ne 0 ]]; then
    echo "Root required. Re-running with sudo."
    exec sudo -E bash "$0" "$@"
fi

if ! getent group "$GROUP" >/dev/null; then
    echo "[+] Creating group: $GROUP"
    groupadd "$GROUP"
else
    echo "[=] Group already exists: $GROUP"
fi

if id -nG "$TARGET_USER" | tr ' ' '\n' | grep -qx "$GROUP"; then
    echo "[=] $TARGET_USER is already a member of $GROUP"
else
    echo "[+] Adding $TARGET_USER to $GROUP"
    usermod -aG "$GROUP" "$TARGET_USER"
    RELOGIN_NEEDED=1
fi

echo "[+] Writing udev rule: $RULE_FILE"
cat > "$RULE_FILE" <<EOF
KERNEL=="event*", SUBSYSTEM=="input", GROUP="$GROUP", MODE="0660"
EOF

echo "[+] Reloading udev rules"
udevadm control --reload-rules
udevadm trigger --subsystem-match=input

echo
echo "Done."
if [[ "${RELOGIN_NEEDED:-0}" == "1" ]]; then
    echo "Note: log out and back in for group change to take effect."
fi
