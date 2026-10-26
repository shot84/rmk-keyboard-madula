//! IQS9151 tuning from User18, User19 and User23..User31 keys; values live in RAM and reset at power-off.
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering::Relaxed};

// Cursor speed is level/4 of the sensor delta; 4 (full speed) became the default after use
// (2 was the original half speed).
const CURSOR: (u8, u8, u8) = (1, 4, 8);
// Scroll steps once per this many centroid units; index 3 (32) is the original.
const SCROLL_DIVISORS: [i32; 7] = [96, 64, 48, 32, 24, 16, 12];
const SCROLL: (u8, u8, u8) = (0, 3, 6);
// One-finger tap time limit in 25 ms steps: 6..16 = 150..400 ms, 10 = 250 ms.
const TAP: (u8, u8, u8) = (6, 10, 16);

static CURSOR_LEVEL: AtomicU8 = AtomicU8::new(CURSOR.1);
static SCROLL_LEVEL: AtomicU8 = AtomicU8::new(SCROLL.1);
static TAP_LEVEL: AtomicU8 = AtomicU8::new(TAP.1);
static TAP_DRAG: AtomicBool = AtomicBool::new(true);
// Three fingers swipe by default (became the default after use); the toggle makes them drag.
static THREE_DRAG: AtomicBool = AtomicBool::new(false);
// One-finger motion is held back this many frames (about 11 ms each) and dropped at lift;
// index 1 (3 frames) is the default.
const LIFT_HOLDS: [usize; 3] = [0, 3, 5];
static LIFT_LEVEL: AtomicU8 = AtomicU8::new(1);
// Chip XY dynamic filter bottom beta: smaller smooths slow motion harder. 80 is Madula ZMK's,
// 20 Lalapad Gen2 RMK's, 10 the ZMK driver's init default; 40 (index 1) became the default
// after use.
const FILTER_BETAS: [u8; 4] = [80, 40, 20, 10];
const FILTER_DEFAULT: u8 = 1;
static FILTER_LEVEL: AtomicU8 = AtomicU8::new(FILTER_DEFAULT);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tune { pub cursor_num: i32, pub scroll_div: i32, pub tap_ms: u64, pub tap_drag: bool, pub lift_hold: usize, pub filter_beta: u8, pub three_drag: bool }
fn from_levels(cursor: u8, scroll: u8, tap: u8) -> Tune {
    Tune { cursor_num: i32::from(cursor), scroll_div: SCROLL_DIVISORS[usize::from(scroll)],
           tap_ms: u64::from(tap) * 25, tap_drag: TAP_DRAG.load(Relaxed),
           lift_hold: LIFT_HOLDS[usize::from(LIFT_LEVEL.load(Relaxed))],
           filter_beta: FILTER_BETAS[usize::from(FILTER_LEVEL.load(Relaxed))],
           three_drag: THREE_DRAG.load(Relaxed) }
}

pub fn current() -> Tune {
    from_levels(CURSOR_LEVEL.load(Relaxed), SCROLL_LEVEL.load(Relaxed), TAP_LEVEL.load(Relaxed))
}

fn step(level: &AtomicU8, (min, _, max): (u8, u8, u8), up: bool) -> u8 {
    let now = level.load(Relaxed);
    let next = if up { now.saturating_add(1).min(max) } else { now.saturating_sub(1).max(min) };
    level.store(next, Relaxed);
    next - min + 1
}

/// Handles one User key press. Returns how many LED pulses show the new level, if any.
pub fn adjust(user: u8) -> Option<u8> {
    let pulses = match user {
        23 => step(&CURSOR_LEVEL, CURSOR, false),
        24 => step(&CURSOR_LEVEL, CURSOR, true),
        25 => step(&SCROLL_LEVEL, SCROLL, false),
        26 => step(&SCROLL_LEVEL, SCROLL, true),
        27 => step(&TAP_LEVEL, TAP, false),
        28 => step(&TAP_LEVEL, TAP, true),
        29 => {
            CURSOR_LEVEL.store(CURSOR.1, Relaxed);
            SCROLL_LEVEL.store(SCROLL.1, Relaxed);
            TAP_LEVEL.store(TAP.1, Relaxed);
            TAP_DRAG.store(true, Relaxed);
            THREE_DRAG.store(false, Relaxed);
            LIFT_LEVEL.store(1, Relaxed);
            FILTER_LEVEL.store(FILTER_DEFAULT, Relaxed);
            1
        }
        // Tap-drag on/off: two pulses = on, one = off.
        30 => if TAP_DRAG.fetch_xor(true, Relaxed) { 1 } else { 2 },
        // Three fingers drag (two pulses) or swipe (one pulse).
        18 => if THREE_DRAG.fetch_xor(true, Relaxed) { 1 } else { 2 },
        // Chip filter cycles 80, 40, 20, 10: one to four pulses.
        19 => { let next = (FILTER_LEVEL.load(Relaxed) + 1) % 4; FILTER_LEVEL.store(next, Relaxed); next + 1 }
        // Lift hold cycles off, 3, 5 frames: one, two or three pulses.
        31 => { let next = (LIFT_LEVEL.load(Relaxed) + 1) % 3; LIFT_LEVEL.store(next, Relaxed); next + 1 }
        _ => return None,
    };
    log::info!("IQS9151 tune {:?}", current());
    Some(pulses)
}
