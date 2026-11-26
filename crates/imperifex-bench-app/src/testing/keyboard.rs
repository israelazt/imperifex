//! A fake keyboard wired to the fake screen: what is typed shows up on it a
//! while later. Like the real one it sends nothing unless the window is in the
//! foreground, and like the fake desktop it fails the test if asked to type
//! into the author's window.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use imperifex_bench_domain::desktop::Rect;
use imperifex_bench_domain::keys::Key;
use imperifex_bench_domain::ports::{Desktop, Keyboard};

use super::desktop::{FakeDesktop, Rig, AUTHORS_HIDDEN, AUTHORS_WINDOW};
use super::processes_of::DesktopProcesses;
use super::screen::{picture_of, FakeScreen, KEY_SIDE};
use super::FakeClock;
use crate::measure_window::TypingPorts;

/// How many pixels typed text, and a single letter, light up.
const A_LINE: usize = 2000;
const A_LETTER: usize = 8;
/// The presses of `a` that come before the ones that count.
pub const BEFORE_COUNTING: usize = 5;
/// The rows of two other things that change: one close enough to the letters to
/// be in their zone, and a clock far from them.
const NEAR_ROW: usize = 15;
const CLOCK_ROW: usize = 300;
const A_MARK: usize = 40;
/// The primary monitor of the typing fakes, with room for the fake windows.
const PRIMARY: Rect = Rect {
    x: 0,
    y: 0,
    w: 1920,
    h: 1080,
};
/// A window that shows its content 100 ms after the launch, which comes once
/// the monitor has been calm for a second.
const OPENS_AT_ONCE: [(u64, usize); 2] = [(0, 0), (1100, 60_000)];

/// How the screen answers to the keys.
#[derive(Debug, Clone, Default)]
pub struct Script {
    /// From a key to the picture that shows it, in milliseconds.
    pub delay_ms: u64,
    /// Nothing is ever presented.
    pub blank: bool,
    /// The presses of `a`, counted from 1, that show nothing.
    pub unseen: Vec<usize>,
    /// Something else changes near the letter this long after each backspace
    /// of a press that counts.
    pub stray_after_erasing: Option<u64>,
    /// The presses of `a` after which the clock changes, 1 ms later.
    pub clock_ticks: Vec<usize>,
    /// Presses of `a` that take another time to show, in milliseconds.
    pub slow: Vec<(usize, u64)>,
    /// How many pixels a letter lights up, if not the usual.
    pub letter_pixels: Option<usize>,
    /// After this many presses of `a`, another window takes the front.
    pub focus_lost_after: Option<usize>,
}

impl Script {
    pub fn showing_after(delay_ms: u64) -> Self {
        Self {
            delay_ms,
            ..Default::default()
        }
    }
}

/// Something that happens when a line is run.
type OnEnter = Rc<dyn Fn(&str)>;

#[derive(Default)]
struct Typed {
    on_enter: Option<OnEnter>,
    on_chord: Option<Rc<dyn Fn()>>,
    log: Vec<String>,
    targets: Vec<u64>,
    times: Vec<Duration>,
    lit: usize,
    letters: usize,
    near: bool,
    clock_on: bool,
}

#[derive(Clone)]
pub struct FakeKeyboard {
    clock: FakeClock,
    desktop: FakeDesktop,
    screen: FakeScreen,
    script: Script,
    typed: Rc<RefCell<Typed>>,
}

impl std::fmt::Debug for FakeKeyboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FakeKeyboard").finish_non_exhaustive()
    }
}

impl FakeKeyboard {
    /// Does `then` with the last text typed each time Enter is pressed.
    pub fn on_enter(&self, then: impl Fn(&str) + 'static) {
        self.typed.borrow_mut().on_enter = Some(Rc::new(then));
    }

    /// Does `then` each time a chord is pressed.
    pub fn on_chord(&self, then: impl Fn() + 'static) {
        self.typed.borrow_mut().on_chord = Some(Rc::new(then));
    }

    fn run_chord(&self) {
        let then = self.typed.borrow().on_chord.clone();
        if let Some(then) = then {
            then();
        }
    }

    fn run_line(&self) {
        let (then, line) = {
            let typed = self.typed.borrow();
            let line = typed.log.iter().rev().find_map(|t| t.strip_prefix("text "));
            (typed.on_enter.clone(), line.map(str::to_string))
        };
        if let (Some(then), Some(line)) = (then, line) {
            then(&line);
        }
    }

    pub fn typed(&self) -> Vec<String> {
        self.typed.borrow().log.clone()
    }

    /// The windows each thing was typed into.
    pub fn targets(&self) -> Vec<u64> {
        self.typed.borrow().targets.clone()
    }

