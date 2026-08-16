use arrayvec::ArrayVec;
use evdev::{KeyCode, RelativeAxisCode};
use evdev::{uinput::VirtualDevice, EventType, InputEvent};
use std::cell::RefCell;
use std::sync::atomic::{AtomicU32, Ordering::Relaxed};
use Direction::*;

use crate::config::{self,keymap};
use crate::config::vmouse::{Mode, Props};
use crate::device::DeviceError;


static ACTIVATED_SET_BITMASK: AtomicU32 = AtomicU32::new(0);

thread_local! {
    static VMOUSE_DEVICE: RefCell<VirtualDevice> = RefCell::new(
        VirtualDevice::builder().unwrap()
            .name("bincur")
            .with_relative_axes(config::vmouse::get_rel_axes()).unwrap()
            .with_keys(config::vmouse::get_keys()).unwrap()
            .build().unwrap()
    );
    static VMOUSE_PROPS: RefCell<Props> = RefCell::new(Props::new());
}

pub fn mark_active(behavior: Behavior) -> bool {
    let bit = 1u32 << (behavior as u32);
    ACTIVATED_SET_BITMASK.fetch_or(bit, Relaxed) & bit == 0
}

pub fn mark_inactive(behavior: Behavior) -> bool {
    let bit = 1u32 << (behavior as u32);
    ACTIVATED_SET_BITMASK.fetch_and(!bit, Relaxed) & bit != 0
}

pub fn is_active(behavior: Behavior) -> bool {
    let bit = 1u32 << (behavior as u32);
    ACTIVATED_SET_BITMASK.load(Relaxed) & bit != 0
}

pub fn clear_activated_set() {
    ACTIVATED_SET_BITMASK.store(0, Relaxed);
}

pub fn initialize() {
    VMOUSE_PROPS.with_borrow_mut(|props| {
        *props = config::vmouse::copy_default_props()
    });
}

pub fn longest_actives(kbd_idx: usize) -> ArrayVec<Behavior, {Behavior::VAR_COUNT}>
{
    let mut longest: ArrayVec<Behavior, {Behavior::VAR_COUNT}> = ArrayVec::new();
    let mut max_combo_len = 0;

    for behavior in Behavior::ALL {
        if !is_active(behavior) {continue;}

        match behavior {
            Behavior::LinearModeOn
            | Behavior::LogarithmicModeOn
            | Behavior::ScrollModeOn
            | Behavior::Exit => {
                continue;
            },
            _ => {
                let len = match keymap::get_combo(kbd_idx, behavior) {
                    Some(combo) => combo.len(),
                    None => 0
                };
                if len < max_combo_len {continue}
                if len > max_combo_len {
                    longest.clear();
                    max_combo_len = len;
                }
                longest.push(behavior);
            }
        }
    }

    longest
}

pub fn get_mode() -> Mode {
    VMOUSE_PROPS.with_borrow(|mouse| mouse.mode)
}


#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum Behavior {
    Reset,

    LinearModeOn, LinearModeOff,
    LogarithmicModeOn, LogarithmicModeOff,
    ScrollModeOn, ScrollModeOff,

    MoveUp, MoveDown,
    MoveLeft, MoveRight,

    ClickLeft, ClickRight,
    ReleaseLeft, ReleaseRight,

    KeyUp,

    /// Must always remain the last variant; `VAR_COUNT` depends on it.
    /// Add new variants above this line.
    Exit
}

/// `Behavior::ALL` must be ordered by variant declaration order,
/// otherwise `Behavior::from_usize` silently returns the wrong variant.
const _: () = {
    let mut i = 0;
    while i < Behavior::VAR_COUNT {
        assert!(Behavior::ALL[i] as usize == i, "Behavior::ALL order must match variant order");
        i += 1;
    }
};

impl Behavior {
    pub const VAR_COUNT: usize = Self::Exit as usize + 1;

    const ALL: [Self; Self::VAR_COUNT] = [
            Self::Reset,

            Self::LinearModeOn, Self::LinearModeOff,
            Self::LogarithmicModeOn, Self::LogarithmicModeOff,
            Self::ScrollModeOn, Self::ScrollModeOff,

            Self::MoveUp, Self::MoveDown,
            Self::MoveLeft, Self::MoveRight,

            Self::ClickLeft, Self::ClickRight,
            Self::ReleaseLeft, Self::ReleaseRight,

            Self::KeyUp,

            Self::Exit
    ];

