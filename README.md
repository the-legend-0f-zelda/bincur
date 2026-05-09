# Binary Cursor
---

## Intro

### What's this

키보드만으로 마우스를 완전히 조작하기 위해 만들어졌습니다.

warpd나 mouseless 같은 기존 솔루션들의 그리드 모드에서 영감을 받았으며, 이를 네추럴 모드와 절묘하게 융합시키는 동시에 단순화해 **실시간으로 마우스 커서를 완전히 장악하고 있는 듯한 느낌**을 내고자 했습니다.

### Features

#### 세 가지 트리거 모드

| 모드 | 설명 |
| --- | --- |
| **Linear** | 마우스 커서가 세팅된 고정 길이만큼 일정하게 움직입니다. |
| **Logarithmic** | 매 입력마다 마우스 커서가 이전 이동거리의 절반씩 움직입니다 (`128px → 64px → 32px ...`). Linear 모드로 대략적인 이동을 마친 후 **이진탐색 방식**으로 마우스를 정밀 조작하는 용도로 쓰입니다. |
| **Scroll** | 이 모드와 설정된 방향 조작키의 조합으로 상/하/좌/우 휠스크롤이 가능합니다. |

> 마우스 좌클릭과 우클릭은 어떤 모드에서나 작동합니다.

#### 그 외 특징

- **No overlay** — 화면상에 그 어떤 추가적인 오버레이도 띄우지 않습니다. 그저 당신이 키보드를 조작하면, 그에 따라 커서가 움직일 뿐입니다.
- **Hold-to-trigger** — 실제 키보드 입력과 일시적인 마우스 조작을 신속히 전환하고 손 꼬임을 줄이기 위해, 모든 모드는 토글이 아닌 **홀드 방식**으로 트리거됩니다.
- **고유 키 구분** — `evdev` 크레이트 기반으로 만들어졌으며, 키 구분을 위해 evdev의 scancode를 그대로 사용합니다. 따라서 모든 키를 고유하게 구분할 수 있습니다. 예를 들어 같은 컨트롤 키도 **좌컨트롤/우컨트롤**을 구분해서 바인드할 수 있습니다.
- **Hot-plug 지원** — 프로그램 실행 중 새로운 키보드 디바이스의 연결/해제를 감지하고 자동으로 재설정합니다. USB 케이블이나 블루투스로 키보드를 새로 연결한 후 프로그램을 재실행할 필요가 없습니다.
- **선택적 grab** — 바인드된 모드 트리거 키를 grab할지 말지 결정할 수 있습니다 ([Virtual Mouse Settings](#virtual-mouse-settings) 참조). 예를 들어 쉬프트 키를 모드 트리거로 설정한 경우 grab하지 않도록 하면, 모드 트리거와 대소문자 레이어 전환 용도로 함께 사용할 수 있습니다.


## Configuration & Examples (purely my preference)

### Virtual Mouse Settings

`~/.config/bincur/vmouse.conf:`
```conf
# Grab mode trigger keys
grab_linear : false
grab_logarithmic : true
grab_scroll : true

# Cursor step size
step_size_x : 128
step_size_y : 128

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

#### References
                                                                       
  - [Key names (evdev scancodes)](https://docs.rs/evdev/0.13.2/src/evdev/scancodes.rs.html#26-579) — valid key identifiers for the left side of `:`
  
---
