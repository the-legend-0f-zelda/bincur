## Configuration Example

### Virtual Mouse Settings

`~/.config/bincur/vmouse.conf:`
```conf
# cursor step size
step_size_x : 128
step_size_y : 128

# wheel scroll distance
scroll_dist_x : 50
scroll_dist_y : 50
```

### Keybinds

`~/.config/bincur/keymap.conf:`
```conf
# switch step mode
rightshift : LINEAR_MODE
leftshift : LOGARITHMIC_MODE

# move cursor
p : MOVE_UP
semicolon : MOVE_DOWN
apostrophe : MOVE_RIGHT
l : MOVE_LEFT

# click
dot : CLICK_LEFT
slash : CLICK_RIGHT

# scroll
rightmeta+p : SCROLL_UP
rightmeta+semicolon : SCROLL_DOWN
```

#### References
                                                                       
  - [Key names (evdev scancodes)](https://docs.rs/evdev/0.13.2/src/evdev/scancodes.rs.html#26-579) — valid key identifiers for the left side of `:`
  
---