    /// When each thing was typed.
    pub fn times(&self) -> Vec<Duration> {
        self.typed.borrow().times.clone()
    }

    fn accept(&self, hwnd: u64, what: String) -> Result<(), String> {
        assert!(
            hwnd != AUTHORS_WINDOW && hwnd != AUTHORS_HIDDEN,
            "S1 broken: asked to type into the author's window"
        );
        if self.desktop.foreground()? != Some(hwnd) {
            return Err("no está en primer plano (regla S3)".into());
        }
        let mut typed = self.typed.borrow_mut();
        typed.log.push(what);
        typed.targets.push(hwnd);
        typed.times.push(self.clock.now());
        Ok(())
    }

    /// The screen will show `lit` pixels `after_ms` from now.
    fn show(&self, after_ms: u64, lit: usize) {
        if !self.script.blank {
            let at = self.clock.now() + Duration::from_millis(after_ms);
            let mut frame = picture_of(KEY_SIDE, at, lit);
            let typed = self.typed.borrow();
            for (on, row) in [(typed.near, NEAR_ROW), (typed.clock_on, CLOCK_ROW)] {
                if on {
                    let from = row * KEY_SIDE * 4;
                    frame.bgra[from..from + A_MARK * 4].fill(255);
                }
            }
            drop(typed);
            self.screen.show(frame);
        }
    }

    fn letter_size(&self) -> usize {
        self.script.letter_pixels.unwrap_or(A_LETTER)
    }

    fn light(&self, change: impl FnOnce(usize) -> usize) -> usize {
        let mut typed = self.typed.borrow_mut();
        // Kept within the picture, however much is typed.
        typed.lit = change(typed.lit) % 9000;
        typed.lit
    }

    fn letter(&self) {
        let number = {
            let mut typed = self.typed.borrow_mut();
            typed.letters += 1;
            typed.letters
        };
        if self.script.clock_ticks.contains(&number) {
            let was = self.typed.borrow().lit;
            let on = !self.typed.borrow().clock_on;
            self.typed.borrow_mut().clock_on = on;
            self.show(1, was);
        }
        let size = self.letter_size();
        let lit = self.light(|lit| lit + size);
        let slow = self.script.slow.iter().find(|(n, _)| *n == number);
        if !self.script.unseen.contains(&number) {
            self.show(slow.map_or(self.script.delay_ms, |s| s.1), lit);
        }
        if self.script.focus_lost_after == Some(number) {
            self.desktop.state.borrow_mut().foreground_taken_by = Some(AUTHORS_WINDOW);
        }
    }

    fn erase(&self) {
        let size = self.letter_size();
        let lit = self.light(|lit| lit.saturating_sub(size));
        self.show(self.script.delay_ms, lit);
        let counting = self.typed.borrow().letters > BEFORE_COUNTING;
        if let Some(after) = self.script.stray_after_erasing.filter(|_| counting) {
            let on = !self.typed.borrow().near;
            self.typed.borrow_mut().near = on;
            self.show(after, lit);
        }
    }

    /// Text and other keys change much of the screen.
    fn line(&self) {
        let lit = self.light(|lit| lit + A_LINE);
        self.show(self.script.delay_ms, lit);
    }
}

impl Keyboard for FakeKeyboard {
    fn type_text(&self, hwnd: u64, text: &str) -> Result<(), String> {
        self.accept(hwnd, format!("text {text}"))?;
        self.line();
        Ok(())
    }

    fn press(&self, hwnd: u64, key: Key) -> Result<(), String> {
        let name = match key {
            Key::Char(c) => c.to_string(),
            other => format!("{other:?}"),
        };
        self.accept(hwnd, name)?;
        match key {
            Key::Char('a') => self.letter(),
            Key::Backspace => self.erase(),
            Key::Enter => {
                self.line();
                self.run_line();
            }
            Key::Chord(_) => {
                self.line();
                self.run_chord();
            }
            _ => self.line(),
        }
        Ok(())
    }
}

pub type FakeTyping =
    TypingPorts<FakeClock, FakeDesktop, DesktopProcesses, FakeScreen, FakeKeyboard>;

/// A desktop with a screen and a keyboard that answers to the script.
pub fn typing(rig: Rig, script: Script) -> FakeTyping {
    let screen = FakeScreen::new(&rig.ports.clock);
    screen.shown.borrow_mut().opening = OPENS_AT_ONCE.to_vec();
    let keyboard = FakeKeyboard {
        clock: rig.ports.clock.clone(),
        desktop: rig.ports.desktop.clone(),
        screen: screen.clone(),
        script,
        typed: Rc::default(),
    };
    TypingPorts {
        session: rig.ports,
        screen,
        keyboard,
        primary: PRIMARY,
    }
}
