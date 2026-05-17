# Binary Cursor
> Linux only (uses `evdev` / `uinput`)

## Table of Contents

- [Intro](#intro)
  - [What's this](#whats-this)
  - [Demo (Copy & Paste)](#demo-copy--paste)
  - [Features](#features)
- [Tested on](#tested-on)
- [Getting Started](#getting-started)
  - [Build](#build)
  - [Setup](#setup)
- [Configuration & Examples](#configuration--examples)
  - [Virtual Mouse Settings](#virtual-mouse-settings)
  - [Keybinds](#keybinds)
- [References](#references)

<br>

## Intro

### What's this

Built to fully control the mouse using only the keyboard.

Inspired by the grid mode of existing solutions like warpd and mouseless, this tool blends that idea with a natural mode in a simplified form, aiming to give you the feeling of **owning the cursor in real time**.

<br>

### Demo (Copy & Paste)
**Full mouse control — movement, scrolling, clicks, and drag — keyboard only.**

https://github.com/user-attachments/assets/de68d664-9fa7-41c9-8a45-5c69f5ace63e

<br>

### Features

#### Three trigger modes

| Mode | Description |
| --- | --- |
| **Linear** | The cursor moves by a fixed step at a constant rate. |
| **Logarithmic** | Each input moves the cursor by half the previous distance (`128 → 64 → 32 ...`). Used for fine-tuning the cursor via **binary search** after a rough move with Linear mode. |
| **Scroll** | Combined with the configured directional keys, this mode performs up/down/left/right wheel scrolling. |

> [!IMPORTANT]
> - For uniform cursor movement, disable **mouse acceleration** in your system settings.
> - Left and right mouse clicks work in any mode.
> - When keybinds overlap, trigger priority is **scroll(highest)** → **logarithmic** → **linear**.

#### Other characteristics

- **Composable primitives** — Bind only a few basic mouse actions to keys, and their combinations work as-is. For instance, moving the cursor while holding click produces a drag, and pressing two adjacent direction keys together produces diagonal movement.
- **No overlay** — Nothing extra is drawn on the screen. You press keys, and the cursor moves. That's it.
- **Hold-to-trigger** — To switch quickly between regular typing and momentary mouse control without finger gymnastics, every mode is triggered by **holding** the key, not by toggling.
- **Per-key distinction** — Built on the [evdev](https://github.com/emberian/evdev) crate, using its raw scancodes(Follows the Linux kernel scancodes) for key identification. Every key is uniquely distinguishable — for example, **left and right Control** can be bound separately.
- **Per-deivce configuration** - 키매핑과 리와이어 설정의 경우, 디바이스의 이름을 사용해 해당 키보드 디바이스에만 적용되는 분리된 설정을 만들 수 있습니다.
- **Hot-plug support** — Detects keyboard connect/disconnect events at runtime and reconfigures automatically. No need to restart after plugging in a new keyboard via USB or Bluetooth.
- **Selective grab** — You can decide whether each mode-trigger key is grabbed (see [Virtual Mouse Settings](#virtual-mouse-settings)). For instance, if Shift is set as a mode trigger, leaving it ungrabbed lets it serve double duty as both a trigger and the normal case-shift key.
- **키 리매핑** - 간단한 설정파일(rewire.conf)로 실제 물리키 입력을 다른 키 입력으로 변환시킬 수 있습니다.

<br>

## Tested on
  - **OS**: EndeavourOS (Arch-based), x86_64
  - **Kernel**: 6.19.11-arch1-1
  - **Compositor**: Hyprland 0.54.3 (Wayland)


<br>

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

<br>

### Setup

A helper script handles permission setup, binary installation, default config file creation, and systemd user service registration in one shot.

```bash
./scripts/setup.sh
```
> [!IMPORTANT]
> **Don't run as root.** The script invokes `sudo` internally only for steps that actually need root. (It caches credentials with `sudo -v` at start, so you'll only be prompted for your password once.)

What the script does:

1. **Add user to the `input` group** — for `/dev/input/event*` access (requires re-login)
2. **Write the udev rule** (`/etc/udev/rules.d/99-bincur-uinput.rules`) — exposes `/dev/uinput` to the `input` group with mode 0660
3. **Install the binary** — `target/release/bincur` → `~/.local/bin/bincur` (run `cargo build --release` first)
4. **Write default config files** — `~/.config/bincur/vmouse.conf`, `~/.config/bincur/keymap.conf` (skipped if they already exist)
5. **Register the systemd user service** — `~/.config/systemd/user/bincur.service`
---
After setup, **start** it with:

```bash
systemctl --user start bincur
```

To **revert**:

```bash
./scripts/setup.sh --uninstall
```

This stops/removes the service, deletes the binary, and removes the udev rule. User config files (`~/.config/bincur/*.conf`) and `input` group membership are preserved (other tools may depend on them).

For usage info: `./scripts/setup.sh --help`

<br>

## Configuration & Examples

Place the config files under the directory pointed to by the `BINCUR_CONF_HOME` environment variable (takes precedence) or `~/.config/bincur` (default).

- `vmouse.conf` — properties of the emulated virtual mouse
- `keymap.conf` — keybind configuration
- `rewire.conf` — 물리키 입력을 다른 키 입력 이벤트로 바꿔주는 설정입니다.

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

# 2. Move cursor or scroll
i : MOVE_UP
k : MOVE_DOWN
j : MOVE_LEFT
l : MOVE_RIGHT

# 3. Mouse click
space : CLICK_LEFT
semicolon : CLICK_RIGHT
```

### Rewire

아래 설정은 키보드의 좌알트 키와 좌메타 키의 입력을 서로 바꾸는 예시 입니다.

`~/.config/bincur/rewire.conf`
```conf
leftalt -> leftmeta
leftmeta -> leftalt
```

---
#### Tips

- In the example, `LOGARITHMIC_MODE` and `SCROLL_MODE` are bound to combinations with `leftalt` to prevent `leftshift` or `c` alone from being grabbed by `vmouse.conf`.
- Although keys are written in lowercase and values in uppercase in the example, the parser is fully case-insensitive — write either side however you want, and you can even mix cases (e.g., camelCase) within a single token.
- To find the name of a key you want to bind, see **Key names** under [References](#references). Strip the **`KEY_`** prefix from each scancode identifier and the remainder is the bindable key name (e.g., `KEY_LEFTCTRL` → `LEFTCTRL`).
- 메타나 알트등 모디파이어를 특정 모드 트리거로 사용할때 백그라운드 앱들이 영향을 받는게 거슬린다면, 크게 세가지 해결책이 있습니다. 
  - 첫째는 vmouse.conf 에서 해당 모드의 grab 설정을 true로 바꾸는것인데, 컴포지터를 포함한 다른 모든 앱들에서 해당 키를 사용할 수 없게되는 부작용이 있습니다. 
  - 두번째는 rewire.conf 설정을 사용하는 것입니다. 예를들어 왼쪽 메타키를 bincur의 모드 트리거키로 사용하려는데 Super_L 심볼에 반응하는 브라우저가 거슬린다면, rewire.conf에 ```leftmeta -> f18``` 이런식으로 설정해볼 수 있습니다. 대부분의 앱들이 f18에 반응하지 않으므로 bincur만 트리거할 수 있게됩니다. 컴포지터나 몇몇 앱에서 선택적으로 왼쪽 메타를 같이 사용하고 싶으면, 해당 개별 앱들의 키바인드 설정에서 좌측 메타키를 f18로 대체하면 됩니다. 하지만 앱이 특정 모디파이어 심볼을 제외한 모든 키 입력에 반응하는 상황이라면, 첫번째나 세번째 방법을 택해야 합니다.
  - 세번째는 xkb 설정을 통해 해당 키의 키심을 변경하는 것입니다. 예를들어 하이퍼랜드를 사용할 경우, 모드 트리거 키의 키심을 Hyper_L 로 설정하면 Super_L 키심에 반응하는 백그라운드 앱들은 반응하지 않게 하면서 메인모드키와 bincur와 하이퍼랜드의 트리거 키로 사용 가능합니다. 

### Per-device configuration
레이아웃이 다른 키보드들을 번갈아가며 사용할때 각각 편하게 느껴지는 키바인드가 다를 수 있습니다. 게이밍용이나 작업용 키보드, 노트북의 빌트인 키보드들을 오갈때 매번 번거롭게 설정 파일들을 일일이 수정할 필요가 없도록 디바이스 개별 키바인드 및 리와이어 설정 기능을 제공합니다.

기본적으로 keymap.conf 와 rewire.conf는 모든 키보드에 디폴트로 적용되는 설정파일 입니다. 특정 키보드에만 다른 설정파일을 적용하고싶다면, 각각 keymap.<디바이스명>.conf 와 rewire.<디바이스명>.conf 형태로 파일명을 짓고 설정 스크립트를 작성하면 됩니다. 사용중인 키보드 디바이스의 이름은 inspect 모드를 통해 간단히 확인할 수 있습니다.

다음 명령어로 inspect 모드 실행합니다.
```bash
bincur -i
```

실행 후 디바이스명을 알고싶은 키보드의 아무 키나 눌러보세요. 터미널에 이벤트가 발생한 키보드의 디바이스 이름, 내부 인덱스, 키 이벤트 정보가 나타납니다. 

---


## References

- [Key names (evdev scancodes)](https://docs.rs/evdev/0.13.2/src/evdev/scancodes.rs.html#26-579) — valid key identifiers for the left side of `:`
