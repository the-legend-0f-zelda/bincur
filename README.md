## Configuration Example (purely my preference)

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

# 2-1. Intuitive
i : MOVE_UP
k : MOVE_DOWN
j : MOVE_LEFT
l : MOVE_RIGHT

# 2-2. Vim
# k : MOVE_UP
# j : MOVE_DOWN
# h : MOVE_LEFT
# l : MOVE_RIGHT

# 3. Mouse click
space : CLICK_LEFT
semicolon : CLICK_RIGHT
```

#### References
                                                                       
  - [Key names (evdev scancodes)](https://docs.rs/evdev/0.13.2/src/evdev/scancodes.rs.html#26-579) — valid key identifiers for the left side of `:`
  
---
