//! The controller, read through XInput (Xbox pads and anything that acts like one, such as the
//! Ally's built-in controls). A thread polls it and turns presses into the same actions the
//! keyboard gives: "a", "b", "x", "y", "lb", "rb", "up", "down", "left", "right", "menu", "view".
//! Directions repeat while held, like a console menu.

use std::time::{Duration, Instant};

const POLL: Duration = Duration::from_millis(16);
const REPEAT_AFTER: Duration = Duration::from_millis(380);
const REPEAT_EVERY: Duration = Duration::from_millis(95);
/// Stick travel that counts as a direction, out of 32767.
const STICK: i16 = 16_000;

#[cfg(windows)]
fn read_pads() -> Vec<(u16, i16, i16)> {
    use windows_sys::Win32::UI::Input::XboxController::{XInputGetState, XINPUT_STATE};
    (0..4)
        .filter_map(|i| unsafe {
            let mut s: XINPUT_STATE = std::mem::zeroed();
            (XInputGetState(i, &mut s) == 0).then(|| (s.Gamepad.wButtons, s.Gamepad.sThumbLX, s.Gamepad.sThumbLY))
        })
        .collect()
}

#[cfg(not(windows))]
fn read_pads() -> Vec<(u16, i16, i16)> {
    Vec::new()
}

const BUTTONS: &[(u16, &str)] = &[
    (0x1000, "a"),
    (0x2000, "b"),
    (0x4000, "x"),
    (0x8000, "y"),
    (0x0100, "lb"),
    (0x0200, "rb"),
    (0x0010, "menu"),
    (0x0020, "view"),
];
const DPAD: &[(u16, &str)] = &[(0x0001, "up"), (0x0002, "down"), (0x0004, "left"), (0x0008, "right")];

/// Starts polling. `send` gets each action; `active` says whether Mantel should listen at all
/// (not while a game is in front).
pub fn start(send: impl Fn(&'static str) + Send + 'static, active: impl Fn() -> bool + Send + 'static) {
    std::thread::spawn(move || {
        let mut held: u16 = 0;
        let mut dir: Option<(&'static str, Instant, Instant)> = None;
        loop {
            std::thread::sleep(POLL);
            let pads = read_pads();
            if pads.is_empty() {
                std::thread::sleep(Duration::from_millis(500));
                held = 0;
                dir = None;
                continue;
            }
            let mut buttons = 0u16;
            let (mut lx, mut ly) = (0i16, 0i16);
            for (b, x, y) in &pads {
                buttons |= b;
                if x.unsigned_abs() > lx.unsigned_abs() {
                    lx = *x;
                }
                if y.unsigned_abs() > ly.unsigned_abs() {
                    ly = *y;
                }
            }
            if !active() {
                held = buttons;
                dir = None;
                continue;
            }
            for (mask, name) in BUTTONS {
                if buttons & mask != 0 && held & mask == 0 {
                    send(name);
                }
            }
            held = buttons;
            // One direction at a time: the d-pad first, then the stick's stronger axis.
            let now_dir = DPAD.iter().find(|(m, _)| buttons & m != 0).map(|(_, n)| *n).or_else(|| {
                if lx.unsigned_abs() >= ly.unsigned_abs() {
                    (lx > STICK).then_some("right").or((lx < -STICK).then_some("left"))
                } else {
                    (ly > STICK).then_some("up").or((ly < -STICK).then_some("down"))
                }
            });
            let now = Instant::now();
            dir = match (now_dir, dir) {
                (None, _) => None,
                (Some(d), Some((was, since, last))) if was == d => {
                    if now - since >= REPEAT_AFTER && now - last >= REPEAT_EVERY {
                        send(d);
                        Some((d, since, now))
                    } else {
                        Some((d, since, last))
                    }
                }
                (Some(d), _) => {
                    send(d);
                    Some((d, now, now))
                }
            };
        }
    });
}
