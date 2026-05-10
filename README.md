# Binary Cursor
> Linux only (uses `evdev` / `uinput`)


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

## Build & Install

### Dependencies
  - Rust 1.85+ (edition 2024)
  - `pkg-config`
  - `libudev` development headers
    - Debian/Ubuntu: `sudo apt install libudev-dev pkg-config`
    - Fedora: `sudo dnf install systemd-devel pkgconf`
    - Arch: already included with `systemd` (default)

### Build
```bash
git clone https://github.com/the-legend-0f-zelda/bincur
cd bincur
cargo build --release
```
Binary will be at `target/release/bincur`.

## Configuration & Examples (purely my preference)

설정 파일들은 `BINCUR_CONF_HOME` 환경변수로 설정된 위치(우선순위) 또는 `~/.config/bincur`(디폴트) 아래에 두면 적용됩니다.

- `vmouse.conf` : 에뮬레이션되는 가상 마우스의 프로퍼티
- `keymap.conf` : 키바인드 설정

아래 설정 예시들은 손이 타이핑을 위해 준비된 상태(각 손가락이 `a`, `s`, `d`, `f`, `j`, `k`, `l`, `;` 위에 있음)에서 움직임을 최소화 하는 방향으로 설정했습니다.

### Virtual Mouse Settings

`~/.config/bincur/vmouse.conf:`
```conf
# Grab mode trigger keys
# TRUE로 설정하면 해당 모드에 바인드된 키 콤보 입력 이벤트가 소모되고 백그라운드로 전파되지 않습니다.
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

#### 팁

- 예시에서 `LOGARITHMIC_MODE`와 `SCROLL_MODE`를 `leftalt`와의 조합으로 설정한 이유는 각각 `leftshift`, `c`만 눌렀을 때 `vmouse.conf`에 의해 grab되는 것을 방지하기 위함입니다.
- 예시에 키를 소문자, 벨류를 대문자로 써두었지만 내부적으로 대소문자 구분은 전혀 하지 않기 때문에 어느 쪽을 어떻게 쓰든 상관이 없습니다. 카멜 케이스처럼 섞어 쓰는 방식도 가능합니다.
- 키바인드에 사용하고 싶은 키의 명칭을 확인하려면 [References](#references)의 **Key names**를 참고하세요. 각 키코드 문자에서 **`KEY_`** 부분을 제외한 나머지가 바인드에 사용 가능한 키 이름이 됩니다. (예: `KEY_LEFTCTRL` → `LEFTCTRL`)

### References

- [Key names (evdev scancodes)](https://docs.rs/evdev/0.13.2/src/evdev/scancodes.rs.html#26-579) — valid key identifiers for the left side of `:`

---
