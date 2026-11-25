//! Probe 1.3 🖥: how Warp opens a new window and how that window closes. It
//! OPENS a Warp window and closes it. Do not use the computer while it runs.
//!
//! `cargo run --release -p imperifex-bench-platform --example probe_warp_window`

#[cfg(windows)]
#[path = "common/desktop.rs"]
mod desktop;

#[cfg(windows)]
#[path = "common/processes.rs"]
mod processes;

#[cfg(windows)]
#[path = "common/warp.rs"]
mod warp;

#[cfg(windows)]
fn main() -> Result<(), String> {
    use std::time::Duration;
    const OPENING: Duration = Duration::from_secs(10);
    const CLOSING: Duration = Duration::from_secs(5);

    for process in processes::processes()?.iter().filter(|p| p.is("warp.exe")) {
        println!(
            "ya existía: warp.exe pid {} (padre {})",
            process.pid, process.parent
        );
    }
    let mut opened = match warp::open_window(warp::Launch::Program, OPENING) {
        Err(warp::NotOpened::Nothing) => {
            println!("lanzando el programa no salió ninguna ventana; se prueba warp://");
            warp::open_window(warp::Launch::Uri, OPENING)
        }
        other => other,
    }
    .map_err(|why| match why {
        warp::NotOpened::Nothing => "tampoco salió ninguna ventana con warp://".to_string(),
        warp::NotOpened::Stop(reason) => reason,
    })?;
    println!(
        "la ventana nueva es {:#x}, del pid {} ({}x{})",
        opened.window.hwnd, opened.window.pid, opened.window.width, opened.window.height
    );
    for process in warp::survivors(&opened.seen)? {
        println!(
            "vivo con la ventana abierta: {} pid {} (padre {})",
            process.image, process.pid, process.parent
        );
    }

    let gone = warp::close_window(&mut opened, CLOSING)?;
    println!("¿la ventana se cerró?: {gone}");
    println!(
        "ventanas nuevas vistas en total: {}",
        opened.seen.windows.len()
    );
    for process in warp::survivors(&opened.seen)? {
        println!(
            "vivo después de cerrar: {} pid {} (padre {})",
            process.image, process.pid, process.parent
        );
    }
    if !gone {
        println!("no se hace nada más: ciérrala a mano y anota qué pedía");
    }
    Ok(())
}

#[cfg(not(windows))]
fn main() {}
