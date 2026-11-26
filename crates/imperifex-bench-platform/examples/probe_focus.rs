//! Probe 1.4 🖥: which way of taking the foreground works, and whether a key
//! reaches the new window. It OPENS two Warp windows, types in one and closes
//! both. Do not use the computer while it runs.
//!
//! `cargo run --release -p imperifex-bench-platform --example probe_focus -- <folder for pictures>`

#[cfg(windows)]
#[path = "common/processes.rs"]
mod processes;

#[cfg(windows)]
#[path = "common/warp.rs"]
mod warp;

#[cfg(windows)]
#[path = "probe_focus/input.rs"]
mod input;

#[cfg(windows)]
mod run {
    use std::collections::HashSet;
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    use imperifex_bench_domain::ports::Desktop;
    use imperifex_bench_platform::WindowsDesktop;

    use crate::{input, warp};

    const OPENING: Duration = Duration::from_secs(8);
    const CLOSING: Duration = Duration::from_secs(5);
    const SHOWN: Duration = Duration::from_secs(2);
    const CAPTURE: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/probe_focus/capture.ps1"
    );

    fn open(name: &str) -> Result<warp::Opened, String> {
        println!("== abriendo la ventana {name}");
        let before = input::foreground();
        let opened = warp::open_new_window(OPENING)?;
        let now = input::foreground();
        println!(
            "ventana {name} = {:#x}; primer plano antes {before:#x}, después {now:#x}; \
             ¿lo tomó sola?: {}",
            opened.window.hwnd,
            now == opened.window.hwnd
        );
        Ok(opened)
    }

    /// A picture of the foreground window, only while it is `target`.
    fn picture(target: u64, to: &Path) -> Result<(), String> {
        if input::foreground() != target {
            return Err("el primer plano cambió: no se hace la foto".into());
        }
        let status = std::process::Command::new("powershell")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", CAPTURE])
            .arg("-Out")
            .arg(to)
            .status()
            .map_err(|error| format!("no se pudo hacer la foto: {error}"))?;
        println!("foto {} ({status})", to.display());
        Ok(())
    }

    fn type_and_erase(target: u64, folder: &Path) -> Result<(), String> {
        std::thread::sleep(SHOWN);
        picture(target, &folder.join("1-antes.png"))?;
        input::type_char(target, 'a')?;
        println!("pulsada «a»");
        std::thread::sleep(SHOWN);
        picture(target, &folder.join("2-con-a.png"))?;
        input::press_backspace(target)?;
        println!("pulsado Retroceso");
        std::thread::sleep(SHOWN);
        picture(target, &folder.join("3-borrada.png"))
    }

    fn focus_and_type(a: u64, b: u64, folder: &Path) -> Result<(), String> {
        let own = HashSet::from([a, b]);
        if input::foreground() == a {
            return Err("A ya está en primer plano: no hay nada que probar".into());
        }
        if !own.contains(&input::foreground()) {
            println!("aviso: el primer plano no es una ventana propia");
        }
        let method = input::take_foreground(a, &own)?;
        println!("funcionó: {method}");
        type_and_erase(a, folder)
    }

    fn close(opened: &mut warp::Opened, name: &str) {
        match warp::close_window(opened, CLOSING) {
            Ok(true) => println!("ventana {name} cerrada"),
            Ok(false) => println!("la ventana {name} NO se cerró: ciérrala a mano"),
            Err(error) => println!("ventana {name}: {error}"),
        }
    }

    pub fn main() -> Result<(), String> {
        let folder = PathBuf::from(
            std::env::args()
                .nth(1)
                .ok_or("falta la carpeta para las fotos")?,
        );
        let start = input::foreground();
        println!("primer plano al empezar: {start:#x}");
        let mut a = open("A")?;
        let mut b = match open("B") {
            Ok(b) => b,
            Err(error) => {
                close(&mut a, "A");
                return Err(error);
            }
        };
        let result = focus_and_type(a.window.hwnd, b.window.hwnd, &folder);
        // Whatever happened, what was opened is closed.
        close(&mut b, "B");
        close(&mut a, "A");
        for opened in [&a, &b] {
            for process in warp::survivors(&opened.seen)? {
                println!("sigue vivo: {} pid {}", process.image, process.pid);
            }
        }
        let end = input::foreground();
        let known = WindowsDesktop.windows()?.iter().any(|w| w.hwnd == end);
        println!(
            "primer plano al terminar: {end:#x} (¿el del principio?: {}, ¿existe?: {known})",
            end == start
        );
        result
    }
}

#[cfg(windows)]
fn main() -> Result<(), String> {
    run::main()
}

#[cfg(not(windows))]
fn main() {}