    pub fn from_usize(n: usize) -> Option<Self> {
        Self::ALL.get(n).copied()
    }

    pub fn from_str(behavior: &str) -> Self {
        match behavior.to_uppercase().as_str() {
            "EXIT" => Self::Exit,
            "RESET" => Self::Reset,

            "LINEAR_MODE" => Self::LinearModeOn,
            "LOGARITHMIC_MODE" => Self::LogarithmicModeOn,
            "SCROLL_MODE" => Self::ScrollModeOn,

            "MOVE_UP" => Self::MoveUp,
            "MOVE_DOWN" => Self::MoveDown,
            "MOVE_LEFT" => Self::MoveLeft,
            "MOVE_RIGHT" => Self::MoveRight,

            "CLICK_LEFT" => Self::ClickLeft,
            "CLICK_RIGHT" => Self::ClickRight,
            "RELEASE_LEFT" => Self::ReleaseLeft,
            "RELEASE_RIGHT" => Self::ReleaseRight,

            _ => {
                eprintln!("ERROR - unknown vmouse behvior: {}", behavior);
                std::process::exit(1);
            }
        }
    }

    pub fn inverse(&self) -> Option<Self> {
        match self {
            Self::ClickLeft => Some(Self::ReleaseLeft),
            Self::ClickRight => Some(Self::ReleaseRight),

            Self::LinearModeOn => Some(Self::LinearModeOff),
            Self::LogarithmicModeOn => Some(Self::LogarithmicModeOff),
            Self::ScrollModeOn => Some(Self::ScrollModeOff),

            _ => Some(Self::KeyUp)
        }
    }

    pub fn dispatch(&self) -> Result<bool, DeviceError> {

        let events:ArrayVec<InputEvent, 2> = match self {
            Self::Exit => {
                println!("Exit bincur.");
                std::process::exit(0);
            },
            Self::Reset => {
                return Err(DeviceError::ResetRequested)
            },

            Self::LinearModeOn => {
                return VMOUSE_PROPS.with_borrow_mut(|mouse| {
                    if mouse.mode == Mode::Inactive {
                        mouse.mode = Mode::Linear;
                    }
                    Ok(mouse.grab_linear)
                });
            },
            Self::LinearModeOff => {
                return VMOUSE_PROPS.with_borrow_mut(|mouse| {
                    if mouse.mode == Mode::Linear {
                        mouse.mode = Mode::Inactive;
                    }
                    Ok(mouse.grab_linear)
                });
            },

            Self::LogarithmicModeOn => {
                return VMOUSE_PROPS.with_borrow_mut(|mouse| {
                    if mouse.mode == Mode::Inactive
                    || mouse.mode == Mode::Linear {
                        mouse.mode = Mode::Logarithmic;
                    }
                    Ok(mouse.grab_logarithmic)
                });
            },
            Self::LogarithmicModeOff => {
                return VMOUSE_PROPS.with_borrow_mut(|mouse| {
                    if mouse.mode == Mode::Logarithmic {
                        if is_active(Behavior::LinearModeOn) {
                            mouse.mode = Mode::Linear;
                        }else {
                            mouse.mode = Mode::Inactive;
                        }
                    }
                    mouse.reset_xy();
                    Ok(mouse.grab_logarithmic)
                });
            },

            Self::ScrollModeOn => {
                return VMOUSE_PROPS.with_borrow_mut(|mouse| {
                    mouse.mode = Mode::Scroll;
                    Ok(mouse.grab_scroll)
                });
            },
            Self::ScrollModeOff => {
                return VMOUSE_PROPS.with_borrow_mut(|mouse| {
                    if mouse.mode == Mode::Scroll {
                        if is_active(Behavior::LogarithmicModeOn) {
                            mouse.mode = Mode::Logarithmic;
                        }else if is_active(Behavior::LinearModeOn) {
                            mouse.mode = Mode::Linear;
                            mouse.reset_xy();
                        }else {
                            mouse.mode = Mode::Inactive;
                        }
                    }
                    Ok(mouse.grab_scroll)
                });
            },

            Self::MoveUp => new_move_event(Up),
            Self::MoveDown => new_move_event(Down),
            Self::MoveLeft => new_move_event(Left),
            Self::MoveRight => new_move_event(Right),

            Self::ClickLeft => new_click_event(Left, 1),
            Self::ClickRight => new_click_event(Right, 1),
            Self::ReleaseLeft => new_click_event(Left, 0),
            Self::ReleaseRight => new_click_event(Right, 0),

            Self::KeyUp => return Ok(true)

        };

        VMOUSE_DEVICE.with_borrow_mut(|device| {
            if let Err(e) = device.emit(&events) {
                eprintln!("ERROR - emit failed: {}", e);
            }
        });

        Ok(true)
    }
}

