## Configuration Example

### Keybinds

`~/.config/bincur/keymap.conf:`
```conf
# layer
rightshift : LAYER_LINEAR
leftshift : LAYER_LOGARITHMIC

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

### References
                                                                       
  - [Key names (evdev scancodes)](https://docs.rs/evdev/0.13.2/src/evdev/scancodes.rs.html#26-579) — valid key identifiers for the left side of `:`
  
---
