# Binary Cursor
> Linux only (uses `evdev` / `uinput`)

> [!IMPORTANT]
> For uniform cursor movement, disable mouse acceleration in your system settings.

## Table of Contents

- [Intro](#intro)
  - [What's this](#whats-this)
  - [Features](#features)
- [Tested on](#tested-on)
- [Getting Started](#getting-started)
  - [Build](#build)
  - [Setup](#setup)
- [Configuration & Examples](#configuration--examples-purely-my-preference)
  - [Virtual Mouse Settings](#virtual-mouse-settings)
  - [Keybinds](#keybinds)
  - [References](#references)

## Intro

### What's this

Built to fully control the mouse using only the keyboard.

Inspired by the grid mode of existing solutions like warpd and mouseless, this tool blends that idea with a natural mode in a simplified form, aiming to give you the feeling of **owning the cursor in real time**.

### Features

#### Three trigger modes

| Mode | Description |
| --- | --- |
| **Linear** | The cursor moves by a fixed step at a constant rate. |
| **Logarithmic** | Each input moves the cursor by half the previous distance (`128 → 64 → 32 ...`). Used for fine-tuning the cursor via **binary search** after a rough move with Linear mode. |
| **Scroll** | Combined with the configured directional keys, this mode performs up/down/left/right wheel scrolling. |

> Left and right mouse clicks work in any mode.

#### Other characteristics

- **No overlay** — Nothing extra is drawn on the screen. You press keys, and the cursor moves. That's it.
- **Hold-to-trigger** — To switch quickly between regular typing and momentary mouse control without finger gymnastics, every mode is triggered by **holding** the key, not by toggling.
- **Per-key distinction** — Built on the [evdev](https://github.com/emberian/evdev) crate, using its raw scancodes(Follows the Linux kernel scancodes) for key identification. Every key is uniquely distinguishable — for example, **left and right Control** can be bound separately.
- **Hot-plug support** — Detects keyboard connect/disconnect events at runtime and reconfigures automatically. No need to restart after plugging in a new keyboard via USB or Bluetooth.
- **Selective grab** — You can decide whether each mode-trigger key is grabbed (see [Virtual Mouse Settings](#virtual-mouse-settings)). For instance, if Shift is set as a mode trigger, leaving it ungrabbed lets it serve double duty as both a trigger and the normal case-shift key.

## Tested on
  - **OS**: EndeavourOS (Arch-based), x86_64
  - **Kernel**: 6.19.11-arch1-1
  - **Compositor**: Hyprland 0.54.3 (Wayland)


## Getting Started

### Build

#### Dependencies
  - Rust 1.85+ (edition 2024)
  - `pkg-config`
  - `libudev` development headers
    - Debian/Ubuntu: `sudo apt install libudev-dev pkg-config`
    - Fedora: `sudo dnf install systemd-devel pkgconf`
    - Arch: already included with `systemd` (default)

```bash
git clone https://github.com/the-legend-0f-zelda/bincur
cd bincur
cargo build --release
```
Binary will be at `target/release/bincur`.

### Setup

A helper script handles permission setup, binary installation, default config file creation, and systemd user service registration in one shot.

```bash
./scripts/setup.sh
```
> **Don't run as root.** The script invokes `sudo` internally only for steps that actually need root. (It caches credentials with `sudo -v` at start, so you'll only be prompted for your password once.)

What the script does:

1. **Add user to the `input` group** — for `/dev/input/event*` access (requires re-login)
2. **Write the udev rule** (`/etc/udev/rules.d/99-bincur-uinput.rules`) — exposes `/dev/uinput` to the `input` group with mode 0660
3. **Install the binary** — `target/release/bincur` → `~/.local/bin/bincur` (run `cargo build --release` first)
4. **Write default config files** — `~/.config/bincur/vmouse.conf`, `~/.config/bincur/keymap.conf` (skipped if they already exist)
5. **Register the systemd user service** — `~/.config/systemd/user/bincur.service`

After setup, start it with:

```bash
systemctl --user start bincur
```

To revert:

```bash
./scripts/setup.sh --uninstall
```

This stops/removes the service, deletes the binary, and removes the udev rule. User config files (`~/.config/bincur/*.conf`) and `input` group membership are preserved (other tools may depend on them).

For usage info: `./scripts/setup.sh --help`

## Configuration & Examples (purely my preference)

Place the config files under the directory pointed to by the `BINCUR_CONF_HOME` environment variable (takes precedence) or `~/.config/bincur` (default).

- `vmouse.conf` — properties of the emulated virtual mouse
- `keymap.conf` — keybind configuration

The examples below are tuned to minimize finger movement from the typing home position (fingers resting on `a`, `s`, `d`, `f`, `j`, `k`, `l`, `;`).

### Virtual Mouse Settings

`~/.config/bincur/vmouse.conf:`
```conf
# Grab mode trigger keys
# When set to TRUE, key-combo input events bound to the corresponding mode are consumed and not propagated to other applications.
grab_linear : FALSE
grab_logarithmic : TRUE
grab_scroll : TRUE

# Cursor step size
step_size_x : 256
step_size_y : 256

# Wheel scroll distance
scroll_dist_x : 2
scroll_dist_y : 2
```

### Keybinds

`~/.config/bincur/keymap.conf:`
```conf
# 0. Terminate process
leftctrl+q : EXIT

# 1. Trigger virtual mouse mode

# 1-1. For standard keyboard layout
leftalt : LINEAR_MODE
leftalt+leftshift : LOGARITHMIC_MODE
leftalt+c : SCROLL_MODE

# 1-2. For HHKB layout
#leftmeta : LINEAR_MODE
#leftmeta+leftshift : LOGARITHMIC_MODE
#leftmeta+c : SCROLL_MODE

# 2. Move cursor or scroll
i : MOVE_UP
k : MOVE_DOWN
j : MOVE_LEFT
l : MOVE_RIGHT

# 3. Mouse click
space : CLICK_LEFT
semicolon : CLICK_RIGHT
```

#### Tips

- In the example, `LOGARITHMIC_MODE` and `SCROLL_MODE` are bound to combinations with `leftalt` to prevent `leftshift` or `c` alone from being grabbed by `vmouse.conf`.
- Although keys are written in lowercase and values in uppercase in the example, the parser is fully case-insensitive — write either side however you want, and you can even mix cases (e.g., camelCase) within a single token.
- To find the name of a key you want to bind, see **Key names** under [References](#references). Strip the **`KEY_`** prefix from each scancode identifier and the remainder is the bindable key name (e.g., `KEY_LEFTCTRL` → `LEFTCTRL`).

### References

- [Key names (evdev scancodes)](https://docs.rs/evdev/0.13.2/src/evdev/scancodes.rs.html#26-579) — valid key identifiers for the left side of `:`

---
