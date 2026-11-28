//! Prints the system's theme and language, and every notice of a change for
//! 60 seconds. Read only: change the theme by hand in Windows' settings.
//!
//! `cargo run -p imperifex-platform --example watch_appearance`

#[cfg(windows)]
fn main() -> Result<(), String> {
    use std::sync::mpsc;
    use std::sync::Arc;
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    use imperifex_domain::ports::{SystemAppearance, SystemLanguage};
    use imperifex_platform::{WindowsAppearance, WindowsLanguage};

    /// The time of day in UTC, to compare with when the setting was changed.
    fn now() -> String {
        let since = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let seconds = since.as_secs() % 86_400;
        format!(
            "{:02}:{:02}:{:02}.{:03} UTC",
            seconds / 3600,
            seconds / 60 % 60,
            seconds % 60,
            since.subsec_millis()
        )
    }

    let appearance = WindowsAppearance;
    println!("idioma: {:?}", WindowsLanguage.locale());
    println!("oscuro: {}", appearance.dark()?);
    let (sender, changes) = mpsc::channel();
    appearance.watch(Arc::new(move || {
        let _ = sender.send(now());
    }))?;
    println!("esperando cambios 60 s…");
    let end = Instant::now() + Duration::from_secs(60);
    while let Ok(at) = changes.recv_timeout(end.saturating_duration_since(Instant::now())) {
        println!("{at} aviso: oscuro = {}", appearance.dark()?);
    }
    println!("fin");
    Ok(())
}

#[cfg(not(windows))]
fn main() {
    println!("solo en Windows");
}