enum Direction {Up, Down, Left, Right,}
impl Direction {
    fn is_vertical(&self) -> bool {
        matches!(self, Up | Down)
    }

    fn move_sign(&self) -> i32 {
        match self {Up | Left => -1, Down | Right => 1}
    }

    fn scroll_sign(&self) -> i32 {
        match self {Down | Left => -1, Up | Right => 1}
    }
}

fn new_rel_event(axis: RelativeAxisCode, value: i32) -> InputEvent {
    InputEvent::new_now(EventType::RELATIVE.0, axis.0, value)
}

fn move_target(mouse: &mut Props, vertical: bool) -> (RelativeAxisCode, &mut i32, i32) {
    if vertical {
        (RelativeAxisCode::REL_Y, &mut mouse.step_size_y, mouse.min_step_size_y)
    }else {
        (RelativeAxisCode::REL_X, &mut mouse.step_size_x, mouse.min_step_size_x)
    }
}

fn new_move_event(direction: Direction) -> ArrayVec<InputEvent, 2>
{
    VMOUSE_PROPS.with_borrow_mut(|mouse| {
        let mut events:ArrayVec<InputEvent, 2> = ArrayVec::new();

        match mouse.mode {
            // Linear mode
            Mode::Linear => {
                let (axis, step, _min_step) = move_target(mouse, direction.is_vertical());
                events.push(new_rel_event(axis, direction.move_sign() * *step));
            },

            // Logarithmic mode
            Mode::Logarithmic => {
                let (axis, step, min_step) = move_target(mouse, direction.is_vertical());
                *step = (*step+1) >> 1;
                if *step < min_step {*step = min_step;}
                events.push(new_rel_event(axis, direction.move_sign() * *step));
            },

            // Scroll mode
            // Emits two synthetic pointer events:
            // 1. A high-resolution scroll event
            // 2. A discrete (notch-based) scroll event,
            //    derived by dividing the accumulated high-resolution scroll value by 120
            Mode::Scroll => {
                let (dist, accum, hi_res_axis, notch_axis, min_dist) =
                    if direction.is_vertical() {
                        (&mut mouse.scroll_dist_y, &mut mouse.scroll_accum_y,
                            RelativeAxisCode::REL_WHEEL_HI_RES,
                            RelativeAxisCode::REL_WHEEL, mouse.min_scroll_dist_y)
                    }else {
                        (&mut mouse.scroll_dist_x, &mut mouse.scroll_accum_x,
                            RelativeAxisCode::REL_HWHEEL_HI_RES,
                            RelativeAxisCode::REL_HWHEEL, mouse.min_scroll_dist_x)
                    };

                if is_active(Behavior::LogarithmicModeOn) {
                    *dist = (*dist+1) >> 1;
                }

                if *dist < min_dist {
                    *dist = min_dist;
                }

                let delta = direction.scroll_sign() * *dist;
                *accum += delta;

                events.push(new_rel_event(hi_res_axis, delta));
                events.push(new_rel_event(notch_axis, *accum / 120));

                *accum %= 120;
            },

            Mode::Inactive => {}
        }

        events
    })
}

fn new_click_event(direction: Direction, value: i32) -> ArrayVec<InputEvent, 2> {
    let mut events:ArrayVec<InputEvent, 2> = ArrayVec::new();

    if VMOUSE_PROPS.with_borrow(|mouse| mouse.mode) == Mode::Inactive {return events}
    match direction {
        Left => {events.push(
            InputEvent::new_now(EventType::KEY.0, KeyCode::BTN_LEFT.0, value)
        )},
        Right => {events.push(
            InputEvent::new_now(EventType::KEY.0, KeyCode::BTN_RIGHT.0, value)
        )},
        _ => {}
    }

    events
}
