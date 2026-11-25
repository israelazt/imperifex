//! Probe 1.1: how fast desktop duplication hands over the frames of the primary
//! monitor, how late they arrive, and what reading a region of them costs.
//! Read-only: it opens no window and presses nothing.
//!
//! `cargo run --release -p imperifex-bench-platform --example probe_duplication`

#[cfg(windows)]
#[path = "common/stats.rs"]
mod stats;

#[cfg(windows)]
#[path = "probe_duplication/capture.rs"]
mod capture;

#[cfg(windows)]
fn main() -> Result<(), String> {
    const SECONDS: i64 = 5;
    let screen = capture::open_primary()?;
    println!("{}", screen.description);
    let mut reader = capture::RegionReader::new(&screen, 1000, 600)?;

    let (mut delays, mut copies) = (Vec::new(), Vec::new());
    let (mut presented, mut pointer_only, mut accumulated) = (0u32, 0u32, 0u32);
    let mut checksum = 0u64;
    let start = stats::ticks();
    let end = start + SECONDS * stats::ticks_per_second();
    while stats::ticks() < end {
        let Some(frame) = screen.next_frame(100)? else {
            continue;
        };
        let received = stats::ticks();
        if frame.info.LastPresentTime == 0 {
            pointer_only += 1;
            continue;
        }
        presented += 1;
        accumulated += frame.info.AccumulatedFrames;
        delays.push(stats::ms(frame.info.LastPresentTime, received));
        let before = stats::ticks();
        checksum = checksum.wrapping_add(reader.read(&frame)?);
        copies.push(stats::ms(before, stats::ticks()));
    }
    let seconds = stats::ms(start, stats::ticks()) / 1000.0;

    println!("duración: {seconds:.2} s (suma de control de lo leído: {checksum:x})");
    println!(
        "frames presentados: {presented} ({:.1}/s); acumulados según DXGI: {accumulated} ({:.1}/s)",
        f64::from(presented) / seconds,
        f64::from(accumulated) / seconds
    );
    println!(
        "avisos solo de puntero (LastPresentTime = 0): {pointer_only} ({:.1}/s)",
        f64::from(pointer_only) / seconds
    );
    println!(
        "retraso de LastPresentTime a recibirlo: {}",
        stats::spread(delays)
    );
    println!("copiar y leer 1000x600: {}", stats::spread(copies));
    Ok(())
}

#[cfg(not(windows))]
fn main() {}
