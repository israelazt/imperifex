//! Opening one new Warp window and closing it, touching nothing that was there
//! before (S1) and never ending a process by force (S2).

use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, HashSet};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use imperifex_bench_domain::profile::{expand_env, find, parse_profiles};

use crate::desktop::{self, Window};
use crate::processes::{processes, Process};

const PROFILES: &str = include_str!("../../../../bench/terminals.json");
const IMAGE: &str = "warp.exe";
/// A smaller new window is a popup or a notice, not a terminal window.
const SMALLEST: (i32, i32) = (400, 300);
const EVERY: Duration = Duration::from_millis(50);

/// How a new window is asked for.
#[derive(Debug, Clone, Copy)]
enum Launch {
    Program,
    Uri,
}

/// What existed before the launch. Nothing in it is ever touched.
pub struct Photo {
    pids: HashSet<u32>,
    windows: HashSet<isize>,
    hidden: HashSet<isize>,
}

impl Photo {
    pub fn take() -> Result<Self, String> {
        let windows = desktop::windows()?;
        Ok(Self {
            pids: processes()?.iter().map(|p| p.pid).collect(),
            hidden: windows
                .iter()
                .filter(|w| !w.visible)
                .map(|w| w.hwnd)
                .collect(),
            windows: windows.iter().map(|w| w.hwnd).collect(),
        })
    }
}

fn start(launch: Launch) -> Result<(), String> {
    let mut command = match launch {
        Launch::Program => {
            let profiles = parse_profiles(PROFILES)?;
            let warp = find(&profiles, "warp")?;
            let exe = expand_env(&warp.exe, &|name| std::env::var(name).ok())?;
            let mut command = Command::new(exe);
            command.args(&warp.new_window_args);
            command
        }
        Launch::Uri => {
            let mut command = Command::new("cmd");
            command.args(["/c", "start", "", "warp://action/new_window"]);
            command
        }
    };
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|child| println!("lanzado ({launch:?}), pid {}", child.id()))
        .map_err(|error| format!("no se pudo lanzar ({launch:?}): {error}"))
}

/// What appeared while watching.
#[derive(Default)]
pub struct Seen {
    pub processes: BTreeMap<u32, Process>,
    pub exited: HashSet<u32>,
    /// Windows that did not exist before and were visible at some moment.
    pub windows: BTreeMap<isize, Window>,
    shown_again: HashSet<isize>,
}

impl Seen {
    fn look(&mut self, photo: &Photo, at: Duration) -> Result<(), String> {
        let now = processes()?;
        for process in now.iter().filter(|p| !photo.pids.contains(&p.pid)) {
            if let Entry::Vacant(slot) = self.processes.entry(process.pid) {
                println!(
                    "{:>6} ms · proceso nuevo: {} pid {} (padre {})",
                    at.as_millis(),
                    process.image,
                    process.pid,
                    process.parent
                );
                slot.insert(process.clone());
            }
        }
        for pid in self.processes.keys() {
            if !now.iter().any(|p| p.pid == *pid) && self.exited.insert(*pid) {
                println!("{:>6} ms · terminó el pid {pid}", at.as_millis());
            }
        }
        for window in desktop::windows()?.into_iter().filter(|w| w.visible) {
            self.note(photo, window, at);
        }
        Ok(())
    }

    fn note(&mut self, photo: &Photo, window: Window, at: Duration) {
        if photo.windows.contains(&window.hwnd) {
            if photo.hidden.contains(&window.hwnd) && self.shown_again.insert(window.hwnd) {
                println!(
                    "{:>6} ms · una ventana oculta que ya existía se ve: pid {} clase {} (no se toca)",
                    at.as_millis(),
                    window.pid,
                    window.class
                );
            }
        } else if let Entry::Vacant(slot) = self.windows.entry(window.hwnd) {
            println!(
                "{:>6} ms · ventana nueva: {:#x} pid {} clase {} {}x{} «{}»",
                at.as_millis(),
                window.hwnd,
                window.pid,
                window.class,
                window.width,
                window.height,
                window.title
            );
            slot.insert(window);
        }
    }
}

pub fn watch(photo: &Photo, time: Duration, seen: &mut Seen) -> Result<(), String> {
    let start = Instant::now();
    while start.elapsed() < time {
        seen.look(photo, start.elapsed())?;
        std::thread::sleep(EVERY);
    }
    Ok(())
}

/// Warp's new windows of a terminal's size that are still on screen.
fn candidates(photo: &Photo) -> Result<Vec<Window>, String> {
    let warp: HashSet<u32> = processes()?
        .iter()
        .filter(|p| p.is(IMAGE))
        .map(|p| p.pid)
        .collect();
    Ok(desktop::windows()?
        .into_iter()
        .filter(|w| w.visible && !photo.windows.contains(&w.hwnd) && warp.contains(&w.pid))
        .filter(|w| w.width >= SMALLEST.0 && w.height >= SMALLEST.1)
        .collect())
}

pub struct Opened {
    pub photo: Photo,
    pub window: Window,
    pub seen: Seen,
}

/// Why there is no window to work with.
enum NotOpened {
    /// Not a single new window of Warp showed up: another way of asking may be tried.
    Nothing,
    /// Something showed up or failed: stop, so that nothing else gets opened.
    Stop(String),
}

/// Launches and watches for `time`. It answers only if exactly one window can
/// be the new one; otherwise nothing is touched.
fn open_window(launch: Launch, time: Duration) -> Result<Opened, NotOpened> {
    let photo = Photo::take().map_err(NotOpened::Stop)?;
    start(launch).map_err(NotOpened::Stop)?;
    let mut seen = Seen::default();
    watch(&photo, time, &mut seen).map_err(NotOpened::Stop)?;
    let mut found = candidates(&photo).map_err(NotOpened::Stop)?;
    match (found.pop(), found.is_empty()) {
        (Some(window), true) => Ok(Opened {
            photo,
            window,
            seen,
        }),
        (None, _) if !any_of_warp(&seen).map_err(NotOpened::Stop)? => Err(NotOpened::Nothing),
        (None, _) => Err(NotOpened::Stop(
            "Warp mostró ventanas nuevas, pero ninguna sirve: revísalas a mano".into(),
        )),
        _ => Err(NotOpened::Stop(
            "hay varias ventanas nuevas de Warp: no se toca ninguna, ciérralas a mano".into(),
        )),
    }
}

fn any_of_warp(seen: &Seen) -> Result<bool, String> {
    let now = processes()?;
    Ok(seen.windows.values().any(|window| {
        now.iter()
            .chain(seen.processes.values())
            .any(|p| p.pid == window.pid && p.is(IMAGE))
    }))
}

/// Opens a window with the program and, only if that showed nothing at all,
/// with the `warp://` address.
pub fn open_new_window(time: Duration) -> Result<Opened, String> {
    let first = match open_window(Launch::Program, time) {
        Err(NotOpened::Nothing) => {
            println!("lanzando el programa no salió ninguna ventana; se prueba warp://");
            open_window(Launch::Uri, time)
        }
        other => other,
    };
    first.map_err(|why| match why {
        NotOpened::Nothing => "tampoco salió ninguna ventana con warp://".to_string(),
        NotOpened::Stop(reason) => reason,
    })
}

/// Asks the opened window to close and reports what happens for `time`.
/// It says whether the window went away.
pub fn close_window(opened: &mut Opened, time: Duration) -> Result<bool, String> {
    let hwnd = opened.window.hwnd;
    if !candidates(&opened.photo)?.iter().any(|w| w.hwnd == hwnd) {
        return Err("la ventana ya no está o no es de las nuevas: no se toca".into());
    }
    desktop::ask_to_close(hwnd)?;
    println!("WM_CLOSE enviado a {hwnd:#x}");
    watch(&opened.photo, time, &mut opened.seen)?;
    let gone = !desktop::windows()?
        .iter()
        .any(|w| w.hwnd == hwnd && w.visible);
    Ok(gone)
}

/// The processes that appeared and are still alive.
pub fn survivors(seen: &Seen) -> Result<Vec<Process>, String> {
    let alive: HashSet<u32> = processes()?.iter().map(|p| p.pid).collect();
    Ok(seen
        .processes
        .values()
        .filter(|p| alive.contains(&p.pid))
        .cloned()
        .collect())
}
